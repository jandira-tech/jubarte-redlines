<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Jubarte-rs: Port the Mined minimax-docx Checks into `validate` — Implementation Plan — 2026-10-23

Provenance: an agent swarm (six agents, two waves) assessed the MIT
`minimax-docx` project's validation assets — `wml-subset.xsd`,
`common-types.xsd` and the C# `XsdValidator`/`GateCheckValidator`/
`ValidationResult` — for porting into jubarte's validation subsystem.
This plan records what is ported, what is not and why, and the TDD
batches that land it. Two decisions were taken by the owner on
2026-10-23: **Word evidence comes first** (Batch 0 precedes the ring1
batches), and **G4 (`DOC_DEFAULTS_CHANGED`) is in scope**.

## Scope decisions (from the swarm's evidence)

Ported, hand-coded — five ring1 codes and three edits-set codes:

- `MARKUP_ID_INVALID` — `w:id` present and integer on revision markup
  and range/reference marks.
- `REVISION_DATE_MALFORMED` — a present, non-empty `w:date` on revision
  markup that does not lexically match xs:dateTime.
- `PAGE_GEOMETRY_INVALID` — `pgSz` `@w`/`@h` required integers; `pgMar`
  `top/right/bottom/left` present-but-non-integer.
- `GRIDSPAN_INVALID` — `gridSpan/@val` a positive integer.
- `TAB_STOP_INVALID` — `w:tab` under `w:tabs` missing `@val`/`@pos`
  (run-level `w:tab` is attribute-less and never fires).
- `STYLE_DROPPED` (G1), `PAGE_SETUP_CHANGED` (G2+G3), and
  `DOC_DEFAULTS_CHANGED` (G4) as edits-set fidelity rules against the
  original document.

Not ported, with the reason on record:

- **Runtime XSD validation and the schema files themselves.** No
  production-grade pure-Rust XSD 1.0 engine exists (October 2026); the
  shipped schema set is broken standalone (`xs:import` without
  `schemaLocation`, `common-types.xsd` referenced by nothing); and the
  subset omits `fldChar`/`instrText`, `proofErr`, `moveFrom`/`moveTo`,
  every `*PrChange`, `rsid*` attributes, `mc:Ignorable` and
  `w14:paraId`, with no `xs:anyAttribute` — a perfectly valid modern
  Word document fails it. It is a generator-side gate, inverted for a
  validator that consumes arbitrary Word files. XSD conformance stays
  with the .NET oracle (`tools/validate-docx`), as
  `src/validate.rs`'s module doc already assigns it. Hand-coding means
  nothing is vendored and no REUSE/licensing change is needed.
- **Bookmark start/end pairing.** Already reported by the triage ring:
  `check_bookmarks` (`src/debug.rs`) emits `BOOKMARK_START_WITHOUT_END`
  and siblings, which flow into `validate()` because they are not in
  `RING1_CODES`. Only the id-shape gap survives: the triage `attr()`
  helper defaults a missing `w:id` to `""`, so id-less marks pair
  silently — that folds into `MARKUP_ID_INVALID`.
- **The C# semantics as written.** document.xml-only blinders,
  `int.TryParse`-to-silent-0 comparisons, last-`sectPr`-only, the
  `entry!` NRE, HashSet duplicate-style collapse, ascii-only font on
  the first default paragraph style (G4 reads `docDefaults` instead),
  and line:column locations (jubarte locates by `element_path`; the
  `Dom` keeps no source positions). `ValidationResult.cs` is subsumed
  by `Finding`.
- **Color hex pattern, props-after-content order, hyperlink-without-target,
  and G5 heading font hierarchy.** The color rule false-positives on
  8-digit ARGB `shd` values and the renderer already tolerates non-6-hex;
  Word's reader is lenient on property-element order and jubarte already
  normalizes order at write time (`src/comparer/finalize.rs`); jubarte's
  own repair deliberately emits hyperlinks with neither `r:id` nor
  anchor, so that check would condemn jubarte's output; G5 duplicates
  the spirit of `HEADING_SKIP` with opinion, not fidelity.
- **No new repairs** — `repair()` is untouched; no new dependencies —
  the crate stays wasm-clean.

TDD discipline (the Iron Law): every production change lands behind a
test that was watched failing for the right reason — a probe filtering
`ring1()` output by the new code, failing `assert_eq!(len, 1)` on an
empty vec, never a compile error. One behavior per test; each probe
embeds a healthy sibling so the negative case is asserted in the same
test (house pattern: `probe_style_name_with_a_comma_fails`,
tests/m_validity_ring1.rs). Minimal code to pass; refactor only while
green; no `#[allow]`. All cargo commands run from the repo root,
sequentially, through the mbx shim.

## Batch 0 — Word evidence first

Goal: a verdict per broken shape — *Word refuses/repairs* (→
`word_fatal`, severity `error`) vs *Word opens cleanly* (→ warning
lead) — recorded like existing rows ("Word probes 2026-10-23, #NNNN").

1. Build the probe files (disposable, `/tmp/ring1_probes/`, short
   stems) with a small python `zipfile` script mirroring
   `zip_with_parts`' minimal package. One file per shape:
   `p01` `w:ins w:id="abc"`; `p02` `w:ins` with no `w:id`; `p03`
   `w:bookmarkStart` with no `w:id`; `p04` `w:ins w:date="yesterday"`;
   `p05` `w:date=""` (empty is absent-equivalent — not flagged); `p06`
   `w:pgSz w:w="a4" w:h="15840"`; `p07` `w:pgMar w:top="wide" …`; `p08`
   `w:gridSpan w:val="0"`; `p09` `w:gridSpan` with no `w:val`; `p10`
   `w:tabs><w:tab w:val="left"/>` (no pos); `p11`
   `w:tabs><w:tab w:pos="720"/>` (no val); `p12` healthy control (must
   convert cleanly — it validates the harness itself).
2. Drive Word per AGENTS.md: `scripts/word_pdf.py` from the
   neurotic_docx_bench root, one file at a time under fresh names; the
   watchdog pointed at the paths the job actually grows; a file that
   fails ("document loaded empty") is refused-or-repairable
   (word_pdf.py answers the repair prompt No); a clean PDF means Word
   opens it. Skip after two failures on one file and record "unknown"
   (that shape then stays a warning).
3. For every refused shape: double-check by opening the file in Word
   alone, under a fresh name, before calling it Word-fatal; run
   `tools/validate-docx` on all probes and record its verdict
   alongside (divergences recorded, not resolved).
4. Output: the verdict table (shape → Word → ooxml validator). It
   drives every `traits()` arm and severity assertion in Batches 1–2.
   No repo changes in this batch.

## Batch 1 — ring1: `MARKUP_ID_INVALID`, `REVISION_DATE_MALFORMED`

RED 1 — in `tests/m_validity_ring1.rs`, `probe_non_numeric_revision_id_fails`:
one `w:ins w:id="abc"` + one healthy `w:ins w:id="7"` in the same
paragraph; filter `ring1()` by `MARKUP_ID_INVALID`; assert `len == 1`,
the message names `abc`, and the Batch-0 verdict
(`word_fatal`/`severity`). Focused variants: missing id on `ins`,
missing id on `bookmarkStart`, healthy `bookmarkStart`/`End` pair
emits none. Watch each fail:
`cargo nextest run -E 'test(probe_markup_id)'`.

GREEN 1 — minimal: `fn check_markup_ids(pkg: &PartFs, out: &mut Vec<Finding>)`
walking the elements `has_unique_revision_id` already names
(`src/validate.rs`) plus `bookmarkStart`/`bookmarkEnd`,
`commentRangeStart`/`End`, `commentReference`, `footnoteReference`,
`endnoteReference`: `w:id` must be present and parse as `i64` (attribute
*shape only* — never id resolution, which `COMMENT_ANCHOR_ORPHAN`
owns). Wire the call into `ring1()` after `check_revision_and_drawing_ids`;
add the code to `RING1_CODES`, bumping `[&str; 28]` → `[&str; 29]`; add
a `traits()` arm only if Batch 0 said Word-fatal; add the module-doc
row with the probe citation. Re-run: probe green + the whole file:
`cargo nextest run --test m_validity_ring1`.

RED 2 / GREEN 2 — same cycle for `REVISION_DATE_MALFORMED`:
`probe_unparseable_revision_date_fails` (broken + healthy sibling;
`w:date=""` asserted *not* flagged). Minimal impl: extend the same
walk — on revision-markup elements, a present, non-empty `w:date` must
lexically match xs:dateTime (`YYYY-MM-DDTHH:MM:SS[.frac][Z|±HH:MM]`,
hand-written matcher in `validate.rs` — no date crate; the matcher gets
its own watched-failing unit tests beside the module's existing
in-file tests). `RING1_CODES` → 30.

Gate: `cargo nextest run -E 'test(healthy_compare_output_passes_ring1)'`
and `cargo nextest run --test md_roundtrip_valid` (the corpus gate —
every new finding on healthy fixtures is a blocker: read the fixture
XML, narrow the check or drop it, never loosen the gate).

## Batch 2 — ring1: `PAGE_GEOMETRY_INVALID`, `GRIDSPAN_INVALID`, `TAB_STOP_INVALID`

One code per RED→GREEN cycle, all landing in one
`fn check_numeric_attribute_shapes(pkg, out)` (modeled on
`check_data_bindings_have_stores`: targeted
`dom.descendants(root, Some(&W::name(...)))` + `dom.attribute`, parse
idiom `.and_then(|v| v.parse::<i64>().ok())`):

- `PAGE_GEOMETRY_INVALID`: `pgSz` present → `@w` and `@h` required
  integers; `pgMar` attrs `top/right/bottom/left` → present-but-non-
  integer fires, absent does not (`Option`, never silent-0 — the C#
  bug must not survive). Probes: `w:w="a4"`; `w:top="wide"`; healthy
  default sectPr emits none.
- `GRIDSPAN_INVALID`: `gridSpan/@val` must be a positive integer (`0`
  and non-numeric both probe).
- `TAB_STOP_INVALID`: `w:tab` only when its direct parent is `w:tabs`
  (run-level `w:tab` is legitimately attribute-less — `src/fields.rs`
  shows the distinction): `@val` and `@pos` per Batch-0 verdicts.

Each cycle: probe red → implement minimal → wire in `ring1()` +
`RING1_CODES` (→ 31, 32, 33) + module-doc row + Batch-0-driven
`traits()`/severity. Same healthy gates after each.

REFACTOR (stay green): merge Batch 1+2 walks where they share a
`parse_part` pass; align message tone with existing findings ("element
+ Word consequence + evidence").

## Batch 3 — edits set: `STYLE_DROPPED`, `PAGE_SETUP_CHANGED`, `DOC_DEFAULTS_CHANGED`

Semantics fix the C# bugs by design: `Option`-parsed (absent ≠ 0),
removals only for styles, page setup compared against the
reject-changes output so a tracked `sectPrChange` by the reviewed
author cannot fire, docDefaults (not first paragraph style) for the
font. These are edits-set errors by construction (`as_edits` forces
`set="edits"`, `severity="error"`), so no Word probes are needed —
fidelity, not Word validity.

Per code, RED first in `tests/validate_public.rs` beside the
FOREIGN_AUTHOR probes, building original/edited pairs with
`common::docx::{docx, para, docx_with_sect_pr, Part, docx_with}` (a
styles part via `Part { name: "word/styles.xml", content_type:
"application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles", … }`;
if `docx_with_sect_pr` doesn't wire that rel cleanly, extend the helper
minimally rather than hand-rolling zips):

- `STYLE_DROPPED`: original styles.xml holds `A`+`B`, edited holds `A`
  (+ adds `C`) → exactly one finding naming `B`; additions never fire;
  neither-side-has-styles → no finding. Compare by
  `w:style/@w:styleId`, duplicates counted (no HashSet collapse).
- `PAGE_SETUP_CHANGED`: edited's final body `sectPr` differs (`pgSz`
  `w`/`h`, `pgMar` four attrs; absent ≠ present) → finding naming the
  first differing attribute; a `w:sectPrChange` by the reviewed author
  must NOT fire (assert via a pair whose difference is exactly a
  tracked sectPr change).
- `DOC_DEFAULTS_CHANGED`: edited
  `docDefaults/rPrDefault/rFonts/@w:ascii` differs from original's →
  finding; same font → none. (`ascii`-only narrowing kept
  deliberately; documented in the code comment.)

GREEN: one
`fn check_template_fidelity(original: &PartFs, edited: &PartFs, reverted: &PartFs, out: &mut Vec<Finding>)`
modeled on `check_dropped_permissions`, called from `audit_tracked`
after `reverted` is computed — styles and docDefaults compare
`original` vs raw `edited`, page setup compares `original` vs
`reverted`. Document the three codes in `audit_tracked`'s doc comment
the way `PERMISSION_RANGE_DROPPED` is. No `RING1_CODES`/CLI/README
changes (edits codes flow through generic JSON; `validate --rules
edits` already exists). REFACTOR: thread the already-opened `PartFs`s
instead of reopening, while `--test validate_public` stays green.

## Batch 4 — sweep and evidence

1. `cargo fmt`; `cargo clippy --all-targets -- -D warnings`.
2. Targeted suites: `cargo nextest run --test m_validity_ring1 --test
   validate_public --test md_roundtrip_valid --test audit_rules --test
   validate_cli`.
3. CLI smoke: `target/debug/jubarte --help` and
   `target/debug/jubarte validate --help`.
4. `scripts/gen_docs.sh --check` (expect no drift — no CLI surface
   changed).
5. One full `cargo miri nextest run` — ring1 gates every produced
   package, this reaches across the crate; know its cost before
   starting (binary-invoking tests fail under it by design).
6. CHANGELOG: entries under `### Added` in the house bold-lead +
   issue-number format (one for the markup/numeric ring1 codes group,
   one for the edits rule group), citing the Word-probe evidence
   dates; module-doc rows carry the same citations.

## Files touched

`src/validate.rs` (checks, `ring1()`, `RING1_CODES` count, `traits()`
per evidence, module doc, `audit_tracked` +
`check_template_fidelity`), `tests/m_validity_ring1.rs` (≈7 probe
tests + matcher unit tests), `tests/validate_public.rs` (≈5 edits
probes), `tests/common/docx.rs` (only if the styles-part helper needs
extending), `CHANGELOG.md`. Not touched: CLI, Python, WASM bindings,
`repair()`, README (ring1 codes are not listed there).

## Abort/narrow criteria

Any new finding on a healthy fixture (corpus gate) blocks that check:
inspect the fixture's XML first; narrow the rule or drop the code —
never widen tolerance. A Word verdict of "unknown" (two failures) pins
that shape to a warning lead. If Batch 0 shows Word opens everything
cleanly, all five ring1 codes still land as documented warning leads —
the batch's value is then the recorded evidence itself.

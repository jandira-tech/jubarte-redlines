<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Deferred notes — workstream S (style-chain resolution)

Findings surfaced while implementing `merge_revised_style_definitions`
(`src/document_comparer.rs`) that are **noted, not implemented**, per the
scope policy: go for the general rule, not the per-fixture special case.

All corpus figures below are measured against
`/Users/arthrod/temp/T/neurotic_docx_bench` at the 2026-08-04 state, using the
`jubarte-rust` 763-document run `019fcc5d-34e6-7029-95d9-463d5513fe7c`.

---

## S-D1 — the `rstyle` / `combos` fixture family is not a style-chain problem

This is the most important note here, because the workstream was targeted at
that family and **it does not move it.**

Measured over the 25 `rstyle`/`combos` pairs in
`corpus/word_redlines_superdoc/centralized_mapping.csv`, comparing the engine's
output before and after this change:

| | count |
|---|---|
| pairs whose output `document.xml` changed | **0 / 25** |
| pairs whose output `styles.xml` changed | 15 / 25 |
| pairs with a *live* colliding style (the population S addresses) | 6 / 25 |

The six live collisions are on `Normal`, `ListParagraph`, `Heading1`,
`TableGrid`, `LightGrid-Accent1` — incidental document chrome, not the character
styles the fixtures are about. The lowest-scoring members of the family —
`highlight × bold` 41.05, `highlight × italic` 41.18, `italic × rFonts` 43.59,
`size × strike` 44.15 — have **zero** live collisions and **zero** body-markup
change.

What those fixtures actually diverge on, measured on
`strike_rstyle_linked_combos × underline_rstyle_linked_combos` (score 54.13)
against the Word oracle:

| | ours | Word oracle |
|---|---:|---:|
| `w:p` | 73 | 47 |
| `w:r` | 107 | 197 |
| `w:rPrChange` | **0** | **8** |
| `w:pPrChange` | 0 | 1 |

Word correlates the two testers' structurally parallel lines paragraph-to-
paragraph and marks the run-level format change; we treat them as unrelated and
emit delete-paragraph + insert-paragraph. The 26 extra paragraphs are the page
drift the pixel scorer is charging for. **That is correlation granularity, not
style resolution** — it belongs to the R2 cluster-lift stage, not to S.

The style ids in that family are disjoint between sides (`SD_StrikeChar` vs
`SD_UnderlineSingleChar`), they canonicalize identically to the oracle
(`SDStrikeChar`, …), and both definitions already reach the output. There is
nothing for S to repair.

The token analysis in `plans/docxodus-version-diff.md` §7 grouped
`rstyle`/`combos`/`linked`/`styles`/`ooxml` as one "style inheritance" family on
the strength of the fixture *names*. The names are about what the fixture
*tests*, not about why the engine loses on it.

## S-D2 — the population workstream S does address

Style-id collisions with materially different definitions, where the id is
referenced by either document:

- **136 of 597** corpus pairs carry at least one.
- Those pairs score **mean 59.65 / median 55.89**, against **79.38 / 86.22** for
  the 434 without one (scored subset: 103 vs 434 pairs).
- **52.5%** of the 564 Word oracle redlines carry style-level
  `w:pPrChange`/`w:rPrChange`; the median oracle marks 13–15 styles.

Caveat that must travel with those numbers: the correlation is not causation.
A live collision is also a proxy for "both sides are real Word documents with
full, differing stylesheets", which is independently hard. The gap is the
addressable population, not the predicted lift.

## S-D3 — Word normalizes the value it records; we record it verbatim

Oracle `two_column_two_page × vrect_node`, `Title`: A declares
`<w:spacing w:after="300" w:line="240" w:lineRule="auto"/>` and Word's
`w:pPrChange` holds `<w:spacing w:after="300"/>` — `line`/`lineRule` dropped
because they restate the inherited value. Word does the same inside
`w:rPrChange` (it drops `w:lang` when it matches `docDefaults`).

We write each side's declared block verbatim. The live block — which is what
renders and therefore what the pixel score sees — is correct either way; only
the recorded history differs. Per-fixture polish.

## S-D4 — table-style properties are not compared

`merge_revised_style_definitions` handles `w:pPr` and `w:rPr` only.
`w:tblPr` / `w:trPr` / `w:tcPr` / `w:tblStylePr` differences between two
definitions of the same table style are neither detected nor recorded (Word
writes `w:tblPrChange` / `w:trPrChange` / `w:tcPrChange`). `TableGrid` and
`LightGrid-Accent1` collisions in the corpus fall in this gap.

Not done because it is a distinct property family with its own change elements
and its own oracle evidence, and because the paragraph/character styles carry
the visible weight.

## S-D5 — per-attribute inheritance is limited to `w:rFonts` and `w:lang`

`ATTR_MERGED_PROPS` covers the two elements whose per-attribute inheritance the
corpus evidence forced (`Hello_docx_world × multi_image_types`: comparing
`w:rFonts` whole marked seven `Heading*Char` styles Word leaves alone).

`w:spacing` (`before`/`after`/`line`/`lineRule`) and `w:ind`
(`left`/`right`/`firstLine`/`hanging`) are arguably per-attribute in the same
way. They were left as whole-element comparisons because no measured
over-firing traced to them, and because widening the rule without evidence
risks the opposite error — silently *not* marking a real spacing change.

## S-D6 — dropping unresolvable style references: already done where it fires

The brief asked for style references the engine cannot resolve to be dropped
rather than emitted. The document side is already implemented —
`crate::comparer::footnotes::strip_unresolved_style_refs`, called from
`compare_documents_internal`, removes `w:pStyle`/`w:rStyle` whose `@w:val` no
`w:style` defines.

The stylesheet side (`w:basedOn` / `w:link` / `w:next` pointing at an undefined
id, which would silently truncate chain resolution) was **measured and found not
to occur**: zero dangling references across the 108 source stylesheets in
`tests/corpus/broken_ones_two/sources`, and zero across 150 generated outputs.
`canonicalize_style_ids` already rewrites those three attributes alongside the
ids it renames. Implementing a dropper for it would be dead code; the cycle and
depth guards in `effective_style_props` already make an unresolvable `basedOn`
harmless (the chain just stops).

Revisit if a corpus ever shows a non-zero count.

## S-D7 — we now mark more styles than Word does

On the 828 colliding styles measured across 135 pairs, style-level change
markup: **ours 766, oracle 571**. Aggregate over all styles: ours 1163, oracle
871.

Part of the excess is not from S at all — `cascade_normal_change_to_based_styles`
(M111) stamps Normal's change onto every `basedOn=Normal` paragraph style that
lacks change markup, and it was already firing 807 times before this work. Now
that S records the *real* A→B difference where one exists, M111's fabricated
cascade (whose "old" value is Normal's `docDefaults` rPr, not the style's own
previous definition) is a weaker fallback than it was.

Worth a follow-up: gate M111 to the styles S did not touch *and* whose effective
formatting genuinely changed only via Normal. Not done here because M111 is
pinned by four passing oracle tests (`m111_*`) and unpicking it needs its own
evidence pass.

## S-D8 — pre-existing OOXML invalidity is unchanged, and large

Validating 150 generated outputs with `tools/validate-docx`:
**55 invalid before this change, the same 55 after.** No document became invalid
and none became valid.

The causes are inherited from malformed sources, not from the comparer:
`paragraphProperties="[object Object]"` and `rsidRDefault` as undeclared
attributes (a producer writing JS object stringification into OOXML), math
`m:sepChr` in the wrong content model, `w:ilvl` outside `w:numPr`, enum
attributes carrying `"false"`. Several corpus sources are themselves invalid.

This is a standing bench-quality finding, out of scope for S, and it should not
be read as a regression from any recent engine work.

# Deferred notes — Word-validity sweep (branch `feat/numpr-child-order`)

Findings from the OOXML-validity sweep that produced the two fixes on this
branch. Recorded rather than implemented, per the stage scope rule.

Measurement base: 504 jubarte-rust outputs (the 197-document `[40,60)` score
cluster plus the 307-document `>=90` control) from the `neurotic_docx_bench`
corpus, validated with `tools/validate-docx`, each compared against the Word
oracle for the same pair and against both source documents.

**Provenance — stamped, because it caught me out once.** These outputs were
generated by the engine at **`1be1fcd`** (workstream S), *not* by
`jubarte-rust@fcea02da49f4`, the build that produced the published 76.2072
baseline run `019fcc5d`. The bench's `dist/` was rebuilt from `1be1fcd` at
17:31:25 and I generated at 17:32:05; a `resolve_local_version` control I ran
*before* the rebuild returned `fcea02da49f4` and I generated *after* it, so the
artifact mutated between the check and the use. Any figure here that reads
`word/styles.xml` therefore describes `1be1fcd`, not the published baseline.

Figures that read **`word/document.xml` only** are unaffected, and that is not
an assumption: `document.xml` is byte-identical between the pre-style build
(`d931a10`) and `1be1fcd` on **197/197** cluster documents, and `stage2-measure`
independently byte-diffed parent against candidate across the corpus — 256 of
803 pairs differ, with `word/styles.xml` the sole changed part in every one.
The `numPr` counts, the in-place/volume/paragraph-count structure and the
body-text identity results all read `document.xml` and stand.

## V-D1 — the validity baseline, and how much of it is ours

Engine `1be1fcd` (see the provenance note above); the Word-oracle and source
figures are engine-independent.

| | documents |
|---|---:|
| our output invalid | 157/504 (31.2%) |
| **Word's own comparison output invalid** | **49/504 (9.7%)** |
| at least one SOURCE document invalid | 226/504 (44.8%) |
| our output invalid while BOTH sources are clean | **55/504 (10.9%)** |

Two things worth keeping in view. Word itself emits schema-invalid comparison
output on ~10% of this corpus, so "the validator is clean" is a stricter bar
than Word-parity, and the two can conflict. And 45% of the sources are already
invalid, so a raw invalid-output count mostly measures the corpus, not the
engine. The 10.9% figure — invalid output from clean inputs — is the number
that isolates us, and it is the one to track.

## V-D2 — residual classes after this branch's two fixes

Measured base-vs-fixed on the same commit, 197 cluster documents:
invalid 103 → 93; `w:author`/`w:date` 55 → 0; ordering 287 → 244; every other
class unchanged; **0 documents gained an error**.

What remains, largest first:

| class | count | notes |
|---|---:|---|
| `invalid child element … w:highlight` | 202 | **mostly not ours** — of the 14 cluster documents carrying it, 12 have a source that already does; only 2 come from clean inputs, both from the `super_editor__text_color_highlight_36cb4c90` fixture. Big count, small cause: do not size work off the 202. |
| `unexpected child element` (residual) | 244 | after `numPr`; the named children are `spacing`, `uiPriority`, `pStyle`, `link` |
| `required attribute 'val' is missing` | 50 | |
| `rsidRDefault` / `paragraphProperties` / `ID` / `path` not declared | 48 | attribute-name defects |
| `w16cid:durableId`, `w15:restartNumberingAfterBreak` not declared | 26 | **probably NOT defects** — these are legitimate Word 2012/2016 extension attributes; the validator's schema does not carry them. Confirm before "fixing". |

The `uiPriority` / `pStyle` / `link` residue is `w:style` child-order, which
`feat/style-chain-resolution` addresses with its `STYLE_CHILD_ORDER` table. It
should be re-measured after that branch lands rather than fixed twice.

## V-D3 — `wml_order_elements_per_standard` covers only 7 containers

`finalize.rs` sorts the children of `pPr`, `rPr`, `tblPr`, `tcPr`, `tcBorders`,
`tblBorders`, `pBdr`. It is a port of PowerTools'
`WmlOrderElementsPerStandard`, and inherits that function's container list, so
any other ordered `CT_*` in the schema is unprotected. `numPr` was found by
hand; this branch adds it and wires it into `tests/schema_consistency.rs`.

The systematic version of this — enumerating every ordered complex type in
`tests/data/wml_main_schema.json` and asserting the emitted order against it —
was not attempted here. It is the difference between fixing the ordering bugs
we tripped over and being unable to ship one.

## V-D4 — Stage R2 of `plans/jubarte-rust-to-target.md` is built on a false premise

Not a validity item, recorded here because it is the reason this branch exists.

Stage R2 prescribes emitting in-place intra-paragraph revisions instead of
paragraph-granular replacement, transferring a defect diagnosed on
`jubarte-final-lossless`. rust does not have that defect:

| | rust cluster (197) | rust >=90 (307) | lossless BOTH_HOLD (124) |
|---|---:|---:|---:|
| in-place paragraphs, candidate vs oracle median | **1 vs 1** | 2 vs 2 | **0 vs 1** |
| candidate has NONE where the oracle has some | 25 (12.7%) | 6 (2.0%) | 69 (55.6%) |

And the deficit does not predict the score inside the cluster — 47.94 / 50.23 /
51.04 / 51.85 across every level of it. 53 of 197 match the oracle on in-place
count, paragraph count and both character volumes and still score 52.25; 46 have
body text byte-identical to Word's and still score 52.29.

The cluster's cause is **accumulated layout drift**, not markup shape. What
drives the drift is still open, and style collision is *not* the answer even
though the correlation is strong (the two sources define a shared `styleId`
differently in 60.3% of the cluster against 15.7% of the control): workstream S
resolves that collision and removes only ~15% of the residual effective-spacing
divergence, and measured on the full 763 it is worth +0.50 mean / +1.02 median.
Real, small, and it leaves the cluster standing.

I recorded the opposite of this for a while — see the provenance note above for
how, and `plans/reviews/what-the-50-cluster-actually-is.md` for the current
leading mechanism (list paragraphs resolving to a different format than Word,
2.1x enriched in the cluster, with a rendered causal chain rather than an
inferred one).

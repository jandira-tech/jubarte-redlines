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

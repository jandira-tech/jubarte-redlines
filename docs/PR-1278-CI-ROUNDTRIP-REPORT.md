<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# PR #1278: CI failures and Markdown round-trip losses

Assessment date: 2026-10-20. Initial head:
`9a0423c22ce6e195c32b886a3f45ccc48087d1dc`. During this investigation the
branch advanced to `6a34e1c498bcf403ed98a443ac11a9fced70892f`; its two new
commits change generated READMEs and adoption fixtures/plans, not the engine.
The failure inventory below comes from the complete
[Markdown job log](https://github.com/jandira-tech/jubarte-redlines/actions/runs/39477423894/job/118816887357).
There are 18 failing cases, not 18 independently diagnosed engine bugs.

## First import and generated IDs

IDs being generated on the first import of ordinary Markdown is expected.
The annotated view also adds YAML metadata, paragraph markers, normalized
line wrapping and style descriptions. Those additions alone do not show
that document content was lost.

Five cases compare an ordinary manuscript directly against that annotated
view: `docx-libraries-comparison.md`, `dolphins.md`, `eLife.00047.md`,
`git_manuscript.md`, and `list_of_figures.md`. The log reports mostly new IDs
and lost unannotated blocks. For example, `docx-libraries-comparison.md`
reports 138 new IDs and one lost block. This comparison does **not** establish
that the manuscript's text, images or formatting survived: those need their
own assertions.

Suggested test contract:

1. Ordinary Markdown → DOCX: assert the imported text, links, images, tables,
   headings and formatting, using the original manuscript as the oracle.
2. That DOCX → annotated Markdown → DOCX → annotated Markdown: require the
   IDs and annotated representation to remain stable.
3. Inputs already in the annotated dialect: retain the current exact
   comparison from the first reconstruction.
4. Retain the independent package/render checks; a stable Markdown view can
   hide a loss that happened on the first export.

`tests/md_fixed_point.rs::markdown_case` currently applies the same exact
comparison to manuscripts and annotated fixtures. The `HAND_WRITTEN` list
includes both kinds. This patch does not change that policy, regenerate the
manuscripts into a different format, add exemptions, or relax the gate.

## Inventory of the other 13 failing cases

Paths in this table are relative to `tests/fixtures/md-roundtrip/` unless a
different location is stated. `views.txt` maps the `ids-*.md` fixtures to
their original DOCX packages. Several cases have more than one symptom.

| Cases | Observed failure | Diagnosis and repair scope |
| --- | --- | --- |
| `edge-table-styles.md`; `edges/edge-table-styles.docx` | Legend changes from `TableNormal, Ledger, borders vary` to `TableNormal, Ledger`. | `from_docx/header.rs::table_style_line` reduces unequal side borders to `borders vary`. `view_styles.rs::table_style_entry` accepts that phrase but stores only an optional uniform border width and name. `package.rs::with_table_styles` therefore cannot reconstruct the differing sides. Requires a lossless border representation, not just retaining the phrase in the output. |
| `ids-fonts.md` | The gate reports unequal table summaries, although Markdown IDs and text are exact. | XML inspection shows **both** packages have six single 0.5pt table borders. The source summary also contains cell shading, which the dialect intentionally does not preserve. `debug.rs::clip_line`/`clip_from` abbreviate that longer human-readable line before `docx_verdicts` removes `cells-shd`; the remaining strings therefore differ in their clipped prefix/text sample. This is not evidence that a new table border appeared. Compare unabridged structured render snapshots before filtering out-of-dialect properties; do not alter the border defaults to fix this diagnostic artifact. Source style `9` also loses its cell-margin definition, and auto table width becomes 100% width; those package differences need separate layout assessment. |
| `ids-header-footer.md` | Six paragraph frames disappear: three footers and three headers, with `hAnchor=margin`, `vAnchor=text`, `wrap=none`, `y=1`, and right alignment on the headers. | The source has `w:framePr`. `debug/render.rs` resolves direct and inherited frame attributes; the Markdown story/style representation has no corresponding frame serialization. Rebuilding the story drops this positioning while retaining its text. Needs a representation for frame properties, including inheritance. |
| `received.md`; `tests/fixtures/agent-view/received.docx` | Two footer frames disappear, including right alignment. Markdown remains exact. | Same missing `w:framePr` representation as the preceding row. An exact text view is insufficient to detect this loss. |
| `ids-paragraphs.md` | CI's package/render comparison changes 107 pages to 96; Markdown is exact. | `debug/render.rs::layout_lines` runs the renderer; `docx_verdicts` compares its page count. The view omits package-level layout inputs, including some font/theme/default-style details. The log proves repagination, but does not isolate a single property causing all 11 pages of difference. Needs a controlled comparison of section properties, effective styles and font resolution, followed by Word checks. |
| `ids-sections.md` | 2 pages → 3; Markdown is exact. | Same renderer-level symptom. Section setup and break properties are the first diagnostic targets; a differing count alone does not identify a faulty section property. Do not force Word's cached page count into the reconstructed package. |
| `ids-table-changes.md` | 7 pages → 5; Markdown is exact. | Repagination remains unexplained at a single-property level. Compare table/row geometry, paragraph spacing and fonts against the source package. |
| `ids-tables.md` | 11 pages → 13; Markdown is exact. | Same unresolved pagination category. This is evidence of a layout change, not proof of one specific table-height bug. |
| `ids-spacing-threads.md` | Revision count falls from 67 to 65; repeated IDs change; adjacent `r43` and `r55` carriers merge. | Local replay confirms separate carriers around untracked formatted text/hyperlinks receive new IDs (`r6` → `r64`, `r42` → `r65` on their later appearances), while adjacent same-ID carriers join. `write.rs::name_revision`/`tag_serial`, revision-ID allocation and run grouping are the relevant paths. This is not ordinary first-import ID allocation: the fixture supplies revision identities already. Requires preserving carrier boundaries and deciding repeated-native-ID identity consistently with the ID contract. No text loss was established by these particular deltas. |
| `tests/fixtures/strict/Strict01.docx` | A drawing payload changes, `No Spacing` becomes an unresolved `NoSpacing` style name, and CI repaginates 13 → 12. | The drawing loss is diagnosed separately below. XML inspection also shows the source's `NoSpacing` style definition (including spacing, linked character style and theme font) is absent after reconstruction; a payload's style reference alone does not carry that definition. `debug/render.rs` falls back to the ID as its display name. Preserving drawing XML does not by itself restore the missing style or prove that pagination is fixed. |
| `tests/corpus/neurotic_docx_bench/corpus/word_based/docx_source/docx_lots_of_comments_addition_removal.docx` | Table 6's `band1Vert` conditional shading changes from `theme:background1` to `theme:accent1`; 11 pages → 13. Markdown remains exact. | `package.rs::with_table_styles` substitutes the built-in Word table-style template when the ID is recognized. That loses this document's customization of `MediumList2-Accent1`; the legend does not serialize its conditional shading. Preserve document-specific style properties instead of assuming that a built-in ID guarantees template-equivalent content. The shading mismatch is diagnosed; its share of the page-count difference is not established. |

## Contained repair: blank text boxes

Local replay of `Strict01.docx` isolates drawing `d14`. Its text box contains
two content controls (Title and Subtitle), placeholder/binding properties,
paragraph/run formatting and whitespace-only text. The exported payload
contains empty text slots with lengths `[5, 5]`, but its text hash is the hash
of the empty string: `mermaid_hash` trims whitespace before hashing.

On reconstruction, `fill_boxes` has no visible words with which to refill
those five-byte slots. Refill fails, so it substitutes the newly parsed blank
story, `<w:p/>`. The controls and their formatting/binding structure vanish.
Decoding the original and reconstructed `drawing:` payloads confirms this
specific structural loss; it is not merely a change in base64 encoding.

The contained repair preserves a blank text box's original structure when
its visible story is unchanged, while still replacing that story when the
user adds text. This protects newly exported views; it cannot recover
structure already discarded in an earlier reconstructed document. This avoids changing the dialect or normalizing away a
failure. It does not restore external style definitions referenced by the
box. The regression test exercises unchanged whitespace-only control
structure and an explicit subsequent text edit.

## Other attached CI failures

- Generated READMEs and the release-script tests: repaired upstream during
  this session by `ec3a926d` and included by fast-forwarding the PR branch.
  `convert-sweep unit` itself passed its conversion tests; its job failed
  later in `test_release_sh.py` because `test_library_readmes.py` found stale
  generated output.
- Adoption examples: `6a34e1c4` regenerates the inputs and pins the plans to
  their current hashes; the comment plan also uses `p9` instead of the
  rejected `body:p:9`. Local isolated replays of examples 09, 12 and 13 pass,
  and 13 still refuses the ambiguous plan with `AMBIGUOUS_ANCHOR` and exit 3.
- The latest `convert-sweep unit` run gets past those script gates and then
  fails two Node CLI tests. The committed WASM package is stamped
  `edec90334de92f5b1d2c9eee8b5caa2d568e3da9`: it returns snapshot schema 1
  and older revision markup, while the current source returns schema 2.
  Rebuild the generated WASM package from source; do not lower the schema
  assertion or patch generated JavaScript/WASM by hand. API/CLI documentation
  must then be regenerated from the refreshed artifacts as well.
- Both Jules failures are the same service-side session-creation rejection:
  `POST https://jules.googleapis.com/v1alpha/sessions` returns
  `400 FAILED_PRECONDITION`, in both agentic and fallback modes. The
  [new head's review run](https://github.com/jandira-tech/jubarte-redlines/actions/runs/39480212203/job/118826498501)
  reproduces it. A GitHub repository secret named `JULES_API_KEY` exists;
  there is no local Jules credential or connected Jules diagnostic tool.
  The response does not identify whether account entitlement, quota or
  repository authorization is the failed prerequisite. An owner must inspect
  the Jules account used by that secret and restore session creation, or
  provide the account-side diagnostic reason. Do not disable the check or
  mislabel it as a passing review.

## Reproduce the diagnostics

Run from the repository root using a binary built from the assessed source.
Keep the Markdown away from its named source document so automatic source
anchoring cannot make a reconstruction look lossless by reusing the original.

```sh
cargo build --bin jubarte
work=$(mktemp -d)
target/debug/jubarte read tests/fixtures/strict/Strict01.docx \
  --no-page-markers > "$work/input.md"
target/debug/jubarte convert "$work/input.md" -o "$work/back.docx" \
  -d 1970-01-01T00:00:00Z
target/debug/jubarte read "$work/back.docx" --no-page-markers > "$work/back.md"
target/debug/jubarte debug tests/fixtures/strict/Strict01.docx \
  "$work/back.docx" --check render
```

For annotated Markdown fixtures, copy the fixture to `$work/input.md`
instead of producing it with `read`, and use the original DOCX listed in
`views.txt` for the `debug` comparison. Compare decoded drawing JSON/XML,
not just its base64 string. For a narrow gate replay:

```sh
MD_FIXED_POINT_ONLY=Strict01 cargo nextest run --all-features \
  --test md_fixed_point -E 'test(markdown_and_docx_round_trip_exactly)'
```

The full Markdown gate remains strict and is expected to remain red after
the contained blank-box repair. The remaining losses are recorded here;
they have not been declared fixed by text stability, passing unit tests,
or a changed cached page count. No new Word automation was performed for
this report, so the package/render diagnostics are not a Word-parity claim.

## Validation of the source repair

- The blank-control regression failed before the repair: the actual output
  was a box containing only `<w:p/>` instead of the original control tree.
  After the repair, all six drawing unit tests and all 44 selected text-box
  integration tests pass under nextest with LLVM branch coverage.
  A CLI replay of the real `Strict01.docx` now produces an exactly equal
  decoded `d14` drawing payload. Its 13 → 12 pagination difference and
  missing `NoSpacing` style definition remain, as reported above.
- The 22 focused grading/outcome tests pass. The redundant comparison fix
  preserves the existing `n > 2` threshold.
- The three relevant script suites pass: 148 tests, two platform-specific
  skips. `convert_sweep.py` coverage is 91.82% lines / 78.95% branches;
  `library_readmes.py` is 94.71% / 88.10%.
- Before the Rust repair, the broader pilot suite passed 151 tests and failed
  `test_validate_returns_findings_on_tracked_change_origin`. That test gives
  `validate` the **same** document as baseline and candidate, then expects a
  foreign-history finding. Current auditing intentionally accepts unchanged
  foreign history. This unrelated fixture/expectation is not changed here.
  That run achieved 93.27% line / 86.65% branch coverage; `harness/grading.py`
  achieved 98.56% / 97.62%.
- The focused Rust runs cover 24.43% of the whole engine's lines and 16.30%
  of its branches; `from_docx/drawing.rs` is 40.12% / 18.34%. These are narrow
  test selections, not a claim about full-suite coverage or release readiness.

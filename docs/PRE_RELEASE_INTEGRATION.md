<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# Pre-release integration, 2026-10-08

Branch: `c/pre-release`, based on `origin/main` at `344fd848`.
The user requested consolidation without waiting for CodeRabbit.

## Included work

| Source | Head at discovery | Integration |
| --- | --- | --- |
| PR #380, shared CLI and document diff views | `1f1fbc27` | Merge, retain the complete native/Python/WASM APIs and regression tests. |
| PR #374, PDF comment placement and changed pages | `c66f0575` | Merge; move page options into the shared clap model introduced by #380. |
| PR #377, Python/WASM PDF options | `8dcc506b` | Merge after #374; preserve shared parser and regenerate all WASM packages from source. |
| PR #375, Word number formats and list labels | `25ed0d26` | Merge, retain bounded formats and last-switch behavior. |
| `wip/own-props-guard` | `2a13694c` | Merge the style inheritance guard and its regression test. |
| `feat/cli-text-diff-native`, `feat/cli-text-diff-adapters` | `7cbf03d5`, `4bcd3522` | Superseded development histories recorded with ours merges; #380 retains and evolves their implementations and tests. |
| `feat/diff-views`, `feat/diff-views-adapters` | `2f7fc096`, `7fd5103e` | Superseded histories recorded with ours merges; #380 corrects accepted unchanged output, escaping, and whole-clause line addresses. |

The security branch `c/address-openssf-findings` and the remaining source branches
were already ancestors of main. The retired `arthrod-legacy` remote is historical.
`gh-pages` is independent website deployment history and stays separate.
`arthrod-patch-1` deletes maintained README sections and restores outdated release
wording. Its ancestry is recorded with an ours merge, preserving current docs.
No source branch is deleted.

## Conflict decisions and review fixes

Retain the shared clap parser in Rust, Python and npm. Flatten `move_comments` and
`changed_only` into host JSON, reject them for text views, and preserve both options
for PDF conversion. Generated reference text and WASM binaries must be rebuilt,
rather than choosing one branch's generated output.

Existing review records were assessed individually: 89 records across the four PRs,
plus the embedded tempfile nitpick. The complete gate ledger follows below.
Three outstanding PDF claims receive deterministic tests before fixes: synthetic
comment markers must not enter field sources, listing measurement and paint must
use the same script fallback, and long comment author headings must wrap. The CLI
integration test fails before adding text-output rejection and passes afterward.

## Validation

Validation results are recorded after the integrated suite completes.
The full run exposed the WIP style regression: Normal cascade recreated the
inherited-only BodyText history. Eligibility is now frozen from the original and
revised style trees, live metrics are normalized, and only new redundant records
are removed. The regression also verifies accepted Times New Roman inheritance
and rejected Arial inheritance; inbound history and identical-declaration cascades
remain protected.
The five deterministic PDF review regressions and shared CLI page-option regression
passed after their red runs. The same-style RTL fixture uses injected bundled font
bytes to avoid installed font lookups.


## Per-comment gate: PRs 374, 375, 377 and 380

Source audit and authorized local fixes, 2026-10-08. This reviewer edited only src/convert/mod.rs and tests/convert_comment_placement.rs after parent confirmed failing coverage regressions. Parent owns all Cargo commands; no Cargo or GitHub posts were performed by this reviewer. Source inspected in integrated working tree and PR375 commit 25ed0d26; line numbers can move during integration. Parent confirmed the first three new regressions pass under branch coverage; two additional audit regressions also passed after their red runs.

Fetched records: PR374 25 inline + 13 top-level + 16 reviews = 54; PR375 10 inline + 10 top-level + 9 reviews = 29. Total 83.

Reconciliation: implement 13 + push back 45 + duplicate-linked 25 = 83. Implement means claim passed; evidence distinguishes already implemented from OPEN.

| PR | Kind | ID | Author | Verdict | Evidence / state |
|---|---|---|---|---|---|
| 374 | inline | 4200517588 | coderabbitai[bot] | implement | Already implemented: mark_comment_anchors explicitly clears style.effect_skip; regression a_marker_after_an_unpainted_effect_run_still_carries_its_comment in tests/convert_comment_placement.rs:433. |
| 374 | inline | 4200517599 | coderabbitai[bot] | implement | Already implemented: paint_stretched places original run comments over start..x after painting pieces; a_comment_inside_a_justified_line_keeps_its_balloon_and_its_listing checks both placements. |
| 374 | inline | 4200534834 | sourcery-ai[bot] | push back | mark_comment_anchors intentionally shows an independently anchored reply when all attached notes are replies; hiding it removes the visible anchor while retaining a separately paged listing. Co-anchored replies are folded; end_placement_folds_a_reply_into_its_threads_marker pins usual Word shape. Existing human reply 4200600455 documents the decision. |
| 374 | inline | 4200534849 | sourcery-ai[bot] | implement | Already implemented: wrap_words passes tokens through break_word; an_overlong_word_in_a_comment_is_broken_to_the_line pins 400-character preservation. |
| 374 | inline | 4200534861 | sourcery-ai[bot] | push back | new_page calls close_rev_bar before pages.push, and column_break closes before advancing columns. Each currently painted page is correctly marked; an_insertion_running_over_a_page_break_keeps_both_pages pins spanning insertions. The stored start-page tuple selects starting top versus col_top, not the sole page to retain. |
| 374 | inline | 4200534869 | sourcery-ai[bot] | implement | Already implemented: RenderRequest::pages and RenderReport::page_count document selectable output pages after filtering/listing; layout still covers document. |
| 374 | inline | 4200534877 | sourcery-ai[bot] | implement | Already implemented migration documentation in CHANGELOG.md:39–40 names both new fields and ..PdfOptions::default(). API is pre-1.0; preserve intended feature addition rather than applying non_exhaustive which would prohibit existing downstream struct-update construction. |
| 374 | inline | 4200534882 | sourcery-ai[bot] | implement | Already implemented: list_comments_at_end sorts label_order rather than geometry; number_comments generates body order labels. comments_are_listed_in_document_order_across_columns pins column order. |
| 374 | inline | 4200600061 | arthrod | duplicate-of-#4200517588 | Reply on canonical claim 4200517588; verified against source/test evidence in its ledger entry. Human implementation or push-back report agrees with current source; provenance only. |
| 374 | inline | 4200600243 | arthrod | duplicate-of-#4200517599 | Reply on canonical claim 4200517599; verified against source/test evidence in its ledger entry. Human implementation or push-back report agrees with current source; provenance only. |
| 374 | inline | 4200600455 | arthrod | duplicate-of-#4200534834 | Reply on canonical claim 4200534834; verified against source/test evidence in its ledger entry. Human implementation or push-back report agrees with current source; provenance only. |
| 374 | inline | 4200600641 | arthrod | duplicate-of-#4200534849 | Reply on canonical claim 4200534849; verified against source/test evidence in its ledger entry. Human implementation or push-back report agrees with current source; provenance only. |
| 374 | inline | 4200600774 | arthrod | duplicate-of-#4200534861 | Reply on canonical claim 4200534861; verified against source/test evidence in its ledger entry. Human implementation or push-back report agrees with current source; provenance only. |
| 374 | inline | 4200600939 | arthrod | duplicate-of-#4200534869 | Reply on canonical claim 4200534869; verified against source/test evidence in its ledger entry. Human implementation or push-back report agrees with current source; provenance only. |
| 374 | inline | 4200601158 | arthrod | duplicate-of-#4200534877 | Reply on canonical claim 4200534877; verified against source/test evidence in its ledger entry. Human implementation or push-back report agrees with current source; provenance only. |
| 374 | inline | 4200601340 | arthrod | duplicate-of-#4200534882 | Reply on canonical claim 4200534882; verified against source/test evidence in its ledger entry. Human implementation or push-back report agrees with current source; provenance only. |
| 374 | inline | 4200602404 | coderabbitai[bot] | duplicate-of-#4200517588 | Reply on canonical claim 4200517588; verified against source/test evidence in its ledger entry. Bot confirms/withdraws; no independent change requested. |
| 374 | inline | 4200605778 | coderabbitai[bot] | duplicate-of-#4200517599 | Reply on canonical claim 4200517599; verified against source/test evidence in its ledger entry. Bot confirms/withdraws; no independent change requested. |
| 374 | inline | 4200649770 | sourcery-ai[bot] | implement | Already implemented: with_options uses Restore Drop guard for REVISIONS and PAGE_PLAN; a_panicking_conversion_restores_the_options pins catch_unwind restoration. |
| 374 | inline | 4200649785 | sourcery-ai[bot] | implement | Already implemented: PageOutOfRange page_count docs describe selectable output; Display says output has N pages. png_page_bounds_use_the_output_count_after_filtering_and_listing pins message/count. |
| 374 | inline | 4201856280 | arthrod | duplicate-of-#4200649770 | Reply on canonical claim 4200649770; verified against source/test evidence in its ledger entry. Human implementation or push-back report agrees with current source; provenance only. |
| 374 | inline | 4201856412 | arthrod | duplicate-of-#4200649785 | Reply on canonical claim 4200649785; verified against source/test evidence in its ledger entry. Human implementation or push-back report agrees with current source; provenance only. |
| 374 | inline | 4216554951 | coderabbitai[bot] | implement | Fixed in the integrated source: explicit comment_marker flag excludes synthetic text from run_word_count, document_bookmark_texts and note_style_hit; justified paint pieces preserve it. synthetic_comment_markers_do_not_enter_field_sources passed parent coverage green. Both wrapping merge guards now preserve classification; wrapping_keeps_synthetic_marker_classification_separate failed for the correct merge assertion before these follow-up edits and awaits parent green. |
| 374 | inline | 4216554960 | coderabbitai[bot] | implement | Fixed in the integrated source: listing_faces uses existing ink_face fallback; listing_width and paint_listing_line share its segmentation. comment_listing_uses_loaded_script_fallback_for_paint_and_width passed parent coverage green with deterministic document-local embedded font fixture. Follow-up coalesces same-style fragments and keeps neutral spaces with the script face; a_script_listing_phrase_keeps_spaces_and_word_order_when_shaping failed on neutral-space segmentation before this fix and awaits parent green. Measurements use coalesced strings exactly as painted; glyph coverage of installed CJK fonts is integration-only. |
| 374 | inline | 4216554966 | coderabbitai[bot] | implement | Fixed in the integrated source: styled labels/author attribution wrap with wrap_listing_runs, shared fallback measurements, full heading fit accounting and overflow pagination. long_comment_attributions_wrap_and_preserve_printable_page_bounds passed parent coverage green; preserves full author and page label. |
| 374 | comments | 6025533537 | chatgpt-codex-connector[bot] | push back | Provider billing/usage limitation only; no code finding. Account purchase/admin changes are outside merge scope; proceed with local evidence. |
| 374 | comments | 6025533919 | qodo-code-review[bot] | push back | Provider billing/usage limitation only; no code finding. Account purchase/admin changes are outside merge scope; proceed with local evidence. |
| 374 | comments | 6025533964 | chatgpt-codex-connector[bot] | push back | Provider billing/usage limitation only; no code finding. Account purchase/admin changes are outside merge scope; proceed with local evidence. |
| 374 | comments | 6025534577 | sourcery-ai[bot] | push back | Full generated walkthrough/guide inspected; descriptions and optional UI controls introduce no separate concrete defect. Supported claims agree with source gated above. PR375 bot-rate-limit notice is overridden by user instruction to proceed without waiting. |
| 374 | comments | 6025535106 | coderabbitai[bot] | push back | Full walkthrough inspected. Docstring metric 68.63% supplies no missing-symbol list or reproducible command; no repository docstring threshold is established. Public changed options/report/error contracts and new helpers have documentation. Metric is not proof of missing API docs. Merge-conflict notice is addressed by integration source reconciliation, not a code claim. Generated unit-test UI actions are optional, not findings. |
| 374 | comments | 6025539591 | arthrod | push back | Review trigger command only; no technical claim or implementation request. User explicitly says do not wait for CodeRabbit, so no bot-wait gate is introduced. |
| 374 | comments | 6025541675 | coderabbitai[bot] | push back | Bot operational status/command acknowledgement or test-generation progress only; full body contains no separate source defect. User instruction removes waiting requirement. |
| 374 | comments | 6025764635 | arthrod | push back | Review trigger command only; no technical claim or implementation request. User explicitly says do not wait for CodeRabbit, so no bot-wait gate is introduced. |
| 374 | comments | 6025764892 | arthrod | push back | Review trigger command only; no technical claim or implementation request. User explicitly says do not wait for CodeRabbit, so no bot-wait gate is introduced. |
| 374 | comments | 6025766535 | coderabbitai[bot] | push back | Bot operational status/command acknowledgement or test-generation progress only; full body contains no separate source defect. User instruction removes waiting requirement. |
| 374 | comments | 6026753597 | coderabbitai[bot] | push back | Bot operational status/command acknowledgement or test-generation progress only; full body contains no separate source defect. User instruction removes waiting requirement. |
| 374 | comments | 6026754603 | coderabbitai[bot] | push back | Bot operational status/command acknowledgement or test-generation progress only; full body contains no separate source defect. User instruction removes waiting requirement. |
| 374 | comments | 6027507409 | codecov[bot] | push back | Codecov reports 99.24812% patch coverage and 2 unnamed uncovered lines; this is historical provider output, not a reproducible behavior defect or a requirement for 100%. Integration must run fresh coverage; do not claim historical report as current validation. |
| 374 | reviews | 5434663064 | coderabbitai[bot] | duplicate-of-#4200517588 | Formal review repeats effect_skip and justified metadata claims: canonical 4200517588 and 4200517599 individually gated above. |
| 374 | reviews | 5434682673 | sourcery-ai[bot] | duplicate-of-#4200534834 | Formal review repeats all six Sourcery findings: canonical 4200534834, 4200534849, 4200534861, 4200534869, 4200534877, 4200534882 individually gated above; no extra finding in full body. |
| 374 | reviews | 5434755206 | arthrod | push back | Empty formal review body (COMMENTED); no factual assertion or code request. Review metadata is not an independent defect. |
| 374 | reviews | 5434755398 | arthrod | push back | Empty formal review body (COMMENTED); no factual assertion or code request. Review metadata is not an independent defect. |
| 374 | reviews | 5434755615 | arthrod | push back | Empty formal review body (COMMENTED); no factual assertion or code request. Review metadata is not an independent defect. |
| 374 | reviews | 5434755830 | arthrod | push back | Empty formal review body (COMMENTED); no factual assertion or code request. Review metadata is not an independent defect. |
| 374 | reviews | 5434756003 | arthrod | push back | Empty formal review body (COMMENTED); no factual assertion or code request. Review metadata is not an independent defect. |
| 374 | reviews | 5434756196 | arthrod | push back | Empty formal review body (COMMENTED); no factual assertion or code request. Review metadata is not an independent defect. |
| 374 | reviews | 5434756396 | arthrod | push back | Empty formal review body (COMMENTED); no factual assertion or code request. Review metadata is not an independent defect. |
| 374 | reviews | 5434756612 | arthrod | push back | Empty formal review body (COMMENTED); no factual assertion or code request. Review metadata is not an independent defect. |
| 374 | reviews | 5434757840 | coderabbitai[bot] | push back | Empty formal review body (COMMENTED); no factual assertion or code request. Review metadata is not an independent defect. |
| 374 | reviews | 5434761428 | coderabbitai[bot] | push back | Empty formal review body (COMMENTED); no factual assertion or code request. Review metadata is not an independent defect. |
| 374 | reviews | 5434810505 | sourcery-ai[bot] | duplicate-of-#4200649770 | Formal review repeats panic restoration and range documentation findings: canonical 4200649770 and 4200649785 individually gated above. |
| 374 | reviews | 5436190869 | arthrod | push back | Empty formal review body (COMMENTED); no factual assertion or code request. Review metadata is not an independent defect. |
| 374 | reviews | 5436191050 | arthrod | push back | Empty formal review body (COMMENTED); no factual assertion or code request. Review metadata is not an independent defect. |
| 374 | reviews | 5453648416 | coderabbitai[bot] | duplicate-of-#4216554951 | Formal review repeats synthetic fields/fallback/heading findings 4216554951, 4216554960, 4216554966 individually gated above. Its extra tempfile nitpick receives separate subfinding row below, so it is not lost. |
| 375 | inline | 4201352466 | coderabbitai[bot] | implement | Already implemented in 25ed0d26 and integrated current tree: ALPHA_LABEL_MAX=780, alpha_label applies modulo before repeat_n (<=30 chars), ROMAN_LABEL_MAX=32767 modulo bounds Roman output. NumberFormat::write rejects oversized fields with UNREPRESENTABLE. list_letters_and_roman_numerals_wrap_as_word_does and number_formats_write_roman_and_letters pin boundaries/u32::MAX. |
| 375 | inline | 4201352472 | coderabbitai[bot] | push back | PAGEREF missing-bookmark precedence deliberately matches Word probe bn1006 documented in reply 4201440368. src/fields.rs::pageref checks defined bookmark before format_page. tests/fields_update.rs::pagerefs_distinguish_undefined_bookmarks_from_unpaged_ones explicitly covers missing CardText. CodeRabbit withdraws claim in 4201442422. |
| 375 | inline | 4201358377 | sourcery-ai[bot] | duplicate-of-#4201352466 | Same allocation issue, including fields; canonical bounded implementation and tests cover both conversion and field write. |
| 375 | inline | 4201358386 | sourcery-ai[bot] | implement | Already implemented in 25ed0d26: number_format loops all switches, overwrites None with later supported format; last format wins, MERGEFORMAT/CHARFORMAT preserve it. the_last_format_switch_wins pins both supported-after-unsupported and reverse order. More precise than ignoring all unsupported switches: final unsupported preserves cache, per documented Word probe. |
| 375 | inline | 4201439986 | arthrod | duplicate-of-#4201352466 | Reply on canonical claim 4201352466; verified against source/test evidence in its ledger entry. Human implementation or push-back report agrees with current source; provenance only. |
| 375 | inline | 4201440108 | arthrod | duplicate-of-#4201358377 | Reply on canonical claim 4201358377; verified against source/test evidence in its ledger entry. Human implementation or push-back report agrees with current source; provenance only. |
| 375 | inline | 4201440235 | arthrod | duplicate-of-#4201358386 | Reply on canonical claim 4201358386; verified against source/test evidence in its ledger entry. Human implementation or push-back report agrees with current source; provenance only. |
| 375 | inline | 4201440368 | arthrod | duplicate-of-#4201352472 | Reply on canonical claim 4201352472; verified against source/test evidence in its ledger entry. Human implementation or push-back report agrees with current source; provenance only. |
| 375 | inline | 4201442422 | coderabbitai[bot] | duplicate-of-#4201352472 | Reply on canonical claim 4201352472; verified against source/test evidence in its ledger entry. Bot confirms/withdraws; no independent change requested. |
| 375 | inline | 4201445052 | coderabbitai[bot] | duplicate-of-#4201352466 | Reply on canonical claim 4201352466; verified against source/test evidence in its ledger entry. Bot confirms/withdraws; no independent change requested. |
| 375 | comments | 6027199659 | chatgpt-codex-connector[bot] | push back | Provider billing/usage limitation only; no code finding. Account purchase/admin changes are outside merge scope; proceed with local evidence. |
| 375 | comments | 6027199756 | sourcery-ai[bot] | push back | Full generated walkthrough/guide inspected; descriptions and optional UI controls introduce no separate concrete defect. Supported claims agree with source gated above. PR375 bot-rate-limit notice is overridden by user instruction to proceed without waiting. |
| 375 | comments | 6027199880 | qodo-code-review[bot] | push back | Provider billing/usage limitation only; no code finding. Account purchase/admin changes are outside merge scope; proceed with local evidence. |
| 375 | comments | 6027200100 | chatgpt-codex-connector[bot] | push back | Provider billing/usage limitation only; no code finding. Account purchase/admin changes are outside merge scope; proceed with local evidence. |
| 375 | comments | 6027200276 | coderabbitai[bot] | push back | Full generated walkthrough/guide inspected; descriptions and optional UI controls introduce no separate concrete defect. Supported claims agree with source gated above. PR375 bot-rate-limit notice is overridden by user instruction to proceed without waiting. |
| 375 | comments | 6027201650 | arthrod | push back | Review trigger command only; no technical claim or implementation request. User explicitly says do not wait for CodeRabbit, so no bot-wait gate is introduced. |
| 375 | comments | 6027202949 | coderabbitai[bot] | push back | Bot operational status/command acknowledgement or test-generation progress only; full body contains no separate source defect. User instruction removes waiting requirement. |
| 375 | comments | 6027432097 | arthrod | push back | Review trigger command only; no technical claim or implementation request. User explicitly says do not wait for CodeRabbit, so no bot-wait gate is introduced. |
| 375 | comments | 6027432377 | arthrod | push back | Review trigger command only; no technical claim or implementation request. User explicitly says do not wait for CodeRabbit, so no bot-wait gate is introduced. |
| 375 | comments | 6027434014 | coderabbitai[bot] | push back | Bot operational status/command acknowledgement or test-generation progress only; full body contains no separate source defect. User instruction removes waiting requirement. |
| 375 | reviews | 5435581553 | coderabbitai[bot] | duplicate-of-#4201352466 | Formal review repeats bounded allocation 4201352466 and unsupported PAGEREF 4201352472; both individually gated above. |
| 375 | reviews | 5435588000 | sourcery-ai[bot] | duplicate-of-#4201358377 | Formal review repeats allocation 4201358377 and later switches 4201358386; both individually gated above. |
| 375 | reviews | 5435681666 | arthrod | push back | Empty formal review body (COMMENTED); no factual assertion or code request. Review metadata is not an independent defect. |
| 375 | reviews | 5435681888 | arthrod | push back | Empty formal review body (COMMENTED); no factual assertion or code request. Review metadata is not an independent defect. |
| 375 | reviews | 5435682063 | arthrod | push back | Empty formal review body (COMMENTED); no factual assertion or code request. Review metadata is not an independent defect. |
| 375 | reviews | 5435682214 | arthrod | push back | Empty formal review body (COMMENTED); no factual assertion or code request. Review metadata is not an independent defect. |
| 375 | reviews | 5435684914 | coderabbitai[bot] | push back | Empty formal review body (COMMENTED); no factual assertion or code request. Review metadata is not an independent defect. |
| 375 | reviews | 5435688460 | coderabbitai[bot] | push back | Empty formal review body (COMMENTED); no factual assertion or code request. Review metadata is not an independent defect. |
| 375 | reviews | 5435709802 | sourcery-ai[bot] | push back | Positive Sourcery review and generic reversible-risk assessment have no concrete new assertion or implementation request; known number-format and bounds claims verified above. No code change from this comment. |

## Embedded subfinding (outside inline comments)

| Parent review | Subfinding | Verdict | Evidence / state |
|---|---|---|---|
| 5453648416 | tests/convert_comment_placement.rs:224–228 unique CLI directory | implement | Fixed in the integrated source: cli_convert_takes_move_comments_and_changed_only now holds tempfile::tempdir() through assertions rather than fixed target path. Style/test isolation change needs no artificial new test; run existing CLI integration coverage. Concurrent Cargo is forbidden by project, but unique test directory also prevents stale output and interference from external invocations. |

All four individually gated work items and both audit follow-ups have local fixes. Initial three regressions passed parent coverage green; two follow-up regressions failed for their intended assertions and passed after the fixes.

## PR377 and PR380 records

| PR | Kind | ID | Author | Verdict | Evidence / state |
|---|---|---|---|---|---|
| 377 | comments | 6028397557 | sourcery-ai[bot] | push back | Provider budget/billing/rate-limit status only; no source finding. User instruction removes waiting requirement; no paid-account change within merge scope. |
| 377 | comments | 6028398140 | qodo-code-review[bot] | push back | Provider budget/billing/rate-limit status only; no source finding. User instruction removes waiting requirement; no paid-account change within merge scope. |
| 377 | comments | 6028398832 | chatgpt-codex-connector[bot] | push back | Completed Codex review status for 8dcc506, merge gate false, no concrete findings in full body; provider status is historical evidence, not current local validation. |
| 377 | comments | 6028399531 | coderabbitai[bot] | push back | Provider budget/billing/rate-limit status only; no source finding. User instruction removes waiting requirement; no paid-account change within merge scope. |
| 377 | comments | 6028400255 | sourcery-ai[bot] | push back | Full Sourcery guide inspected: API/ABI/CLI walkthrough and optional commands, no independent defect. Root owns binding integration verification; no code request from this guide. |
| 380 | comments | 6063771873 | coderabbitai[bot] | push back | Draft-review-not-run notification plus optional manual-review/config controls; no source claim. User explicitly instructed proceeding without waiting for CodeRabbit. |

Final input-record reconciliation: PR374 54 + PR375 29 + PR377 5 + PR380 1 = 89 records; implement 13 + push back 51 + duplicate-linked 25 = 89. Extra embedded review subfinding 5453648416/tempdir is individually gated as implement, outside these 89 GitHub records.

## Local implementation state (focused green validation completed)

After parent ran the three new deterministic regressions under coverage and confirmed the intended red assertions, this reviewer implemented all four remaining items in the integrated tree, without committing or invoking Cargo:

- 4216554951: `TextRun::comment_marker` is set by `mark_comment_anchors`; `run_word_count`, `document_bookmark_texts`, and `note_style_hit` exclude it. Justified painting preserves the flag in reconstructed pieces. Regression `comment_listing_review_tests::synthetic_comment_markers_do_not_enter_field_sources` pins all collectors and justified style capture.
- 4216554960: `listing_faces` routes script chunks through existing `ink_face`; `listing_width` and `paint_listing_line` share this segmentation. Regression `comment_listing_uses_loaded_script_fallback_for_paint_and_width` injects the font dependency's document-local fallback catalogue using bundled bytes and verifies face identity and measured line bounds. This proves fallback plumbing deterministically; actual installed font glyph coverage remains an integration concern.
- 4216554966: `wrap_listing_runs` wraps both bold labels and gray author attribution, splits oversized tokens, and preserves body paragraph breaks. Page-fit logic accounts for full heading height plus first content baseline, and oversized headings continue onto new pages. Regression `long_comment_attributions_wrap_and_preserve_printable_page_bounds` pins width, baseline, pagination, and content retention.
- 5453648416/tempdir: existing CLI integration test now holds `tempfile::tempdir()` for its input/output lifetime. No redundant new test added.

Formatting performed with `/Users/arthrod/.cargo/bin/rustfmt --edition 2024` on owned files. Parent owns green coverage, full verification, smoke test, commit, and publication. Earlier evidence naming `wrap_words`/`break_word` refers to original already-addressed PR374 code; this fix replaces them with `wrap_listing_runs` using script-aware measurement.

## Follow-up source audit

Reviewed all production `TextRun::new` reconstruction sites. Tab painting, field replacement, script splitting, page/column splitting, bidi pieces and normal line wrappers preserve full runs by clone/with_text; justified painting preserves the new flag explicitly. Two `wrap_runs_segment` merge guards can still combine source and synthetic text when styles match; `wrapping_keeps_synthetic_marker_classification_separate` reproduced that defect during parent coverage red. Both guards now require equal marker classification.

The listing replacement initially painted words separately. `a_script_listing_phrase_keeps_spaces_and_word_order_when_shaping` protects contiguous fallback text including neutral spaces and exact whole-phrase glyph ordering. The fixture uses dependency-native embedded-font injection with bundled bytes, and the test module now injects all catalogue keys to bypass installed font overrides. This also fixes the earlier test setup mismatch: the installed Calibri override has Hebrew while bundled Carlito does not. The corrected red phase was confirmed before the production follow-ups. Long-author fixture now numbers its source page explicitly, matching its page-label assertion.

## Follow-up implementation state

Parent confirmed corrected coverage red: marker reconstruction merged 1 run rather than 2, and Hebrew phrase split its neutral space into another face; long-author regression passed after explicit page numbering. Production follow-ups now require equal marker classification in both `wrap_runs_segment` merge guards, coalesce same-style listing text before shaping, preserve neutral spaces with the preceding script face, and measure each coalesced wrap candidate in the exact faces used by painting. Existing whole-phrase HarfBuzz ordering is preserved without introducing unrelated paragraph bidi behavior. All five focused regressions passed with branch coverage. Complete checks are recorded in Validation.

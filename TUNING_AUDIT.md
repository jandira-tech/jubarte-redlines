<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Convert tuning-constant audit (plan Step 8)

One row per `src/convert` site that `grep -n -i` matches as a word-boundary
`mini` (the historical mini-bench locks), plus the Step 8 starters
`word_device_track` / `word_device_paint` / `word_device_pt` and the
heading-gap helper `apply_latent_ppr`.

Class:

- **a** — substitute for a font metric. Retire with plan Step 4 (real face metrics / Word device), not a per-document constant.
- **b** — document-specific gate. Remove only after both the 398 corpus and the 76 fixtures still hold.
- **c** — genuine Word / ECMA-376 rule. Keep the behaviour; replace the “mini NNN” justification with the spec or Word-oracle citation.

Disposition is the action, not a score. Parked items stay parked; this table does not author engine edits.

| file | line | symbol | class | disposition |
|---|---|---|---|---|
| src/convert/font.rs | 182 | word_device_track | a | retire with Step 4: Word Quartz 300dpi Tc (−0.0015 at 11.04pt, −0.0018 at 16.08pt); linear hmtx elsewhere |
| src/convert/font.rs | 195 | word_device_paint | a | retire with Step 4: Word Quartz paints 11.04/16.08 as 46/67 ppem inside 0.24 cm; not a per-document gate |
| src/convert/font.rs | 1517 | mini 727 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. // liga (mini 727) was ITT-neg: file_170 −0.0036 / potpourri |
| src/convert/font.rs | 1522 | mini 727 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. assert_eq!(g.len(), 2, "mini 727 Aptos 12 liga ITT-neg; glyphs={g:?}"); |
| src/convert/font.rs | 1636 | mini 505 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. "mini 505 ITT-neg WideLatin overlay; keep Calibri" |
| src/convert/metafile.rs | 716 | mini 365 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. // 5×7 bitmap digits (mini 365) were Word-shaped but ITT-neg: |
| src/convert/metafile.rs | 727 | mini 365 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. "mini 365 EXTTEXTOUTW bitmap ITT-neg; dark={}", |
| src/convert/mod.rs | 229 | word_device_pt | a | retire with Step 4: snap 10/11/16/32pt to integer 300dpi ppem; other sizes stay raw because ungated snaps ITT-dropped fixtures |
| src/convert/mod.rs | 231 | mini snap, mini snap8 | a | class-a size snap / 300dpi device substitute; gated because ungated snaps ITT-drop. // Do not snap 8pt (sd_2517 cover 7.92 is Word-faithful but mini snap8 |
| src/convert/mod.rs | 232 | mini 99 | b | Document-specific mini site; remeasure 398+76. // dropped file_34 −0.011 with sd_2517/file_22 ~0), 9.5 (mini 99), |
| src/convert/mod.rs | 233 | mini 110 | b | Document-specific mini site; remeasure 398+76. // 10.5 (mini 110: I_am_sharing −1.14, comments-lots −1.23, |
| src/convert/mod.rs | 234 | mini 105 | b | Document-specific mini site; remeasure 398+76. // image_out_of_folder −3.23), 20/28 (mini 105), 14/15 |
| src/convert/mod.rs | 235 | mini 522 | b | Document-specific mini site; remeasure 398+76. // (heading_3 / file_61), Calibri 14 (mini 522: comments-lots family |
| src/convert/mod.rs | 236 | mini 429 | b | Document-specific mini site; remeasure 398+76. // −0.03 to −0.06 / file_8 −0.33), or 13/26 (mini 429: table_bookmark |
| src/convert/mod.rs | 237 | mini 704 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. // −0.070 / file_134 −0.059; mini 704 Calibri-Light also ITT-neg). |
| src/convert/mod.rs | 261 | mini 522 | a | class-a size snap / 300dpi device substitute; gated because ungated snaps ITT-drop. // (58 ppem). Calibri 14 (mini 522) and Arial 14 (heading_3) |
| src/convert/mod.rs | 267 | mini 105 | a | class-a size snap / 300dpi device substitute; gated because ungated snaps ITT-drop. // 28.08). Ungated 28 snap (mini 105) dropped file_34 Arial |
| src/convert/mod.rs | 337 | mini 504 | b | Document-specific mini site; remeasure 398+76. /// Empty `TOC` field (no cached `w:t`). Mini 504 collapse-to-zero |
| src/convert/mod.rs | 469 | mini 112 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. /// Not the table-level rPr color — that was mini 112 ITT-wrong. |
| src/convert/mod.rs | 697 | mini 569 | b | Document-specific mini site; remeasure 398+76. /// Ungated 3-col pad (mini 569) also compacted Strict01 |
| src/convert/mod.rs | 840 | mini 511 | b | KEEP: measured gate. /// Other boxes keep the 0.6 black hairline (mini 511). |
| src/convert/mod.rs | 844 | mini 511 | b | Document-specific mini site; remeasure 398+76. /// (mini 511) / 1.0 (KEEP 591 lnRef idx=2). KEEP 512 a:ln |
| src/convert/mod.rs | 864 | mini 414 | b | KEEP: Word-shaped change lost named fixtures. /// Do not add ECMA bodyPr lIns=7.2: stacked (mini 414) dropped mcdoc |
| src/convert/mod.rs | 865 | mini 417 | b | Document-specific mini site; remeasure 398+76. /// −1.83; unindented-only (mini 417) dropped RL Strict01/file_100. |
| src/convert/mod.rs | 870 | mini 414, mini 510 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. /// (mini 414/417 ITT-neg) or tIns/bIns (mini 510 ITT-neg: XML 3.6pt |
| src/convert/mod.rs | 871 | mini 647–650 | b | Document-specific mini site; remeasure 398+76. /// vs pad=4 dropped Strict01 family −0.049). Mini 647–650 |
| src/convert/mod.rs | 937 | mini 639–642 | b | Document-specific mini site; remeasure 398+76. /// Mini 639–642: relativeFrom=margin (Text Box 2 40% of content |
| src/convert/mod.rs | 1732 | mini 90 | b | Document-specific mini site; remeasure 398+76. // file_134, but applying it (mini 90) also retargeted file_2 / |
| src/convert/mod.rs | 1734 | mini 396 | b | Document-specific mini site; remeasure 398+76. // Cambria para gap is ~24.7 (line ~14.9 + after). Mini 396 on the |
| src/convert/mod.rs | 1760 | // Aptos-only gate was a mini-set trade  | b | KEEP retired: Aptos-only minor-slot gate was a mini-set trade; do not restore. |
| src/convert/mod.rs | 1830 | mini 350 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. // Not w14:shadow extra copy (mini 350 ITT-neg). |
| src/convert/mod.rs | 1834 | mini 350 | b | Document-specific mini site; remeasure 398+76. // (filled bars in the oracle). Shadow-only (mini 350) still |
| src/convert/mod.rs | 1835 | mini 371 | b | Document-specific mini site; remeasure 398+76. // paints. textOutline+w:color with explicit sz (mini 371 Keyword |
| src/convert/mod.rs | 2356 | mini 108 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. // Keep the PUA (mini 108 U+00B7 was ITT-wrong); append |
| src/convert/mod.rs | 2405 | mini sechang | b | Document-specific mini site; remeasure 398+76. // hanging packed sd_2517 107→106 (mini sechang). |
| src/convert/mod.rs | 2532 | mini 108 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. // U+00B7 (mini 108) put the real bullet at x=72, but ITT dropped the |
| src/convert/mod.rs | 2860 | mini 619–622 | c | Word/ECMA behaviour; replace mini N citation with the oracle/spec. // Mini 619–622: Word-faithful `w:separator` 144×0.72 (Strict01 p13) |
| src/convert/mod.rs | 3228 | mini 342 | b | KEEP: measured gate. // first-row tcW (mini 342) dropped comments-lots. Keep the cache. |
| src/convert/mod.rs | 3454 | apply_latent_ppr | c | Word latent Heading3/4 spacing (before=10 after=0) when styles.xml omits the definition; Heading1 stays defaults (red_bold_heading). Cite Word latent built-ins, not a mini N |
| src/convert/mod.rs | 3529 | mini 423 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. // does not strike/underline the bullet (mini 423 ITT −0.003 |
| src/convert/mod.rs | 3748 | mini 338–341 | b | Document-specific mini site; remeasure 398+76. // Gating last_row_fill to tblLook lastRow=0 (mini 338–341) was |
| src/convert/mod.rs | 3846 | mini 454 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. // but mini 454 ITT-neg: file_100/115/185/196 13→14pp (−23 ITT). |
| src/convert/mod.rs | 3886 | mini 78, mini empty | b | Document-specific mini site; remeasure 398+76. // sample −2.5 (mini 78 and mini empty). Skip empty |
| src/convert/mod.rs | 3952 | mini 59 | b | KEEP: ITT drop on named fixtures. // rewrite trPr/del rows — that was mini 59 (−5 ITT). |
| src/convert/mod.rs | 4160 | mini 221–224 | b | KEEP: ITT drop on named fixtures. // (mini 221–224) dropped Cicero −0.027 ITT (2.6pt pad, >5px align). |
| src/convert/mod.rs | 4165 | mini 430 | b | Document-specific mini site; remeasure 398+76. // Fixed L/R pad 0 (mini 430) was Word Test 1 x=90 (+0.059) but |
| src/convert/mod.rs | 4329 | mini 59 | b | Document-specific mini site; remeasure 398+76. // lines — extra ink vs the oracle, not mini 59 (whole-row rewrite). |
| src/convert/mod.rs | 4330 | mini 739 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. // Mini 739 repeated once per cellDel (Word 3 lines) but ITT-neg |
| src/convert/mod.rs | 4488 | mini 401 | b | KEEP: measured gate. /// Body without pBdr stays collapsed (mini 401). Courier New body |
| src/convert/mod.rs | 4489 | mini 520 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. /// pads (file_69 code) stay collapsed too (mini 520 ITT-neg). |
| src/convert/mod.rs | 4566 | mini 336–337 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. // RedBoldCharacter 12pt) was mini 336–337 ITT-neg: redline |
| src/convert/mod.rs | 4603 | mini 732 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. // index. Mini 732 put Word #005B70 in slot 1 and ITT-neg'd NR |
| src/convert/mod.rs | 4606 | mini 737 | b | Document-specific mini site; remeasure 398+76. // file_146 (first-seen index 1 vs 2). Mini 737 name-keyed |
| src/convert/mod.rs | 4614 | mini 732 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. // (mini 732 slot-1 retune ITT-neg). Color is name-keyed. |
| src/convert/mod.rs | 4650 | mini 239 | b | Document-specific mini site; remeasure 398+76. // there. Second/third-author del as ins palette (mini 239) |
| src/convert/mod.rs | 4928 | mini 359 | b | Document-specific mini site; remeasure 398+76. // Strict01 binomial is m:f type=noBar. Linear n/k (mini 359) |
| src/convert/mod.rs | 5005 | mini 88 | b | Document-specific mini site; remeasure 398+76. // (mini 88). Paragraph-level keep of ≥3 generator pads (file_146 |
| src/convert/mod.rs | 5006 | mini 401 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. // Suggestion mode → Word page-2 Serialises) was mini 401 ITT-neg: |
| src/convert/mod.rs | 5268 | mini 511 | b | Document-specific mini site; remeasure 398+76. // Distinct from mini 511 a:ln/@w width (still 0.6 when stroking). |
| src/convert/mod.rs | 5269 | mini 568 | b | Document-specific mini site; remeasure 398+76. // Chart-bearing boxes still stroke 0.6 black (mini 568): |
| src/convert/mod.rs | 6431 | mini 511 | b | Document-specific mini site; remeasure 398+76. // a:ln/@w when present. Box emit still ignores this (mini 511). |
| src/convert/mod.rs | 6460 | mini 715 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. // Mini 715 two-stop Type 2 axial was Word-faithful but ITT-neg |
| src/convert/mod.rs | 7842 | mini 504 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. // Mini 504 collapse-to-zero ITT-neg. Do not use ascent |
| src/convert/mod.rs | 7914 | mini 217–220 | b | KEEP: measured gate. // Do not skip empty/del-only pBdr (mini 217–220): no-redline |
| src/convert/mod.rs | 7951 | mini 440 | b | Document-specific mini site; remeasure 398+76. // Honoring T/B w:space (mini 440) was Word-shaped (file_146 |
| src/convert/mod.rs | 7954 | mini 480–483 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. // space=4 (mini 480–483) was also ITT-neg: NR 16 comments-lots |
| src/convert/mod.rs | 7959 | mini 225–228 | b | KEEP: measured gate. // Do not outset 1.44pt / 6px@300dpi (mini 225–228): Word |
| src/convert/mod.rs | 7969 | // Word's extra 1.44pt Quartz outset is  | c | Word/ECMA behaviour; replace mini N citation with the oracle/spec. // Word's extra 1.44pt Quartz outset is gated to 4-edge — mini |
| src/convert/mod.rs | 7971 | mini 440 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. // lock) and ITT-neg file_134 −0.003. Not mini 440 T/B space. |
| src/convert/mod.rs | 7991 | mini 225–228 | b | Document-specific mini site; remeasure 398+76. // rules (mini 225–228 file_134 −0.003) unless L/R exist. |
| src/convert/mod.rs | 8002 | mini revx | b | Document-specific mini site; remeasure 398+76. // CiceroDo Word is margin_l-36=54, but shipping that (mini revx) |
| src/convert/mod.rs | 8011 | mini revx | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. // Do not move x to margin_l-36 (mini revx ITT-neg). Merge |
| src/convert/mod.rs | 8046 | mini 279 | b | Document-specific mini site; remeasure 398+76. // Empty-spacer slack 40 (mini 279) lifted file_146 +0.039 |
| src/convert/mod.rs | 8573 | mini 523 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. // Word 0.24pt (file_34) was mini 523 ITT-neg. |
| src/convert/mod.rs | 8581 | mini 197 | b | Document-specific mini site; remeasure 398+76. // size×0.075 on all u: mini 197 median −0.007 |
| src/convert/mod.rs | 8582 | mini 199 | b | Document-specific mini site; remeasure 398+76. // (green_underline 90.4→89.2). size≥20: mini 199 mean |
| src/convert/mod.rs | 8583 | mini 238 | b | Document-specific mini site; remeasure 398+76. // −0.007. 28pt+ / 32pt title-only (mini 238) no-redline |
| src/convert/mod.rs | 8584 | mini 470 | b | Document-specific mini site; remeasure 398+76. // 59.1612→59.1552. 9.5pt→0.48 (file_146 github, mini 470) |
| src/convert/mod.rs | 8809 | mini 623 | b | Document-specific mini site; remeasure 398+76. // 631 title y+dh-19 compensated that 3pt. Mini 623 |
| src/convert/mod.rs | 9930 | mini 635–638 | b | Document-specific mini site; remeasure 398+76. // Mini 635–638: Word wrapNone Rectangle 1 closed |
| src/convert/mod.rs | 9936 | mini 511 | b | Document-specific mini site; remeasure 398+76. // lnRef idx (Rectangle 1 idx=2 → 1pt). Mini 511 |
| src/convert/mod.rs | 9938 | mini 568 | b | KEEP: measured gate. // ChartSpace 0.6 black stays 4-edge (mini 568). |
| src/convert/mod.rs | 9958 | mini 568 | b | KEEP: measured gate. // corners. Mini 568 keeps 0.6 black (do not skip; |
| src/convert/mod.rs | 9959 | mini 384, mini 635 | b | KEEP: mini lock on chart/layout constant; remeasure 398+76. // do not add 0.75 gray mini 384). Mini 635 locked |
| src/convert/mod.rs | 10091 | mini 635 | b | KEEP: mini lock on chart/layout constant; remeasure 398+76. // 4-edge Lines grow square-cap corners. Mini 635 locked |
| src/convert/mod.rs | 10154 | mini 511 | b | KEEP: measured gate. // without @w is 1pt. Box strokes stay 0.6 (mini 511). |
| src/convert/mod.rs | 10181 | mini 522 | b | KEEP: measured gate. // Body Calibri 14 stays 14.00 (mini 522). SmartArt 14pt labels |
| src/convert/mod.rs | 10182 | mini 453 | b | KEEP: measured gate. // do not go through emit_label (mini 453 lock). |
| src/convert/mod.rs | 10198 | mini 385–388 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. // Not grid 0.85 (mini 385–388 ITT-neg vs Quartz 0.88). |
| src/convert/mod.rs | 10437 | mini 384 | b | KEEP: mini lock on chart/layout constant; remeasure 398+76. // plot. Mini 384 locked the 0.75 gray *frame*; fill-only. |
| src/convert/mod.rs | 10481 | mini 381 | b | KEEP: measured gate. // at plot_x (mini 381 + KEEP 694). valAx labels stay x+6.5. |
| src/convert/mod.rs | 10486 | mini 381, mini 428 | b | KEEP: mini lock on chart/layout constant; remeasure 398+76. // dy 31. Mini 381 locked bar width; mini 428 locked legend x. |
| src/convert/mod.rs | 10489 | mini 691 | b | Document-specific mini site; remeasure 398+76. // Mini 691 sat FillRect on axis_y; NR +0.0059 8/0 but RL mean |
| src/convert/mod.rs | 10500 | mini 381 | b | KEEP: mini lock on chart/layout constant; remeasure 398+76. // Mini 381 locked gapWidth/overlap (packed ~27.6 width stays). |
| src/convert/mod.rs | 10504 | mini 385–388 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. // tx1 lumMod=15%/lumOff=85% → 0.85 (mini 385–388) ITT-neg vs Quartz 0.88. |
| src/convert/mod.rs | 10511 | mini 385, mini 690 | b | KEEP: measured gate. // line at plot_y. Mini 385 color stays. Mini 690 0.75pt |
| src/convert/mod.rs | 10549 | mini 428 | b | KEEP: mini lock on chart/layout constant; remeasure 398+76. // Td 323). y+34 leftover was 325.9. Mini 428 locked x. |
| src/convert/mod.rs | 10558 | mini 384, mini 385 | b | KEEP: measured gate. // (mini 384 greps 0.850). valAx grid stays 0.4pt 0.88 (mini 385 |
| src/convert/mod.rs | 10559 | mini 690 | b | Document-specific mini site; remeasure 398+76. // color / mini 690 width). |
| src/convert/mod.rs | 10581 | mini 428 | b | KEEP: mini lock on chart/layout constant; remeasure 398+76. // ink. Mini 428 locked centering the row, not the size. |
| src/convert/mod.rs | 10902 | mini 536 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. // mini 536 ITT-neg: file_34 −0.82 / uipriority −1.05, 0 gains. |
| src/convert/mod.rs | 11067 | mini 244 | b | Document-specific mini site; remeasure 398+76. // Quartz 1.44pt outset (mini 244) was no-redline mean |
| src/convert/mod.rs | 11069 | mini 225–228 | b | Document-specific mini site; remeasure 398+76. // content box like body pBdr (mini 225–228). |
| src/convert/mod.rs | 11098 | mini 244 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. // mini 244 chrome outset ITT-neg; keep content box. |
| src/convert/mod.rs | 11378 | mini 57 | b | Document-specific mini site; remeasure 398+76. /// Keep `://` intact. Not generic character-break (Test 7 / mini 57). |
| src/convert/mod.rs | 11436 | mini sechang | b | Document-specific mini site; remeasure 398+76. // but treating it as a marker packed 107→106pp (mini sechang −0.10). |
| src/convert/mod.rs | 11441 | // file_146 ListBullet lvlText is U+2013 | b | Document-specific mini site; remeasure 398+76. // file_146 ListBullet lvlText is U+2013 (–). Hanging it (mini |
| src/convert/mod.rs | 11677 | // Do not max body→Heading1 (potpourri b | b | KEEP: measured gate. // Do not max body→Heading1 (potpourri before=18): mini |
| src/convert/mod.rs | 11703 | mini 627–630 | b | Document-specific mini site; remeasure 398+76. // Mini 627–630: default w:widowControl (orphan 2-line |
| src/convert/mod.rs | 11771 | mini 623–626 | b | Document-specific mini site; remeasure 398+76. // Mini 623–626: skipping Normal after=8 under a |
| src/convert/mod.rs | 14725 | mini 360 | b | Document-specific mini site; remeasure 398+76. // Strict01 m:r rFonts Cambria Math + TTC face 1 (mini 360) was |
| src/convert/mod.rs | 14752 | mini 360 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. "mini 360 Cambria Math ITT-neg; family={:?}", |
| src/convert/mod.rs | 14806 | mini 370 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. // Applying RGB×0.5 (mini 370) was Word-shaped but ITT-neg: |
| src/convert/mod.rs | 14845 | mini 370 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. "mini 370 lumMod ITT-neg; keep unmodulated {want:?}, got {:?}", |
| src/convert/mod.rs | 14853 | mini 371 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. // Tr=2 (mini 371) was Word-shaped but ITT-neg: Strict01 family |
| src/convert/mod.rs | 14888 | mini 371 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. "mini 371 outline ITT-neg; fill stays F7CAAC, got {:?}", |
| src/convert/mod.rs | 14942 | mini 359 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. // (mini 359) was Word-shaped but ITT-neg: Strict01 family |
| src/convert/mod.rs | 14966 | mini 359 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. assert_eq!(joined, "x+a", "mini 359 parens ITT-neg; joined={joined:?}"); |
| src/convert/mod.rs | 14975 | mini 359 | b | Document-specific mini site; remeasure 398+76. // Strict01 binomial is m:f type=noBar. Linear n/k (mini 359) |
| src/convert/mod.rs | 15003 | mini 359 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. "mini 359 linear slash ITT-neg; joined={joined:?}" |
| src/convert/mod.rs | 15019 | mini 359 | b | Document-specific mini site; remeasure 398+76. // Strict01 binomial is m:f type=noBar. Linear n/k (mini 359) |
| src/convert/mod.rs | 15044 | mini 359 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. assert_eq!(joined, "nk", "mini 359 n/k ITT-neg; joined={joined:?}"); |
| src/convert/mod.rs | 15154 | mini 401 | b | Document-specific mini site; remeasure 398+76. // put file_146 Serialises on page 2 but mini 401 dropped the |
| src/convert/mod.rs | 15177 | mini 401 | b | KEEP: measured gate. "mini 401: body generator pad stays collapsed, got {joined:?}" |
| src/convert/mod.rs | 15212 | mini 401 | b | Document-specific mini site; remeasure 398+76. // carry bottom pBdr E2E8F0 plus generator xml:space pads. Mini 401 |
| src/convert/mod.rs | 15242 | mini 521 | c | Word/ECMA behaviour; replace mini N citation with the oracle/spec. // Stripping potpourri/file_19 U+FEFF (Word-faithful) was mini 521 |
| src/convert/mod.rs | 15262 | mini 521 | b | KEEP test lock. "mini 521: keep U+FEFF in the run, got {joined:?}" |
| src/convert/mod.rs | 15269 | mini 520 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. // onto page 2 (Word) but mini 520 ITT-neg: NR 59.4772→59.0833 / |
| src/convert/mod.rs | 15271 | mini 401 | b | KEEP: measured gate. // −7. Same packing class as mini 401. Stay collapsed. |
| src/convert/mod.rs | 15292 | mini 520 | b | KEEP: measured gate. "mini 520: Courier body xml:space stays collapsed, got {joined:?}" |
| src/convert/mod.rs | 15419 | mini 429 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. // Word Quartz 26pt/13pt is 25.92/12.96 but mini 429 ITT-neg |
| src/convert/mod.rs | 15668 | mini 417 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. "mini 417 ITT-neg unindented lIns=7.2; keep pad=4 path text_dx=0; text_dx={}", |
| src/convert/mod.rs | 21164 | mini 57 | b | KEEP: ITT drop on named fixtures. // (mini 57 / table-gated −24 ITT). |
| src/convert/mod.rs | 21559 | mini 385–388 | b | KEEP pending 398+76 remeasure; ITT-neg/wrong. // (mini 385–388 ITT-neg). This XML must still parse series. |

---
view: tracked                      # revisions as CriticMarkup, comments inline
track_changes: off                 # w:trackRevisions not set; new edits are not tracked unless edit sets it
revisions: 67                      # 67 marks shown as 65 changes: 52 insertions, 13 deletions
comments: 0
authors:
  document_owner: html-to-docx     # dc:creator
  ASR: Arthur Souza Rodrigues      # 67 revisions, 2026-05-02..2026-05-02
body: p0-p131, 2 tables, 5 pages   # pages from layout
page: Letter portrait, margins top 1in, right 1.25in, bottom 1in, left 1.25in, header/footer 0.5in
styles:
  Normal: Times New Roman 11pt, after 6pt, line 12pt atLeast, left, named normal  # default; unannotated paragraphs use it
  "#": Heading1, Times New Roman bold 24pt, before 24pt, keep-next, keep-lines
  "##": Heading2, Times New Roman bold 18pt, before 18pt, after 4pt, keep-next, keep-lines
  table: TableNormal
---

<!-- p0 -->
The Pot-Pourri Compendium

<!-- p1 -->
A Sampler Document for Extraction Testing

<!-- p2 -->
This document is intentionally varied. It mixes headings, body copy, inline formatting, colors, lists, a table, footnotes, a hyperlink, and a block quote so an extraction tool has plenty to chew on. **Emphasis lives here. See note.**<sup>[\[1\]](#footnote-1)</sup>﻿

<!-- p3 -->
# 1. Typography Samples

<!-- p4 -->
Below are inline runs with mixed formatting. Each run is a separate span so an extractor can detect boundaries.

<!-- p5 -->
**Bold blue, italic red, underlined green, yellow-highlighted, strikethrough, monospace.**

<!-- p6 -->
# 2. Lists

<!-- p7 -->
## 2.1 Bulleted

<!-- p8 -->
Apples — a pomaceous fruit

<!-- p9 -->
Bourbon — distilled in Kentucky

<!-- p10 -->
Cardamom — a fragrant spice

<!-- p11 -->
## 2.2 Numbered (with nested)

<!-- p12 -->
Preheat the oven to 375°F.

<!-- p13 -->
Whisk dry ingredients.

<!-- p14 -->
Sift the flour twice.

<!-- p15 -->
Add a pinch of salt.

<!-- p16 -->
Bake for 22 minutes.

<!-- p17 -->
# 3. Tabular Data

<!-- p18 -->
A small table with a header row:

<!-- t0 center 5x4, cells p19-p38 by row, borders single 0.25pt 000000, cell-borders top,left,bottom,right single 1pt CCCCCC in all -->
|Region|Q1|Q2|Q3|
|:-|:-|:-|:-|
|North|$120k|$135k|$150k|
|South|$98k|$110k|$128k|
|East|$143k|$151k|$162k|
|West|$87k|$92k|$104k|

<!-- p39 empty -->

<!-- p40 -->
# 4. Quote & Link

<!-- p41 -->
“Not all those who wander are lost.” — a line often quoted to test block-quote handling.

<!-- p42 -->
More information available at our [reference page](https://example.com/reference).

<!-- p43 -->
This sentence carries a second footnote for testing multi-note extraction. †<sup>[\[2\]](#footnote-2)</sup>﻿

<!-- p44 -->
# 5. Color Callouts

<!-- p45 -->
**WARNING: Do not feed the extraction tool after midnight.**

<!-- p46 -->
**INFO: Tables, lists, and footnotes should each round-trip cleanly.**

<!-- p47 -->
_— end of sample —_

<!-- p48 -->
{++The Pot-Pourri Compendium++}{>>r1 @ASR<<}

<!-- p49 -->
{++A Sampler Document for Extraction Testing++}{>>r2 @ASR<<}

<!-- p50 -->
{++This document is intentionally varied. It mixes headings, body copy, inline formatting, colors, lists, a table, footnotes, a hyperlink, and a block quote so an extraction tool has plenty to chew on.++}{>>r3 @ASR<<} **Emphasis lives here. See note.**{++<sup>[\[3\]](#footnote-3)</sup>++}{>>r3 @ASR<<}﻿

<!-- p51 -->
# {++1. Typography Samples++}{>>r4 @ASR<<}

<!-- p52 -->
{++Below are inline runs with mixed formatting. Each run is a separate span so an extractor can detect boundaries.++}{>>r5 @ASR<<}

<!-- p53 -->
{++**Bold blue,** ++}{>>r6 @ASR<<}**italic red, underlined green, yellow-highlighted,** {++**strikethrough, monospace.**++}{>>r6 @ASR<<}

<!-- p54 -->
# {++2. Lists++}{>>r7 @ASR<<}

<!-- p55 -->
## {++2.1 Bulleted++}{>>r8 @ASR<<}

<!-- p56 -->
{++Apples — a pomaceous fruit++}{>>r9 @ASR<<}

<!-- p57 -->
{++Bourbon — distilled in Kentucky++}{>>r10 @ASR<<}

<!-- p58 -->
{++Cardamom — a fragrant spice++}{>>r11 @ASR<<}

<!-- p59 -->
## {++2.2 Numbered (with nested)++}{>>r12 @ASR<<}

<!-- p60 -->
{++Preheat the oven to 375°F.++}{>>r13 @ASR<<}

<!-- p61 -->
{++Whisk dry ingredients.++}{>>r14 @ASR<<}

<!-- p62 -->
{++Sift the flour twice.++}{>>r15 @ASR<<}

<!-- p63 -->
{++Add a pinch of salt.++}{>>r16 @ASR<<}

<!-- p64 -->
{++Bake for 22 minutes.++}{>>r17 @ASR<<}

<!-- p65 -->
# {++3. Tabular Data++}{>>r18 @ASR<<}

<!-- p66 -->
{++A small table with a header row:++}{>>r19 @ASR<<}

<!-- t1 center 5x4, cells p67-p86 by row, borders single 0.25pt 000000, cell-borders top,left,bottom,right single 1pt CCCCCC in all -->
|{++Region++}{>>r20 @ASR<<}|{++Q1++}{>>r21 @ASR<<}|{++Q2++}{>>r22 @ASR<<}|{++Q3++}{>>r23 @ASR<<}|
|:-|:-|:-|:-|
|{++North++}{>>r24 @ASR<<}|{++$120k++}{>>r25 @ASR<<}|{++$135k++}{>>r26 @ASR<<}|{++$150k++}{>>r27 @ASR<<}|
|{++South++}{>>r28 @ASR<<}|{++$98k++}{>>r29 @ASR<<}|{++$110k++}{>>r30 @ASR<<}|{++$128k++}{>>r31 @ASR<<}|
|{++East++}{>>r32 @ASR<<}|{++$143k++}{>>r33 @ASR<<}|{++$151k++}{>>r34 @ASR<<}|{++$162k++}{>>r35 @ASR<<}|
|{++West++}{>>r36 @ASR<<}|{++$87k++}{>>r37 @ASR<<}|{++$92k++}{>>r38 @ASR<<}|{++$104k++}{>>r39 @ASR<<}|

<!-- p87 empty -->

<!-- p88 -->
﻿

<!-- p89 -->
# {++4. Quote & Link++}{>>r40 @ASR<<}

<!-- p90 -->
{++“Not all those who wander are lost.” — a line often quoted to test block-quote handling.++}{>>r41 @ASR<<}

<!-- p91 -->
{++More information available at our ++}{>>r42 @ASR<<}[reference page](https://example.com/reference){++.++}{>>r42 @ASR<<}

<!-- p92 -->
{++This sentence carries a second footnote for testing multi-note extraction. †<sup>[\[4\]](#footnote-4)</sup>++}{>>r43+43 @ASR<<}﻿

<!-- p93 -->
# {++5. Color Callouts++}{>>r44 @ASR<<}

<!-- p94 -->
{++**WARNING: Do not feed the extraction tool after midnight.**++}{>>r45 @ASR<<}

<!-- p95 -->
{++**INFO: Tables, lists, and footnotes should each round-trip cleanly.**++}{>>r46 @ASR<<}

<!-- p96 -->
{++_— end of sample —_++}{>>r47 @ASR<<}

<!-- p97 -->
﻿

<!-- p98 -->
The Pot-Pourri Compendium

<!-- p99 -->
A Sampler Document for Extraction Testing

<!-- p100 -->
This document is intentionally {--varied. It mixes headings, body copy, inline formatting, colors, lists, a table, footnotes, a hyperlink, and a --}{>>r48 @ASR<<}block quote so an extraction tool has plenty to chew on. **Emphasis lives here. See note.**<sup>[\[5\]](#footnote-5)</sup>﻿

<!-- p101 -->
# 1. Typography Samples

<!-- p102 -->
Below are inline runs with mixed formatting. Each run is a separate span so an extractor can detect boundaries.

<!-- p103 -->
**Bold blue, italic red, underlined green, yellow-highlighted, strikethrough, monospace.**

<!-- p104 -->
# 2. Lists

<!-- p105 -->
## 2.1 Bulleted

<!-- p106 -->
Apples — a pomaceous fruit

<!-- p107 -->
Bourbon — distilled in Kentucky

<!-- p108 -->
Cardamom — a fragrant spice

<!-- p109 -->
## 2.2 Numbered (with nested)

<!-- p110 -->
Preheat the oven to 375°F.

<!-- p111 -->
Whisk dry ingredients.

<!-- p112 -->
Sift the flour twice.

<!-- p113 -->
Add a pinch of salt.

<!-- p114 -->
{--Bake for 22 minutes.--}{>>r49 @ASR<<}

<!-- p115 -->
# {--3. Tabular Data--}{>>r50 @ASR<<}

<!-- p116 -->
{--A small table with a header row:--}{>>r51 @ASR<<}

<!-- p117 -->
﻿

<!-- p118 -->
# {--4. Quote & Link--}{>>r52 @ASR<<}

<!-- p119 -->
{--“Not all those who wander are lost.” — a line often quoted to test block-quote handling.--}{>>r53 @ASR<<}

<!-- p120 -->
{--More information available at our reference page.--}{>>r54 @ASR<<}

<!-- p121 -->
{--This sentence carries a second footnote for testing multi-note extraction. †<sup>[\[6\]](#footnote-6)</sup>--}{>>r55+55 @ASR<<}﻿

<!-- p122 -->
# {--5. Color Callouts--}{>>r56 @ASR<<}

<!-- p123 -->
{--**WARNING: Do not feed the extraction tool after midnight.**--}{>>r57 @ASR<<}

<!-- p124 -->
{--**INFO: Tables, lists, and footnotes should each round-trip cleanly.**--}{>>r58 @ASR<<}

<!-- p125 -->
{--_— end of sample —_--}{>>r59 @ASR<<}

<!-- p126 -->
Footnote one: this footnote is attached to the introductory paragraph. [↑](#footnote-ref-1)﻿

<!-- p127 -->
Footnote two: cross-reference target for extraction tools that map markers to notes. [↑](#footnote-ref-2)﻿

<!-- p128 -->
{++ Footnote one: this footnote is attached to the introductory paragraph.++}{>>r60 @ASR<<} [↑](#footnote-ref-3)﻿

<!-- p129 -->
{++ Footnote two: cross-reference target for extraction tools that map markers to notes.++}{>>r61 @ASR<<} [↑](#footnote-ref-4)﻿

<!-- p130 -->
Footnote one: this footnote is attached to the introductory paragraph. [↑](#footnote-ref-5)﻿

<!-- p131 -->
{-- Footnote two: cross-reference target for extraction tools that map markers to notes.--}{>>r62 @ASR<<} [↑](#footnote-ref-6)﻿

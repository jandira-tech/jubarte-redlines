---
view: tracked                      # revisions as CriticMarkup, comments inline
track_changes: on                  # w:trackRevisions set; edits are tracked
revisions: 176                     # 170 marks shown as 170 changes: 47 insertions, 32 deletions, 81 paragraph marks, 10 table rows; 6 in footnotes
comments: 0
authors:
  document_owner: Arthur Souza Rodrigues  # dc:creator
  ASR: Arthur Souza Rodrigues      # 176 revisions, 2026-05-02..2026-05-02
body: p0-p145, 3 tables, 4 pages   # pages from Word's cached layout
page: Letter portrait, margins 1in, header/footer 0.5in
styles:
  Normal: Aptos 12pt, after 8pt, line 1.16, left  # default; unannotated paragraphs use it
  "#": Heading1, Aptos Display 20pt, before 18pt, after 4pt, keep-next, keep-lines
  "##": Heading2, Aptos Display 16pt, before 8pt, after 4pt, keep-next, keep-lines
  Title: Aptos Display 28pt, after 4pt, contextual spacing
  Subtitle: Aptos 14pt
  ListBullet: Aptos 12pt, contextual spacing, named List Bullet
  ListNumber: Aptos 12pt, contextual spacing, named List Number
  Quote: Aptos italic 12pt, before 8pt, center
  table: GridTable4-Accent1, all borders 0.5pt
notes: {empty: [endnotes]}         # notes parts that hold no note
headers:
  first: {id: header3, text: "", inactive}  # defined, but different first page is off
  default: {id: header2, text: ""}
  even: {id: header1, text: "", inactive}  # defined, but even/odd headers are off
footers:
  first: {id: footer3, text: "", inactive}  # defined, but different first page is off
  default: {id: footer2, text: ""}
  even: {id: footer1, text: "", inactive}  # defined, but even/odd headers are off
---

<!-- p0 Title -->
# The Pot-Pourri Compendium

<!-- p1 Subtitle -->
A Sampler Document for Extraction Testing

<!-- p2 -->
This document is intentionally varied. It mixes headings, body copy, inline formatting, colors, lists, a table, footnotes, a hyperlink, and a block quote so an extraction tool has plenty to chew on. **_Emphasis lives here. See note._**[^1]

<!-- p3 -->
# 1. Typography Samples

<!-- p4 -->
Below are inline runs with mixed formatting. Each run is a separate span so an extractor can detect boundaries.

<!-- p5 -->
**Bold blue,** **_italic red,_** **_<u>underlined green,&#32;</u>_**<b><i><u><mark>yellow-highlighted,&#32;</mark></u></i></b>**_<u><mark>~~strikethrough,&#32;~~</mark></u>_**<b><i><u><mark>~~`monospace.`~~</mark></u></i></b>

<!-- p6 -->
# 2. Lists

<!-- p7 -->
## 2.1 Bulleted

<!-- p8 ListBullet bullet -->
- Apples — a pomaceous fruit

<!-- p9 ListBullet bullet -->
- Bourbon — distilled in Kentucky

<!-- p10 ListBullet bullet -->
- Cardamom — a fragrant spice

<!-- p11 -->
## 2.2 Numbered (with nested)

<!-- p12 ListNumber num "1." -->
1. Preheat the oven to 375°F.

<!-- p13 ListNumber num "2." -->
2. Whisk dry ingredients.

   <!-- p14 ListNumber bullet -->
   - Sift the flour twice.

   <!-- p15 ListNumber bullet -->
   - Add a pinch of salt.

<!-- p16 ListNumber num "3." -->
3. Bake for 22 minutes.

<!-- p17 -->
# 3. Tabular Data

<!-- p18 -->
A small table with a header row:

<!-- t0 5x4, cells p19-p38 by row -->
|Region|Q1|Q2|Q3|
|-|-|-|-|
|North|$120k|$135k|$150k|
|South|$98k|$110k|$128k|
|East|$143k|$151k|$162k|
|West|$87k|$92k|$104k|

<!-- p39 empty -->

<!-- p40 -->
# 4. Quote & Link

<!-- p41 Quote -->
_“Not all those who wander are lost.” — a line often quoted to test block-quote handling._

<!-- p42 -->
More information available at our [reference page](https://example.com/reference).

<!-- p43 -->
This sentence carries a second footnote for testing multi-note extraction. †[^2]

<!-- p44 -->
# 5. Color Callouts

<!-- p45 -->
**WARNING: Do not feed the extraction tool after midnight.**

<!-- p46 -->
**INFO: Tables, lists, and footnotes should each round-trip cleanly.**

<!-- p47 center -->
_— end of sample —_

<!-- p48 Title break-ins r3 @ASR -->
# {++The Pot-Pourri Compendium++}{>>r4 @ASR<<}

<!-- p49 Subtitle break-ins r5 @ASR -->
{++A Sampler Document for Extraction Testing++}{>>r6 @ASR<<}

<!-- p50 break-ins r7 @ASR -->
{++This document is intentionally varied. It mixes headings, body copy, inline formatting, colors, lists, a table, footnotes, a hyperlink, and a block quote so an extraction tool has plenty to chew on. **_Emphasis lives here. See note._**[^3]++}{>>r8 @ASR<<}

<!-- p51 break-ins r11 @ASR -->
# {++1. Typography Samples++}{>>r12 @ASR<<}

<!-- p52 break-ins r13 @ASR -->
{++Below are inline runs with mixed formatting. Each run is a separate span so an extractor can detect boundaries.++}{>>r14 @ASR<<}

<!-- p53 break-ins r15 @ASR -->
{++**Bold blue,** **_italic red,_** **_<u>underlined green,&#32;</u>_**<b><i><u><mark>yellow-highlighted,&#32;</mark></u></i></b>**_<u><mark>~~strikethrough,&#32;~~</mark></u>_**<b><i><u><mark>~~`monospace.`~~</mark></u></i></b>++}{>>r16 @ASR<<}

<!-- p54 break-ins r17 @ASR -->
# {++2. Lists++}{>>r18 @ASR<<}

<!-- p55 break-ins r19 @ASR -->
## {++2.1 Bulleted++}{>>r20 @ASR<<}

<!-- p56 ListBullet bullet, break-ins r21 @ASR -->
- {++Apples — a pomaceous fruit++}{>>r22 @ASR<<}

<!-- p57 ListBullet bullet, break-ins r23 @ASR -->
- {++Bourbon — distilled in Kentucky++}{>>r24 @ASR<<}

<!-- p58 ListBullet bullet, break-ins r25 @ASR -->
- {++Cardamom — a fragrant spice++}{>>r26 @ASR<<}

<!-- p59 break-ins r27 @ASR -->
## {++2.2 Numbered (with nested)++}{>>r28 @ASR<<}

<!-- p60 ListNumber num "4.", break-ins r29 @ASR -->
4. {++Preheat the oven to 375°F.++}{>>r30 @ASR<<}

<!-- p61 ListNumber num "5.", break-ins r31 @ASR -->
5. {++Whisk dry ingredients.++}{>>r32 @ASR<<}

   <!-- p62 ListNumber bullet, break-ins r33 @ASR -->
   - {++Sift the flour twice.++}{>>r34 @ASR<<}

   <!-- p63 ListNumber bullet, break-ins r35 @ASR -->
   - {++Add a pinch of salt.++}{>>r36 @ASR<<}

<!-- p64 ListNumber num "6.", break-ins r37 @ASR -->
6. {++Bake for 22 minutes.++}{>>r38 @ASR<<}

<!-- p65 break-ins r39 @ASR -->
# {++3. Tabular Data++}{>>r40 @ASR<<}

<!-- p66 break-ins r41 @ASR -->
{++A small table with a header row:++}{>>r42 @ASR<<}

<!-- t1 5x4, cells p67-p86 by row, break-ins r44 @ASR in p67; r46 @ASR in p68; r48 @ASR in p69; r50 @ASR in p70; r53 @ASR in p71; r55 @ASR in p72; r57 @ASR in p73; r59 @ASR in p74; r62 @ASR in p75; r64 @ASR in p76; r66 @ASR in p77; r68 @ASR in p78; r71 @ASR in p79; r73 @ASR in p80; r75 @ASR in p81; r77 @ASR in p82; r80 @ASR in p83; r82 @ASR in p84; r84 @ASR in p85; r86 @ASR in p86, rev r43 @ASR in t1.row0; r52 @ASR in t1.row1; r61 @ASR in t1.row2; r70 @ASR in t1.row3; r79 @ASR in t1.row4 -->
|{++Region++}{>>r45 @ASR<<}|{++Q1++}{>>r47 @ASR<<}|{++Q2++}{>>r49 @ASR<<}|{++Q3++}{>>r51 @ASR<<}|
|-|-|-|-|
|{++North++}{>>r54 @ASR<<}|{++$120k++}{>>r56 @ASR<<}|{++$135k++}{>>r58 @ASR<<}|{++$150k++}{>>r60 @ASR<<}|
|{++South++}{>>r63 @ASR<<}|{++$98k++}{>>r65 @ASR<<}|{++$110k++}{>>r67 @ASR<<}|{++$128k++}{>>r69 @ASR<<}|
|{++East++}{>>r72 @ASR<<}|{++$143k++}{>>r74 @ASR<<}|{++$151k++}{>>r76 @ASR<<}|{++$162k++}{>>r78 @ASR<<}|
|{++West++}{>>r81 @ASR<<}|{++$87k++}{>>r83 @ASR<<}|{++$92k++}{>>r85 @ASR<<}|{++$104k++}{>>r87 @ASR<<}|

<!-- p87 empty, break-ins r88 @ASR -->

<!-- p88 break-ins r89 @ASR -->
# {++4. Quote & Link++}{>>r90 @ASR<<}

<!-- p89 Quote break-ins r91 @ASR -->
{++_“Not all those who wander are lost.” — a line often quoted to test block-quote handling._++}{>>r92 @ASR<<}

<!-- p90 break-ins r93 @ASR -->
{++More information available at our [reference page](https://example.com/reference).++}{>>r94 @ASR<<}

<!-- p91 break-ins r95 @ASR -->
{++This sentence carries a second footnote for testing multi-note extraction. †[^4]++}{>>r96 @ASR<<}

<!-- p92 break-ins r99 @ASR -->
# {++5. Color Callouts++}{>>r100 @ASR<<}

<!-- p93 break-ins r101 @ASR -->
{++**WARNING: Do not feed the extraction tool after midnight.**++}{>>r102 @ASR<<}

<!-- p94 break-ins r103 @ASR -->
{++**INFO: Tables, lists, and footnotes should each round-trip cleanly.**++}{>>r104 @ASR<<}

<!-- p95 center, break-ins r105 @ASR -->
{++_— end of sample —_++}{>>r106 @ASR<<}

<!-- p96 empty, break-ins r107 @ASR -->

<!-- p97 Title -->
# The Pot-Pourri Compendium

<!-- p98 Subtitle -->
A Sampler Document for Extraction Testing

<!-- p99 -->
This document is intentionally {--varied. It mixes headings, body copy, inline formatting, colors, lists, a table, footnotes, a hyperlink, and a --}{>>r108 @ASR<<}block quote so an extraction tool has plenty to chew on. **_Emphasis lives here. See note._**[^5]

<!-- p100 -->
# 1. Typography Samples

<!-- p101 -->
Below are inline runs with mixed formatting. Each run is a separate span so an extractor can detect boundaries.

<!-- p102 -->
**Bold blue,** **_italic red,_** **_<u>underlined green,&#32;</u>_**<b><i><u><mark>yellow-highlighted,&#32;</mark></u></i></b>**_<u><mark>~~strikethrough,&#32;~~</mark></u>_**<b><i><u><mark>~~`monospace.`~~</mark></u></i></b>

<!-- p103 -->
# 2. Lists

<!-- p104 -->
## 2.1 Bulleted

<!-- p105 ListBullet bullet -->
- Apples — a pomaceous fruit

<!-- p106 ListBullet bullet -->
- Bourbon — distilled in Kentucky

<!-- p107 ListBullet bullet -->
- Cardamom — a fragrant spice

<!-- p108 -->
## 2.2 Numbered (with nested)

<!-- p109 ListNumber num "7." -->
7. Preheat the oven to 375°F.

<!-- p110 ListNumber num "8." -->
8. Whisk dry ingredients.

   <!-- p111 ListNumber bullet -->
   - Sift the flour twice.

   <!-- p112 ListNumber bullet -->
   - Add a pinch of salt.

<!-- p113 ListNumber num "9.", break-del r109 @ASR -->
9. {--Bake for 22 minutes.--}{>>r110 @ASR<<}

<!-- p114 break-del r111 @ASR -->
# {--3. Tabular Data--}{>>r112 @ASR<<}

<!-- p115 break-del r113 @ASR -->
{--A small table with a header row:--}{>>r114 @ASR<<}

<!-- t2 5x4, cells p116-p135 by row, break-del r116 @ASR in p116; r118 @ASR in p117; r120 @ASR in p118; r122 @ASR in p119; r125 @ASR in p120; r127 @ASR in p121; r129 @ASR in p122; r131 @ASR in p123; r134 @ASR in p124; r136 @ASR in p125; r138 @ASR in p126; r140 @ASR in p127; r143 @ASR in p128; r145 @ASR in p129; r147 @ASR in p130; r149 @ASR in p131; r152 @ASR in p132; r154 @ASR in p133; r156 @ASR in p134; r158 @ASR in p135, rev-del r115 @ASR in t2.row0; r124 @ASR in t2.row1; r133 @ASR in t2.row2; r142 @ASR in t2.row3; r151 @ASR in t2.row4 -->
|{--Region--}{>>r117 @ASR<<}|{--Q1--}{>>r119 @ASR<<}|{--Q2--}{>>r121 @ASR<<}|{--Q3--}{>>r123 @ASR<<}|
|-|-|-|-|
|{--North--}{>>r126 @ASR<<}|{--$120k--}{>>r128 @ASR<<}|{--$135k--}{>>r130 @ASR<<}|{--$150k--}{>>r132 @ASR<<}|
|{--South--}{>>r135 @ASR<<}|{--$98k--}{>>r137 @ASR<<}|{--$110k--}{>>r139 @ASR<<}|{--$128k--}{>>r141 @ASR<<}|
|{--East--}{>>r144 @ASR<<}|{--$143k--}{>>r146 @ASR<<}|{--$151k--}{>>r148 @ASR<<}|{--$162k--}{>>r150 @ASR<<}|
|{--West--}{>>r153 @ASR<<}|{--$87k--}{>>r155 @ASR<<}|{--$92k--}{>>r157 @ASR<<}|{--$104k--}{>>r159 @ASR<<}|

<!-- p136 empty, break-del r160 @ASR -->

<!-- p137 break-del r161 @ASR -->
# {--4. Quote & Link--}{>>r162 @ASR<<}

<!-- p138 Quote break-del r163 @ASR -->
{--_“Not all those who wander are lost.” — a line often quoted to test block-quote handling._--}{>>r164 @ASR<<}

<!-- p139 break-del r165 @ASR -->
{--More information available at our [reference page](https://example.com/reference).--}{>>r166 @ASR<<}

<!-- p140 break-del r167 @ASR -->
{--This sentence carries a second footnote for testing multi-note extraction. †[^6]--}{>>r168 @ASR<<}

<!-- p141 break-del r171 @ASR -->
# {--5. Color Callouts--}{>>r172 @ASR<<}

<!-- p142 break-del r173 @ASR -->
{--**WARNING: Do not feed the extraction tool after midnight.**--}{>>r174 @ASR<<}

<!-- p143 break-del r175 @ASR -->
{--**INFO: Tables, lists, and footnotes should each round-trip cleanly.**--}{>>r176 @ASR<<}

<!-- p144 center, break-del r177 @ASR -->
{--_— end of sample —_--}{>>r178 @ASR<<}

<!-- p145 empty -->

<!-- footnotes.p0 -->
[^1]: Footnote one: this footnote is attached to the introductory paragraph.
<!-- footnotes.p1 -->
[^2]: Footnote two: cross-reference target for extraction tools that map markers to notes.
<!-- footnotes.p2 -->
{++[^3]: Footnote one: this footnote is attached to the introductory paragraph.++}{>>r8 @ASR<<} <!-- rev footnotes.r9 @ASR; footnotes.r10 @ASR -->
<!-- footnotes.p3 -->
{++[^4]: Footnote two: cross-reference target for extraction tools that map markers to notes.++}{>>r96 @ASR<<} <!-- rev footnotes.r97 @ASR; footnotes.r98 @ASR -->
<!-- footnotes.p4 -->
[^5]: Footnote one: this footnote is attached to the introductory paragraph.
<!-- footnotes.p5 -->
{--[^6]: Footnote two: cross-reference target for extraction tools that map markers to notes.--}{>>r168 @ASR<<} <!-- rev-del footnotes.r169 @ASR; footnotes.r170 @ASR -->

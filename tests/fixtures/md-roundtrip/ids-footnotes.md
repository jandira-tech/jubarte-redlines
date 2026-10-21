---
view: tracked                      # revisions as CriticMarkup, comments inline
track_changes: on                  # w:trackRevisions set; edits are tracked
revisions: 178                     # 172 marks shown as 172 changes: 48 insertions, 32 deletions, 82 paragraph marks, 10 table rows; 6 in footnotes
comments: 1 thread open, style Normal  # 3 comments: c2 (+ replies c3, c4)
authors:
  document_owner: Arthur Souza Rodrigues  # dc:creator
  AS: Arthur Souza Rodrigues       # 178 revisions, 3 comments, 2026-05-02..2026-07-10
body: p0-p146, 3 tables, 5 pages   # pages from Word's cached layout
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

<!-- p0 Title break-ins r0 @AS -->
# {++file_170.docx++}{>>r1 @AS<<}

<!-- p1 Title -->
# The Pot-Pourri Compendium

<!-- p2 Subtitle -->
A Sampler Document for Extraction Testing

<!-- p3 -->
This document is intentionally varied{==. It mixes headings, body copy, inline formatting, colors, lists, a table, footnotes, a hyperlink, and a block quote so an extraction tool has plenty to chew on. **_Emphasis lives here._** ==}{>>c2 @AS: Testing.<<}{>>c3 @AS re c2: Nested 1.<<}{>>c4 @AS re c2: Nested 2.<<}**_See note._**[^1]

<!-- p4 -->
# 1. Typography Samples

<!-- p5 -->
Below are inline runs with mixed formatting. Each run is a separate span so an extractor can detect boundaries.

<!-- p6 -->
**Bold blue,** **_italic red,_** **_<u>underlined green,&#32;</u>_**<b><i><u><mark>yellow-highlighted,&#32;</mark></u></i></b>**_<u><mark>~~strikethrough,&#32;~~</mark></u>_**<b><i><u><mark>~~`monospace.`~~</mark></u></i></b>

<!-- p7 -->
# 2. Lists

<!-- p8 -->
## 2.1 Bulleted

<!-- p9 ListBullet bullet -->
- Apples — a pomaceous fruit

<!-- p10 ListBullet bullet -->
- Bourbon — distilled in Kentucky

<!-- p11 ListBullet bullet -->
- Cardamom — a fragrant spice

<!-- p12 -->
## 2.2 Numbered (with nested)

<!-- p13 ListNumber num "1." -->
1. Preheat the oven to 375°F.

<!-- p14 ListNumber num "2." -->
2. Whisk dry ingredients.

   <!-- p15 ListNumber bullet -->
   - Sift the flour twice.

   <!-- p16 ListNumber bullet -->
   - Add a pinch of salt.

<!-- p17 ListNumber num "3." -->
3. Bake for 22 minutes.

<!-- p18 -->
# 3. Tabular Data

<!-- p19 -->
A small table with a header row:

<!-- t0 5x4, cells p20-p39 by row -->
|Region|Q1|Q2|Q3|
|-|-|-|-|
|North|$120k|$135k|$150k|
|South|$98k|$110k|$128k|
|East|$143k|$151k|$162k|
|West|$87k|$92k|$104k|

<!-- p40 empty -->

<!-- p41 -->
# 4. Quote & Link

<!-- p42 Quote -->
_“Not all those who wander are lost.” — a line often quoted to test block-quote handling._

<!-- p43 -->
More information available at our [reference page](https://example.com/reference).

<!-- p44 -->
This sentence carries a second footnote for testing multi-note extraction. †[^2]

<!-- p45 -->
# 5. Color Callouts

<!-- p46 -->
**WARNING: Do not feed the extraction tool after midnight.**

<!-- p47 -->
**INFO: Tables, lists, and footnotes should each round-trip cleanly.**

<!-- p48 center -->
_— end of sample —_

<!-- p49 Title break-ins r5 @AS -->
# {++The Pot-Pourri Compendium++}{>>r6 @AS<<}

<!-- p50 Subtitle break-ins r7 @AS -->
{++A Sampler Document for Extraction Testing++}{>>r8 @AS<<}

<!-- p51 break-ins r9 @AS -->
{++This document is intentionally varied. It mixes headings, body copy, inline formatting, colors, lists, a table, footnotes, a hyperlink, and a block quote so an extraction tool has plenty to chew on. **_Emphasis lives here. See note._**[^3]++}{>>r10 @AS<<}

<!-- p52 break-ins r13 @AS -->
# {++1. Typography Samples++}{>>r14 @AS<<}

<!-- p53 break-ins r15 @AS -->
{++Below are inline runs with mixed formatting. Each run is a separate span so an extractor can detect boundaries.++}{>>r16 @AS<<}

<!-- p54 break-ins r17 @AS -->
{++**Bold blue,** **_italic red,_** **_<u>underlined green,&#32;</u>_**<b><i><u><mark>yellow-highlighted,&#32;</mark></u></i></b>**_<u><mark>~~strikethrough,&#32;~~</mark></u>_**<b><i><u><mark>~~`monospace.`~~</mark></u></i></b>++}{>>r18 @AS<<}

<!-- p55 break-ins r19 @AS -->
# {++2. Lists++}{>>r20 @AS<<}

<!-- p56 break-ins r21 @AS -->
## {++2.1 Bulleted++}{>>r22 @AS<<}

<!-- p57 ListBullet bullet, break-ins r23 @AS -->
- {++Apples — a pomaceous fruit++}{>>r24 @AS<<}

<!-- p58 ListBullet bullet, break-ins r25 @AS -->
- {++Bourbon — distilled in Kentucky++}{>>r26 @AS<<}

<!-- p59 ListBullet bullet, break-ins r27 @AS -->
- {++Cardamom — a fragrant spice++}{>>r28 @AS<<}

<!-- p60 break-ins r29 @AS -->
## {++2.2 Numbered (with nested)++}{>>r30 @AS<<}

<!-- p61 ListNumber num "4.", break-ins r31 @AS -->
4. {++Preheat the oven to 375°F.++}{>>r32 @AS<<}

<!-- p62 ListNumber num "5.", break-ins r33 @AS -->
5. {++Whisk dry ingredients.++}{>>r34 @AS<<}

   <!-- p63 ListNumber bullet, break-ins r35 @AS -->
   - {++Sift the flour twice.++}{>>r36 @AS<<}

   <!-- p64 ListNumber bullet, break-ins r37 @AS -->
   - {++Add a pinch of salt.++}{>>r38 @AS<<}

<!-- p65 ListNumber num "6.", break-ins r39 @AS -->
6. {++Bake for 22 minutes.++}{>>r40 @AS<<}

<!-- p66 break-ins r41 @AS -->
# {++3. Tabular Data++}{>>r42 @AS<<}

<!-- p67 break-ins r43 @AS -->
{++A small table with a header row:++}{>>r44 @AS<<}

<!-- t1 5x4, cells p68-p87 by row, break-ins r46 @AS in p68; r48 @AS in p69; r50 @AS in p70; r52 @AS in p71; r55 @AS in p72; r57 @AS in p73; r59 @AS in p74; r61 @AS in p75; r64 @AS in p76; r66 @AS in p77; r68 @AS in p78; r70 @AS in p79; r73 @AS in p80; r75 @AS in p81; r77 @AS in p82; r79 @AS in p83; r82 @AS in p84; r84 @AS in p85; r86 @AS in p86; r88 @AS in p87, rev r45 @AS in t1.row0; r54 @AS in t1.row1; r63 @AS in t1.row2; r72 @AS in t1.row3; r81 @AS in t1.row4 -->
|{++Region++}{>>r47 @AS<<}|{++Q1++}{>>r49 @AS<<}|{++Q2++}{>>r51 @AS<<}|{++Q3++}{>>r53 @AS<<}|
|-|-|-|-|
|{++North++}{>>r56 @AS<<}|{++$120k++}{>>r58 @AS<<}|{++$135k++}{>>r60 @AS<<}|{++$150k++}{>>r62 @AS<<}|
|{++South++}{>>r65 @AS<<}|{++$98k++}{>>r67 @AS<<}|{++$110k++}{>>r69 @AS<<}|{++$128k++}{>>r71 @AS<<}|
|{++East++}{>>r74 @AS<<}|{++$143k++}{>>r76 @AS<<}|{++$151k++}{>>r78 @AS<<}|{++$162k++}{>>r80 @AS<<}|
|{++West++}{>>r83 @AS<<}|{++$87k++}{>>r85 @AS<<}|{++$92k++}{>>r87 @AS<<}|{++$104k++}{>>r89 @AS<<}|

<!-- p88 empty, break-ins r90 @AS -->

<!-- p89 break-ins r91 @AS -->
# {++4. Quote & Link++}{>>r92 @AS<<}

<!-- p90 Quote break-ins r93 @AS -->
{++_“Not all those who wander are lost.” — a line often quoted to test block-quote handling._++}{>>r94 @AS<<}

<!-- p91 break-ins r95 @AS -->
{++More information available at our [reference page](https://example.com/reference).++}{>>r96 @AS<<}

<!-- p92 break-ins r97 @AS -->
{++This sentence carries a second footnote for testing multi-note extraction. †[^4]++}{>>r98 @AS<<}

<!-- p93 break-ins r101 @AS -->
# {++5. Color Callouts++}{>>r102 @AS<<}

<!-- p94 break-ins r103 @AS -->
{++**WARNING: Do not feed the extraction tool after midnight.**++}{>>r104 @AS<<}

<!-- p95 break-ins r105 @AS -->
{++**INFO: Tables, lists, and footnotes should each round-trip cleanly.**++}{>>r106 @AS<<}

<!-- p96 center, break-ins r107 @AS -->
{++_— end of sample —_++}{>>r108 @AS<<}

<!-- p97 empty, break-ins r109 @AS -->

<!-- p98 Title -->
# The Pot-Pourri Compendium

<!-- p99 Subtitle -->
A Sampler Document for Extraction Testing

<!-- p100 -->
This document is intentionally {--varied. It mixes headings, body copy, inline formatting, colors, lists, a table, footnotes, a hyperlink, and a --}{>>r110 @AS<<}block quote so an extraction tool has plenty to chew on. **_Emphasis lives here. See note._**[^5]

<!-- p101 -->
# 1. Typography Samples

<!-- p102 -->
Below are inline runs with mixed formatting. Each run is a separate span so an extractor can detect boundaries.

<!-- p103 -->
**Bold blue,** **_italic red,_** **_<u>underlined green,&#32;</u>_**<b><i><u><mark>yellow-highlighted,&#32;</mark></u></i></b>**_<u><mark>~~strikethrough,&#32;~~</mark></u>_**<b><i><u><mark>~~`monospace.`~~</mark></u></i></b>

<!-- p104 -->
# 2. Lists

<!-- p105 -->
## 2.1 Bulleted

<!-- p106 ListBullet bullet -->
- Apples — a pomaceous fruit

<!-- p107 ListBullet bullet -->
- Bourbon — distilled in Kentucky

<!-- p108 ListBullet bullet -->
- Cardamom — a fragrant spice

<!-- p109 -->
## 2.2 Numbered (with nested)

<!-- p110 ListNumber num "7." -->
7. Preheat the oven to 375°F.

<!-- p111 ListNumber num "8." -->
8. Whisk dry ingredients.

   <!-- p112 ListNumber bullet -->
   - Sift the flour twice.

   <!-- p113 ListNumber bullet -->
   - Add a pinch of salt.

<!-- p114 ListNumber num "9.", break-del r111 @AS -->
9. {--Bake for 22 minutes.--}{>>r112 @AS<<}

<!-- p115 break-del r113 @AS -->
# {--3. Tabular Data--}{>>r114 @AS<<}

<!-- p116 break-del r115 @AS -->
{--A small table with a header row:--}{>>r116 @AS<<}

<!-- t2 5x4, cells p117-p136 by row, break-del r118 @AS in p117; r120 @AS in p118; r122 @AS in p119; r124 @AS in p120; r127 @AS in p121; r129 @AS in p122; r131 @AS in p123; r133 @AS in p124; r136 @AS in p125; r138 @AS in p126; r140 @AS in p127; r142 @AS in p128; r145 @AS in p129; r147 @AS in p130; r149 @AS in p131; r151 @AS in p132; r154 @AS in p133; r156 @AS in p134; r158 @AS in p135; r160 @AS in p136, rev-del r117 @AS in t2.row0; r126 @AS in t2.row1; r135 @AS in t2.row2; r144 @AS in t2.row3; r153 @AS in t2.row4 -->
|{--Region--}{>>r119 @AS<<}|{--Q1--}{>>r121 @AS<<}|{--Q2--}{>>r123 @AS<<}|{--Q3--}{>>r125 @AS<<}|
|-|-|-|-|
|{--North--}{>>r128 @AS<<}|{--$120k--}{>>r130 @AS<<}|{--$135k--}{>>r132 @AS<<}|{--$150k--}{>>r134 @AS<<}|
|{--South--}{>>r137 @AS<<}|{--$98k--}{>>r139 @AS<<}|{--$110k--}{>>r141 @AS<<}|{--$128k--}{>>r143 @AS<<}|
|{--East--}{>>r146 @AS<<}|{--$143k--}{>>r148 @AS<<}|{--$151k--}{>>r150 @AS<<}|{--$162k--}{>>r152 @AS<<}|
|{--West--}{>>r155 @AS<<}|{--$87k--}{>>r157 @AS<<}|{--$92k--}{>>r159 @AS<<}|{--$104k--}{>>r161 @AS<<}|

<!-- p137 empty, break-del r162 @AS -->

<!-- p138 break-del r163 @AS -->
# {--4. Quote & Link--}{>>r164 @AS<<}

<!-- p139 Quote break-del r165 @AS -->
{--_“Not all those who wander are lost.” — a line often quoted to test block-quote handling._--}{>>r166 @AS<<}

<!-- p140 break-del r167 @AS -->
{--More information available at our [reference page](https://example.com/reference).--}{>>r168 @AS<<}

<!-- p141 break-del r169 @AS -->
{--This sentence carries a second footnote for testing multi-note extraction. †[^6]--}{>>r170 @AS<<}

<!-- p142 break-del r173 @AS -->
# {--5. Color Callouts--}{>>r174 @AS<<}

<!-- p143 break-del r175 @AS -->
{--**WARNING: Do not feed the extraction tool after midnight.**--}{>>r176 @AS<<}

<!-- p144 break-del r177 @AS -->
{--**INFO: Tables, lists, and footnotes should each round-trip cleanly.**--}{>>r178 @AS<<}

<!-- p145 center, break-del r179 @AS -->
{--_— end of sample —_--}{>>r180 @AS<<}

<!-- p146 empty -->

<!-- footnotes.p0 -->
[^1]: Footnote one: this footnote is attached to the introductory paragraph.
<!-- footnotes.p1 -->
[^2]: Footnote two: cross-reference target for extraction tools that map markers to notes.
<!-- footnotes.p2 -->
{++[^3]: Footnote one: this footnote is attached to the introductory paragraph.++}{>>r10 @AS<<} <!-- rev footnotes.r11 @AS; footnotes.r12 @AS -->
<!-- footnotes.p3 -->
{++[^4]: Footnote two: cross-reference target for extraction tools that map markers to notes.++}{>>r98 @AS<<} <!-- rev footnotes.r99 @AS; footnotes.r100 @AS -->
<!-- footnotes.p4 -->
[^5]: Footnote one: this footnote is attached to the introductory paragraph.
<!-- footnotes.p5 -->
{--[^6]: Footnote two: cross-reference target for extraction tools that map markers to notes.--}{>>r170 @AS<<} <!-- rev-del footnotes.r171 @AS; footnotes.r172 @AS -->

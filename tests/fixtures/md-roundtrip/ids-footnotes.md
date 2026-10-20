---
view: tracked                      # revisions as CriticMarkup, comments inline
track_changes: on                  # w:trackRevisions set; edits are tracked
revisions: 178                     # 172 marks shown as 93 changes: 48 insertions, 32 deletions, 3 paragraph marks, 10 table rows; 3 in footnotes (6 marks)
comments: 1 thread open, style Normal  # 3 comments: c2 (+ replies c3, c4)
authors:
  document_owner: Arthur Souza Rodrigues  # dc:creator
  AS: Arthur Souza Rodrigues       # 178 revisions, 3 comments, latest 2026-07-10T21:32:00-04:00
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

<!-- p0 Title -->
# {++file_170.docx++}{r1+0 @AS}¶{r1+0 @AS}

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

<!-- p49 Title -->
# {++The Pot-Pourri Compendium++}{r6+5 @AS}¶{r6+5 @AS}

<!-- p50 Subtitle -->
{++A Sampler Document for Extraction Testing++}{r8+7 @AS}¶{r8+7 @AS}

<!-- p51 -->
{++This document is intentionally varied. It mixes headings, body copy, inline formatting, colors, lists, a table, footnotes, a hyperlink, and a block quote so an extraction tool has plenty to chew on. **_Emphasis lives here. See note._**[^3]++}{r10+9 @AS}¶{r10+9 @AS}

<!-- p52 -->
# {++1. Typography Samples++}{r14+13 @AS}¶{r14+13 @AS}

<!-- p53 -->
{++Below are inline runs with mixed formatting. Each run is a separate span so an extractor can detect boundaries.++}{r16+15 @AS}¶{r16+15 @AS}

<!-- p54 -->
{++**Bold blue,** **_italic red,_** **_<u>underlined green,&#32;</u>_**<b><i><u><mark>yellow-highlighted,&#32;</mark></u></i></b>**_<u><mark>~~strikethrough,&#32;~~</mark></u>_**<b><i><u><mark>~~`monospace.`~~</mark></u></i></b>++}{r18+17 @AS}¶{r18+17 @AS}

<!-- p55 -->
# {++2. Lists++}{r20+19 @AS}¶{r20+19 @AS}

<!-- p56 -->
## {++2.1 Bulleted++}{r22+21 @AS}¶{r22+21 @AS}

<!-- p57 ListBullet bullet -->
- {++Apples — a pomaceous fruit++}{r24+23 @AS}¶{r24+23 @AS}

<!-- p58 ListBullet bullet -->
- {++Bourbon — distilled in Kentucky++}{r26+25 @AS}¶{r26+25 @AS}

<!-- p59 ListBullet bullet -->
- {++Cardamom — a fragrant spice++}{r28+27 @AS}¶{r28+27 @AS}

<!-- p60 -->
## {++2.2 Numbered (with nested)++}{r30+29 @AS}¶{r30+29 @AS}

<!-- p61 ListNumber num "4." -->
4. {++Preheat the oven to 375°F.++}{r32+31 @AS}¶{r32+31 @AS}

<!-- p62 ListNumber num "5." -->
5. {++Whisk dry ingredients.++}{r34+33 @AS}¶{r34+33 @AS}

   <!-- p63 ListNumber bullet -->
   - {++Sift the flour twice.++}{r36+35 @AS}¶{r36+35 @AS}

   <!-- p64 ListNumber bullet -->
   - {++Add a pinch of salt.++}{r38+37 @AS}¶{r38+37 @AS}

<!-- p65 ListNumber num "6." -->
6. {++Bake for 22 minutes.++}{r40+39 @AS}¶{r40+39 @AS}

<!-- p66 -->
# {++3. Tabular Data++}{r42+41 @AS}¶{r42+41 @AS}

<!-- p67 -->
{++A small table with a header row:++}{r44+43 @AS}¶{r44+43 @AS}

<!-- t1 5x4, cells p68-p87 by row, break-ins r47+46 @AS in p68; r49+48 @AS in p69; r51+50 @AS in p70; r53+52 @AS in p71; r56+55 @AS in p72; r58+57 @AS in p73; r60+59 @AS in p74; r62+61 @AS in p75; r65+64 @AS in p76; r67+66 @AS in p77; r69+68 @AS in p78; r71+70 @AS in p79; r74+73 @AS in p80; r76+75 @AS in p81; r78+77 @AS in p82; r80+79 @AS in p83; r83+82 @AS in p84; r85+84 @AS in p85; r87+86 @AS in p86; r89+88 @AS in p87, rev r45 @AS in t1.row0; r54 @AS in t1.row1; r63 @AS in t1.row2; r72 @AS in t1.row3; r81 @AS in t1.row4 -->
|{++Region++}{r47+46 @AS}|{++Q1++}{r49+48 @AS}|{++Q2++}{r51+50 @AS}|{++Q3++}{r53+52 @AS}|
|-|-|-|-|
|{++North++}{r56+55 @AS}|{++$120k++}{r58+57 @AS}|{++$135k++}{r60+59 @AS}|{++$150k++}{r62+61 @AS}|
|{++South++}{r65+64 @AS}|{++$98k++}{r67+66 @AS}|{++$110k++}{r69+68 @AS}|{++$128k++}{r71+70 @AS}|
|{++East++}{r74+73 @AS}|{++$143k++}{r76+75 @AS}|{++$151k++}{r78+77 @AS}|{++$162k++}{r80+79 @AS}|
|{++West++}{r83+82 @AS}|{++$87k++}{r85+84 @AS}|{++$92k++}{r87+86 @AS}|{++$104k++}{r89+88 @AS}|

<!-- p88 -->
¶{r90 @AS}

<!-- p89 -->
# {++4. Quote & Link++}{r92+91 @AS}¶{r92+91 @AS}

<!-- p90 Quote -->
{++_“Not all those who wander are lost.” — a line often quoted to test block-quote handling._++}{r94+93 @AS}¶{r94+93 @AS}

<!-- p91 -->
{++More information available at our [reference page](https://example.com/reference).++}{r96+95 @AS}¶{r96+95 @AS}

<!-- p92 -->
{++This sentence carries a second footnote for testing multi-note extraction. †[^4]++}{r98+97 @AS}¶{r98+97 @AS}

<!-- p93 -->
# {++5. Color Callouts++}{r102+101 @AS}¶{r102+101 @AS}

<!-- p94 -->
{++**WARNING: Do not feed the extraction tool after midnight.**++}{r104+103 @AS}¶{r104+103 @AS}

<!-- p95 -->
{++**INFO: Tables, lists, and footnotes should each round-trip cleanly.**++}{r106+105 @AS}¶{r106+105 @AS}

<!-- p96 center -->
{++_— end of sample —_++}{r108+107 @AS}¶{r108+107 @AS}

<!-- p97 -->
¶{r109 @AS}

<!-- p98 Title -->
# The Pot-Pourri Compendium

<!-- p99 Subtitle -->
A Sampler Document for Extraction Testing

<!-- p100 -->
This document is intentionally {--varied. It mixes headings, body copy, inline formatting, colors, lists, a table, footnotes, a hyperlink, and a --}{r110 @AS}block quote so an extraction tool has plenty to chew on. **_Emphasis lives here. See note._**[^5]

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

<!-- p114 ListNumber num "9.", break-del r112+111 @AS -->
9. {--Bake for 22 minutes.--}{r112+111 @AS}

<!-- p115 break-del r114+113 @AS -->
# {--3. Tabular Data--}{r114+113 @AS}

<!-- p116 break-del r116+115 @AS -->
{--A small table with a header row:--}{r116+115 @AS}

<!-- t2 5x4, cells p117-p136 by row, break-del r119+118 @AS in p117; r121+120 @AS in p118; r123+122 @AS in p119; r125+124 @AS in p120; r128+127 @AS in p121; r130+129 @AS in p122; r132+131 @AS in p123; r134+133 @AS in p124; r137+136 @AS in p125; r139+138 @AS in p126; r141+140 @AS in p127; r143+142 @AS in p128; r146+145 @AS in p129; r148+147 @AS in p130; r150+149 @AS in p131; r152+151 @AS in p132; r155+154 @AS in p133; r157+156 @AS in p134; r159+158 @AS in p135; r161+160 @AS in p136, rev-del r117 @AS in t2.row0; r126 @AS in t2.row1; r135 @AS in t2.row2; r144 @AS in t2.row3; r153 @AS in t2.row4 -->
|{--Region--}{r119+118 @AS}|{--Q1--}{r121+120 @AS}|{--Q2--}{r123+122 @AS}|{--Q3--}{r125+124 @AS}|
|-|-|-|-|
|{--North--}{r128+127 @AS}|{--$120k--}{r130+129 @AS}|{--$135k--}{r132+131 @AS}|{--$150k--}{r134+133 @AS}|
|{--South--}{r137+136 @AS}|{--$98k--}{r139+138 @AS}|{--$110k--}{r141+140 @AS}|{--$128k--}{r143+142 @AS}|
|{--East--}{r146+145 @AS}|{--$143k--}{r148+147 @AS}|{--$151k--}{r150+149 @AS}|{--$162k--}{r152+151 @AS}|
|{--West--}{r155+154 @AS}|{--$87k--}{r157+156 @AS}|{--$92k--}{r159+158 @AS}|{--$104k--}{r161+160 @AS}|

<!-- p137 empty, break-del r162 @AS -->

<!-- p138 break-del r164+163 @AS -->
# {--4. Quote & Link--}{r164+163 @AS}

<!-- p139 Quote break-del r166+165 @AS -->
{--_“Not all those who wander are lost.” — a line often quoted to test block-quote handling._--}{r166+165 @AS}

<!-- p140 break-del r168+167 @AS -->
{--More information available at our [reference page](https://example.com/reference).--}{r168+167 @AS}

<!-- p141 break-del r170+169 @AS -->
{--This sentence carries a second footnote for testing multi-note extraction. †[^6]--}{r170+169 @AS}

<!-- p142 break-del r174+173 @AS -->
# {--5. Color Callouts--}{r174+173 @AS}

<!-- p143 break-del r176+175 @AS -->
{--**WARNING: Do not feed the extraction tool after midnight.**--}{r176+175 @AS}

<!-- p144 break-del r178+177 @AS -->
{--**INFO: Tables, lists, and footnotes should each round-trip cleanly.**--}{r178+177 @AS}

<!-- p145 center, break-del r180+179 @AS -->
{--_— end of sample —_--}{r180+179 @AS}

<!-- p146 empty -->

<!-- footnotes.p0 -->
[^1]: Footnote one: this footnote is attached to the introductory paragraph.
<!-- footnotes.p1 -->
[^2]: Footnote two: cross-reference target for extraction tools that map markers to notes.
<!-- footnotes.p2 -->
{++[^3]: Footnote one: this footnote is attached to the introductory paragraph.++}{r10+9 @AS} <!-- rev footnotes.r12+11 @AS -->
<!-- footnotes.p3 -->
{++[^4]: Footnote two: cross-reference target for extraction tools that map markers to notes.++}{r98+97 @AS} <!-- rev footnotes.r100+99 @AS -->
<!-- footnotes.p4 -->
[^5]: Footnote one: this footnote is attached to the introductory paragraph.
<!-- footnotes.p5 -->
{--[^6]: Footnote two: cross-reference target for extraction tools that map markers to notes.--}{r170+169 @AS} <!-- rev-del footnotes.r172+171 @AS -->

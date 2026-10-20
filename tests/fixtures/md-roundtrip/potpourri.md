---
view: tracked                      # revisions as CriticMarkup, comments inline
track_changes: on                  # w:trackRevisions set; edits are tracked
revisions: 176                     # 170 marks shown as 92 changes: 47 insertions, 32 deletions, 3 paragraph marks, 10 table rows; 6 in footnotes
comments: 0
authors:
  document_owner: Arthur Souza Rodrigues  # dc:creator
  ASR: Arthur Souza Rodrigues      # 176 revisions, latest 2026-05-02T14:42:00-04:00
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

<!-- p48 Title -->
# {++The Pot-Pourri Compendium++}{r4+3 @ASR}¶{r4+3 @ASR}

<!-- p49 Subtitle -->
{++A Sampler Document for Extraction Testing++}{r6+5 @ASR}¶{r6+5 @ASR}

<!-- p50 -->
{++This document is intentionally varied. It mixes headings, body copy, inline formatting, colors, lists, a table, footnotes, a hyperlink, and a block quote so an extraction tool has plenty to chew on. **_Emphasis lives here. See note._**[^3]++}{r8+7 @ASR}¶{r8+7 @ASR}

<!-- p51 -->
# {++1. Typography Samples++}{r12+11 @ASR}¶{r12+11 @ASR}

<!-- p52 -->
{++Below are inline runs with mixed formatting. Each run is a separate span so an extractor can detect boundaries.++}{r14+13 @ASR}¶{r14+13 @ASR}

<!-- p53 -->
{++**Bold blue,** **_italic red,_** **_<u>underlined green,&#32;</u>_**<b><i><u><mark>yellow-highlighted,&#32;</mark></u></i></b>**_<u><mark>~~strikethrough,&#32;~~</mark></u>_**<b><i><u><mark>~~`monospace.`~~</mark></u></i></b>++}{r16+15 @ASR}¶{r16+15 @ASR}

<!-- p54 -->
# {++2. Lists++}{r18+17 @ASR}¶{r18+17 @ASR}

<!-- p55 -->
## {++2.1 Bulleted++}{r20+19 @ASR}¶{r20+19 @ASR}

<!-- p56 ListBullet bullet -->
- {++Apples — a pomaceous fruit++}{r22+21 @ASR}¶{r22+21 @ASR}

<!-- p57 ListBullet bullet -->
- {++Bourbon — distilled in Kentucky++}{r24+23 @ASR}¶{r24+23 @ASR}

<!-- p58 ListBullet bullet -->
- {++Cardamom — a fragrant spice++}{r26+25 @ASR}¶{r26+25 @ASR}

<!-- p59 -->
## {++2.2 Numbered (with nested)++}{r28+27 @ASR}¶{r28+27 @ASR}

<!-- p60 ListNumber num "4." -->
4. {++Preheat the oven to 375°F.++}{r30+29 @ASR}¶{r30+29 @ASR}

<!-- p61 ListNumber num "5." -->
5. {++Whisk dry ingredients.++}{r32+31 @ASR}¶{r32+31 @ASR}

   <!-- p62 ListNumber bullet -->
   - {++Sift the flour twice.++}{r34+33 @ASR}¶{r34+33 @ASR}

   <!-- p63 ListNumber bullet -->
   - {++Add a pinch of salt.++}{r36+35 @ASR}¶{r36+35 @ASR}

<!-- p64 ListNumber num "6." -->
6. {++Bake for 22 minutes.++}{r38+37 @ASR}¶{r38+37 @ASR}

<!-- p65 -->
# {++3. Tabular Data++}{r40+39 @ASR}¶{r40+39 @ASR}

<!-- p66 -->
{++A small table with a header row:++}{r42+41 @ASR}¶{r42+41 @ASR}

<!-- t1 5x4, cells p67-p86 by row, break-ins r45+44 @ASR in p67; r47+46 @ASR in p68; r49+48 @ASR in p69; r51+50 @ASR in p70; r54+53 @ASR in p71; r56+55 @ASR in p72; r58+57 @ASR in p73; r60+59 @ASR in p74; r63+62 @ASR in p75; r65+64 @ASR in p76; r67+66 @ASR in p77; r69+68 @ASR in p78; r72+71 @ASR in p79; r74+73 @ASR in p80; r76+75 @ASR in p81; r78+77 @ASR in p82; r81+80 @ASR in p83; r83+82 @ASR in p84; r85+84 @ASR in p85; r87+86 @ASR in p86, rev r43 @ASR in t1.row0; r52 @ASR in t1.row1; r61 @ASR in t1.row2; r70 @ASR in t1.row3; r79 @ASR in t1.row4 -->
|{++Region++}{r45+44 @ASR}|{++Q1++}{r47+46 @ASR}|{++Q2++}{r49+48 @ASR}|{++Q3++}{r51+50 @ASR}|
|-|-|-|-|
|{++North++}{r54+53 @ASR}|{++$120k++}{r56+55 @ASR}|{++$135k++}{r58+57 @ASR}|{++$150k++}{r60+59 @ASR}|
|{++South++}{r63+62 @ASR}|{++$98k++}{r65+64 @ASR}|{++$110k++}{r67+66 @ASR}|{++$128k++}{r69+68 @ASR}|
|{++East++}{r72+71 @ASR}|{++$143k++}{r74+73 @ASR}|{++$151k++}{r76+75 @ASR}|{++$162k++}{r78+77 @ASR}|
|{++West++}{r81+80 @ASR}|{++$87k++}{r83+82 @ASR}|{++$92k++}{r85+84 @ASR}|{++$104k++}{r87+86 @ASR}|

<!-- p87 -->
¶{r88 @ASR}

<!-- p88 -->
# {++4. Quote & Link++}{r90+89 @ASR}¶{r90+89 @ASR}

<!-- p89 Quote -->
{++_“Not all those who wander are lost.” — a line often quoted to test block-quote handling._++}{r92+91 @ASR}¶{r92+91 @ASR}

<!-- p90 -->
{++More information available at our [reference page](https://example.com/reference).++}{r94+93 @ASR}¶{r94+93 @ASR}

<!-- p91 -->
{++This sentence carries a second footnote for testing multi-note extraction. †[^4]++}{r96+95 @ASR}¶{r96+95 @ASR}

<!-- p92 -->
# {++5. Color Callouts++}{r100+99 @ASR}¶{r100+99 @ASR}

<!-- p93 -->
{++**WARNING: Do not feed the extraction tool after midnight.**++}{r102+101 @ASR}¶{r102+101 @ASR}

<!-- p94 -->
{++**INFO: Tables, lists, and footnotes should each round-trip cleanly.**++}{r104+103 @ASR}¶{r104+103 @ASR}

<!-- p95 center -->
{++_— end of sample —_++}{r106+105 @ASR}¶{r106+105 @ASR}

<!-- p96 -->
¶{r107 @ASR}

<!-- p97 Title -->
# The Pot-Pourri Compendium

<!-- p98 Subtitle -->
A Sampler Document for Extraction Testing

<!-- p99 -->
This document is intentionally {--varied. It mixes headings, body copy, inline formatting, colors, lists, a table, footnotes, a hyperlink, and a --}{r108 @ASR}block quote so an extraction tool has plenty to chew on. **_Emphasis lives here. See note._**[^5]

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

<!-- p113 ListNumber num "9.", break-del r110+109 @ASR -->
9. {--Bake for 22 minutes.--}{r110+109 @ASR}

<!-- p114 break-del r112+111 @ASR -->
# {--3. Tabular Data--}{r112+111 @ASR}

<!-- p115 break-del r114+113 @ASR -->
{--A small table with a header row:--}{r114+113 @ASR}

<!-- t2 5x4, cells p116-p135 by row, break-del r117+116 @ASR in p116; r119+118 @ASR in p117; r121+120 @ASR in p118; r123+122 @ASR in p119; r126+125 @ASR in p120; r128+127 @ASR in p121; r130+129 @ASR in p122; r132+131 @ASR in p123; r135+134 @ASR in p124; r137+136 @ASR in p125; r139+138 @ASR in p126; r141+140 @ASR in p127; r144+143 @ASR in p128; r146+145 @ASR in p129; r148+147 @ASR in p130; r150+149 @ASR in p131; r153+152 @ASR in p132; r155+154 @ASR in p133; r157+156 @ASR in p134; r159+158 @ASR in p135, rev-del r115 @ASR in t2.row0; r124 @ASR in t2.row1; r133 @ASR in t2.row2; r142 @ASR in t2.row3; r151 @ASR in t2.row4 -->
|{--Region--}{r117+116 @ASR}|{--Q1--}{r119+118 @ASR}|{--Q2--}{r121+120 @ASR}|{--Q3--}{r123+122 @ASR}|
|-|-|-|-|
|{--North--}{r126+125 @ASR}|{--$120k--}{r128+127 @ASR}|{--$135k--}{r130+129 @ASR}|{--$150k--}{r132+131 @ASR}|
|{--South--}{r135+134 @ASR}|{--$98k--}{r137+136 @ASR}|{--$110k--}{r139+138 @ASR}|{--$128k--}{r141+140 @ASR}|
|{--East--}{r144+143 @ASR}|{--$143k--}{r146+145 @ASR}|{--$151k--}{r148+147 @ASR}|{--$162k--}{r150+149 @ASR}|
|{--West--}{r153+152 @ASR}|{--$87k--}{r155+154 @ASR}|{--$92k--}{r157+156 @ASR}|{--$104k--}{r159+158 @ASR}|

<!-- p136 empty, break-del r160 @ASR -->

<!-- p137 break-del r162+161 @ASR -->
# {--4. Quote & Link--}{r162+161 @ASR}

<!-- p138 Quote break-del r164+163 @ASR -->
{--_“Not all those who wander are lost.” — a line often quoted to test block-quote handling._--}{r164+163 @ASR}

<!-- p139 break-del r166+165 @ASR -->
{--More information available at our [reference page](https://example.com/reference).--}{r166+165 @ASR}

<!-- p140 break-del r168+167 @ASR -->
{--This sentence carries a second footnote for testing multi-note extraction. †[^6]--}{r168+167 @ASR}

<!-- p141 break-del r172+171 @ASR -->
# {--5. Color Callouts--}{r172+171 @ASR}

<!-- p142 break-del r174+173 @ASR -->
{--**WARNING: Do not feed the extraction tool after midnight.**--}{r174+173 @ASR}

<!-- p143 break-del r176+175 @ASR -->
{--**INFO: Tables, lists, and footnotes should each round-trip cleanly.**--}{r176+175 @ASR}

<!-- p144 center, break-del r178+177 @ASR -->
{--_— end of sample —_--}{r178+177 @ASR}

<!-- p145 empty -->

<!-- footnotes.p0 -->
[^1]: Footnote one: this footnote is attached to the introductory paragraph.
<!-- footnotes.p1 -->
[^2]: Footnote two: cross-reference target for extraction tools that map markers to notes.
<!-- footnotes.p2 -->
{++[^3]: Footnote one: this footnote is attached to the introductory paragraph.++}{r8+7 @ASR} <!-- rev footnotes.r10+9 @ASR -->
<!-- footnotes.p3 -->
{++[^4]: Footnote two: cross-reference target for extraction tools that map markers to notes.++}{r96+95 @ASR} <!-- rev footnotes.r98+97 @ASR -->
<!-- footnotes.p4 -->
[^5]: Footnote one: this footnote is attached to the introductory paragraph.
<!-- footnotes.p5 -->
{--[^6]: Footnote two: cross-reference target for extraction tools that map markers to notes.--}{r168+167 @ASR} <!-- rev-del footnotes.r170+169 @ASR -->

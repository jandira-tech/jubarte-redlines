---
view: tracked                      # revisions as CriticMarkup, comments inline
track_changes: off                 # w:trackRevisions not set; new edits are not tracked unless edit sets it
revisions: 0
comments: 0
authors:
  document_owner: Un-named         # dc:creator
body: p0-p68, 1 table, 2 pages     # pages from layout
page: A4 portrait, margins 1in, header/footer 0.492in
styles:
  "1": Arial 12pt, left, named Normal  # default; unannotated paragraphs use it
  "#": 2, Calibri bold 16pt, before 12pt, after 6pt, named heading 1
  "##": 3, Calibri bold 14pt, before 9pt, after 4.5pt, named heading 2
  "13": Arial 28pt, named Title
  "17": Arial bold 12pt, before 6pt, after 6pt, named Highlighted Style
  "14": Arial 12pt, named List Paragraph
  table: 9, named Normal Table
---

<!-- p0 13 -->
# Comprehensive DOCX Features Demonstration

<!-- p1 2 -->
# 1. Inline Text Formatting

<!-- p2 -->
This section demonstrates various inline text formatting options.

<!-- p3 3 -->
## 1.1 Basic Formatting

<!-- p4 -->
Normal text, **bold text**, _italic text_, **_bold and italic_**.

<!-- p5 -->
<u>Single underline</u>, <u>double underline</u>, <u>wavy underline</u>.

<!-- p6 -->
~~Strikethrough text~~, <mark>highlighted text</mark>, SMALL CAPS TEXT.

<!-- p7 -->
Mathematical notation: x<sup>2</sup> + y = H<sub>2</sub>O

<!-- p8 2 -->
# 2. Font Types and Sizes

<!-- p9 3 -->
## 2.1 Different Font Types

<!-- p10 -->
Arial font (default sans-serif)

<!-- p11 -->
Times New Roman font (classic serif)

<!-- p12 -->
Calibri font (modern sans-serif)

<!-- p13 -->
Georgia font (elegant serif)

<!-- p14 -->
Verdana font (screen-optimized)

<!-- p15 -->
`Courier New font (monospace)`

<!-- p16 3 -->
## 2.2 Different Font Sizes

<!-- p17 -->
8pt text (very small)

<!-- p18 -->
10pt text (small)

<!-- p19 -->
12pt text (standard body)

<!-- p20 -->
14pt text (slightly larger)

<!-- p21 -->
18pt text (subheading size)

<!-- p22 -->
24pt text (heading size)

<!-- p23 2 -->
# 3. Text Colors

<!-- p24 -->
**Red text** | **Blue text** | **Green text** | **Orange text**

<!-- p25 -->
**Purple text** | **Teal text** | **Brown text** | **Gray text**

<!-- p26 -->
<mark>Yellow highlight</mark> | <mark class="hl-green">Green highlight</mark> | <mark class="hl-cyan">Cyan highlight</mark>

<!-- p27 2 -->
# 4. Styles from styles.xml

<!-- p28 -->
The following paragraphs use styles defined in the styles section above:

<!-- p29 17 -->
**This paragraph uses the 'CustomHighlight' style with yellow background.**

<!-- p30 -->
This sentence has **red bold character style** and _blue italic character style_ applied from styles.xml.

<!-- p31 2 -->
# 5. Lists

<!-- p32 3 -->
## 5.1 Bullet List

<!-- p33 14 bullet -->
- First bullet point - demonstrating proper bullet formatting

<!-- p34 14 bullet -->
- Second bullet point - uses LevelFormat.BULLET constant

<!-- p35 14 bullet -->
- Third bullet point - not unicode symbols

<!-- p36 3 -->
## 5.2 Numbered List

<!-- p37 14 num "1." -->
1. First numbered item - uses decimal numbering

<!-- p38 14 num "2." -->
2. Second numbered item - continues sequence

<!-- p39 14 num "3." -->
3. Third numbered item - proper Word numbering

<!-- p40 2 -->
# 6. Tables

<!-- p41 -->
Below is a table demonstrating borders, shading, and cell formatting:

<!-- t0 4x3, cells p42-p53 by row, p42 center, p43 center, p44 center, header row repeats, borders single 0.5pt, row-borders single 0.5pt in t0.row0 t0.row1 t0.row2 t0.row3, cell-borders top,left,bottom,right single 0pt 000000 in t0.row0; top,left,bottom,right single 0pt CCCCCC in t0.row1 t0.row2 t0.row3 -->
|**Feature**|**Description**|**Example**|
|-|-|-|
|Bold|Makes text bold|**Bold text**|
|Italic|Makes text italic|_Italic text_|
|Color|Changes text color|Colored text|

<!-- p54 2 -->
# 7. Text Alignment

<!-- p55 left -->
This paragraph is left-aligned (default).

<!-- p56 center -->
This paragraph is center-aligned.

<!-- p57 right -->
This paragraph is right-aligned.

<!-- p58 justify -->
This paragraph is justified. When there is enough text in a justified paragraph, it will stretch to fill the entire width of the page from left to right margin, creating clean edges on both sides.

<!-- p59 2 -->
# Summary

<!-- p60 -->
This document demonstrates all major DOCX features:

<!-- p61 14 bullet -->
- Inline formatting: bold, italic, underline, strikethrough, highlights

<!-- p62 14 bullet -->
- Multiple font types: Arial, Times New Roman, Calibri, Georgia, Verdana, Courier New

<!-- p63 14 bullet -->
- Font sizes: from 8pt to 24pt

<!-- p64 14 bullet -->
- Text colors and highlights

<!-- p65 14 bullet -->
- Style-based formatting using styles.xml references

<!-- p66 14 bullet -->
- Bullet and numbered lists

<!-- p67 14 bullet -->
- Tables with borders, shading, and formatted cells

<!-- p68 14 bullet -->
- Text alignment options

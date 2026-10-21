---
view: tracked                      # revisions as CriticMarkup, comments inline
track_changes: off                 # w:trackRevisions not set; new edits are not tracked unless edit sets it
revisions: 10                      # 4 marks shown as 3 changes: 2 table rows, 1 table cell; 6 formatting
comments: 3 threads open           # 3 comments: c0, c1, c2
authors:
  document_owner: none             # no docProps/core.xml
  AL: Ann Lee                      # 9 revisions, 2026-03-02T10:00:00Z
  B: Bo Chen                       # 3 comments, 2026-03-02T10:00:00Z
body: p0-p21, 5 tables, 1 page     # pages from layout
page: Letter portrait, margins 1in, header/footer 0.5in
styles:
  Normal: Calibri 11pt, after 8pt, line 1.08, left  # default; unannotated paragraphs use it
  table: TableNormal
unplaced: [ins r109 @AL in p20]    # changes no other line shows (text boxes, Word's Fallback copies)
---

<!-- p0 center, fmt r100 @AL; r101 @AL; r102 @AL -->
**Three** _format changes._

<!-- p1 -->
Changed table:

<!-- t0 2x2, cells p2-p5 by row, p4 center, rev r103 @AL in t0.row0; r104 @AL in t0.row1.cell1, fmt r106 @AL; r107 @??; r105 @AL in p4 -->
|||
|-|-|
|a2|{++b2++}{>>r104 @AL<<}|

<!-- p6 -->
Inserted row:

<!-- t1 2x2, cells p7-p10 by row -->
|x0|y0|
|-|-|
|{++x1++}{>>r108 @AL<<}|{++y1++}{>>r108 @AL<<}|

<!-- p11 -->
Cell to cell:

<!-- t2 2x2, cells p12-p15 by row -->
|{==a1==}|{==b1==}|
|-|-|
|{==a2==}|{==b2==}{>>c0 @B: From cell to cell.<<}|

<!-- t3 1x2, cells p16-p17 by row, in c1 c2 -->
|{==m1==}|{==n1==}|
|-|-|

<!-- p18 in c1 c2 -->
{==After two ranges.==}{>>c1 @B: First over the table.<<}{>>c2 @B: Second over the table.<<}

<!-- t4 1x2, cells p19-p20 by row -->
|e1|
|-|

<!-- p21 -->
End.

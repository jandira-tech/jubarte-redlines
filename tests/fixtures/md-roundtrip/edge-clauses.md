---
view: tracked                      # revisions as CriticMarkup, comments inline
track_changes: off                 # w:trackRevisions not set; new edits are not tracked unless edit sets it
revisions: 8                       # 2 marks shown as 2 changes: 1 insertion, 1 paragraph mark; 6 formatting
comments: 2 threads open           # 2 comments: c0, c1
authors:
  document_owner: none             # no docProps/core.xml
  AL: Ann Lee                      # 8 revisions, 2026-03-02T10:00:00Z
  B: Bo Chen                       # 2 comments, 2026-03-02T10:00:00Z
body: p0-p13, 1 table, 1 page      # pages from layout
page: Letter portrait, margins 1in, header/footer 0.5in; fmt r17 @AL
styles:
  Normal: Calibri 11pt, after 8pt, line 1.08, left  # default; unannotated paragraphs use it
  table: TableNormal
---

<!-- p0 center, fmt r10 @AL -->
A paragraph whose alignment changed.

<!-- p1 fmt r11 @AL -->
Bold **after a bold change.**

<!-- p2 -->
Whitespace inserted:{++ ++}{>>r13 @AL<<}unshown.

<!-- p3 break-ins r12 @AL -->
A paragraph mark inserted.

<!-- p4 in c0 -->
{==First paragraph of a comment range.==}

<!-- p5 in c0 -->
{==Second paragraph of the range.==}

<!-- t0 2x3, cells p6-p11 by row, fmt r16 @AL; r14 @AL in t0.row0; r15 @AL in t0.row1.cell0, in c0 -->
|{==a1==}|{==b1==}|{==c1==}|
|-|-|-|
|{==a2==}|{==b2==}|{==c2==}|

<!-- p12 in c0 -->
{==Last paragraph of the range.==}{>>c0 @B: Spans paragraphs and a table.<<}

<!-- p13 -->
Outside: {==inside==}{>>c1 @B: Inline.<<} outside again.

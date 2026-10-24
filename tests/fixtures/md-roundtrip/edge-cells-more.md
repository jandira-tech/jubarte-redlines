---
view: tracked                      # revisions as CriticMarkup, comments inline
track_changes: off                 # w:trackRevisions not set; new edits are not tracked unless edit sets it
revisions: 0
comments: 0
authors:
  document_owner: none             # no docProps/core.xml
body: p0-p26, 4 tables, 1 page     # pages from layout
page: Letter portrait, margins 1in, header/footer 0.5in
styles:
  Normal: Calibri 11pt, after 8pt, line 1.08, left  # default; unannotated paragraphs use it
  table: TableNormal
---

<!-- p0 -->
Short:

<!-- t0 3x3, cells p1-p8 by row, gaps t0.row1 after 1 -->
|a|b|c|
|-|-|-|
|d|e||
|g|h|i|

<!-- p9 -->
Past:

<!-- t1 2x4, cells p10-p15 by row, gaps t1.row0 after 1; t1.row1 after 1 -->
|j|k|l|
|-|-|-|
|m|n|o|

<!-- p16 -->
Merged:

<!-- t2 2x2, cells p17-p20 by row, merged t2.row0.cell1 down 2 -->
|q|
|-|
|s|

<!-- p21 -->
Empty:

<!-- t3 2x2, cells p22-p25 by row -->
||
|-|
||

<!-- p26 -->
After.

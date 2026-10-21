---
view: tracked                      # revisions as CriticMarkup, comments inline
track_changes: off                 # w:trackRevisions not set; new edits are not tracked unless edit sets it
revisions: 0
comments: 0
authors:
  document_owner: none             # no docProps/core.xml
body: p0-p12, 3 tables, 1 page     # pages from layout
page: Letter portrait, margins 1in, header/footer 0.5in
styles:
  Normal: Calibri 11pt, after 8pt, line 1.08, left  # default; unannotated paragraphs use it
  tables: TableNormal, Ledger, borders vary
---

<!-- p0 -->
No style:

<!-- t0 1x3, cells p1-p3 by row, style TableNormal -->
|x|y|z|
|-|-|-|

<!-- p4 -->
Table Grid:

<!-- t1 1x3, cells p5-p7 by row -->
|x|y|z|
|-|-|-|

<!-- p8 -->
Custom:

<!-- t2 1x3, cells p9-p11 by row, style Ledger -->
|x|y|z|
|-|-|-|

<!-- p12 -->
After.

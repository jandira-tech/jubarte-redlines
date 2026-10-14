---
view: tracked                      # revisions as CriticMarkup, comments inline
track_changes: off                 # w:trackRevisions not set; new edits are not tracked unless edit sets it
revisions: 7                       # 7 marks shown as 3 changes: 1 insertion, 1 deletion, 1 substitution
comments: 1 thread open            # 1 comment: c0
authors:
  document_owner: none             # no docProps/core.xml
  AL: Ann Lee                      # 7 revisions, 2026-03-02T10:00:00Z
  B: Bo Chen                       # 1 comment, 2026-03-02T10:00:00Z
body: p0-p2, 0 tables, 1 page      # pages from layout
page: Letter portrait, margins 1in, header/footer 0.5in
styles:
  Normal: Calibri 11pt, after 8pt, line 1.08, left  # default; unannotated paragraphs use it
---

<!-- p0 -->
Inside {--cut {==by a==}{>>c0 @B: Inside the deletion.<<} comment--}{>>r40 @AL<<} after.

<!-- p1 -->
Sub {~~plain _italic_~>x~~}{>>r5+6+7 @AL<<} end.

<!-- p2 rev r9 @AL; r10 @AL -->
Empty {++a++}{>>r8 @AL<<} end.

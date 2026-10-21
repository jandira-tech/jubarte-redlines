---
view: tracked                      # revisions as CriticMarkup, comments inline
track_changes: off                 # w:trackRevisions not set; new edits are not tracked unless edit sets it
revisions: 12                      # 12 marks shown as 7 changes: 2 insertions, 2 deletions, 3 substitutions
comments: 1 thread open            # 1 comment: c0
authors:
  document_owner: none             # no docProps/core.xml
  AL: Ann Lee                      # 10 revisions, 2026-03-02T10:00:00Z
  EF: Eve Fox                      # 2 revisions, 2026-03-02..2026-03-04
  B: Bo Chen                       # 1 comment, 2026-03-02T10:00:00Z
body: p0-p4, 0 tables, 1 page      # pages from layout
page: Letter portrait, margins 1in, header/footer 0.5in
styles:
  Normal: Calibri 11pt, after 8pt, line 1.08, left  # default; unannotated paragraphs use it
---

<!-- p0 -->
Link: {~~[old](https://example.com/)~>[new](https://example.com/)~~}{>>r81+80 @AL<<}.

<!-- p1 -->
Mark: {++new++}{>>r82 @AL<<}<a id="mark"></a>{--old--}{>>r83 @AL<<} [go](#mark).

<!-- p2 -->
Range: {++new++}{>>r84 @AL<<}{=={--old--}{>>r85 @AL<<}==}{>>c0 @B: On the deletion.<<}.

<!-- p3 -->
Runs: {~~CC DD~>AA BB ~~}{>>r88+89+86+87 @AL<<}.

<!-- p4 -->
Days: {~~old~>new~~}{>>r91+90 @EF<<}.

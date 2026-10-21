---
view: tracked                      # revisions as CriticMarkup, comments inline
track_changes: off                 # w:trackRevisions not set; new edits are not tracked unless edit sets it
revisions: 9                       # 9 marks shown as 8 changes: 3 insertions, 4 deletions, 1 paragraph mark
comments: 1 thread open            # 1 comment: c0
authors:
  document_owner: none             # no docProps/core.xml
  AL: Ann Lee                      # 9 revisions, 2026-03-02T10:00:00Z
  B: Bo Chen                       # 1 comment, 2026-03-02T10:00:00Z
body: p0-p4, 0 tables, 1 page      # pages from layout
page: Letter portrait, margins 1in, header/footer 0.5in
styles:
  Normal: Calibri 11pt, after 8pt, line 1.08, left  # default; unannotated paragraphs use it
---

<!-- p0 -->
Before {--cut --}{>>r40 @AL<<}{=={--by a--}{>>r41 @AL<<}==}{>>c0 @B: Cuts the deletion.<<}{-- comment--}{>>r42 @AL<<} after.

<!-- p1 -->
Joined: {--plain _italic_--}{>>r43+44 @AL<<} end.

<!-- p2 -->
Link: {++[site](https://example.com/)++}{>>r45 @AL<<}.

<!-- p3 break-ins r47 @AL -->
Across {++two++}{>>r46 @AL<<}

<!-- p4 -->
{++paragraphs++}{>>r48 @AL<<} here.

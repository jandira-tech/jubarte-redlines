---
view: tracked                      # revisions as CriticMarkup, comments inline
track_changes: off                 # w:trackRevisions not set; new edits are not tracked unless edit sets it
revisions: 12                      # 12 marks shown as 5 changes: 5 substitutions
comments: 1 thread open            # 1 comment: c0
authors:
  document_owner: none             # no docProps/core.xml
  AL: Ann Lee                      # 10 revisions, 2026-03-02T10:00:00Z
  EF: Eve Fox                      # 2 revisions, latest 2026-03-04T16:30:00Z
  B: Bo Chen                       # 1 comment, 2026-03-02T10:00:00Z
body: p0-p4, 0 tables, 1 page      # pages from layout
page: Letter portrait, margins 1in, header/footer 0.5in
styles:
  Normal: Calibri 11pt, after 8pt, line 1.08, left  # default; unannotated paragraphs use it
---

<!-- p0 -->
Link: {++[new](https://example.com/)++}{r80+81 @AL}{--[old](https://example.com/)--}{r80+81 @AL}.

<!-- p1 -->
Mark: {++new++}{r82+83 @AL}<a id="mark"></a>{--old--}{r82+83 @AL} [go](#mark).

<!-- p2 -->
Range: {++new++}{r84+85 @AL}{=={--old--}{r84+85 @AL}==}{>>c0 @B: On the deletion.<<}.

<!-- p3 -->
Runs: {++AA ++}{r86+87+88+89 @AL}{++BB ++}{r86+87+88+89 @AL}{--CC --}{r86+87+88+89 @AL}{--DD--}{r86+87+88+89 @AL}.

<!-- p4 -->
Days: {++new++}{r90+91 @EF}{--old--}{r90+91 @EF}.

---
view: tracked                      # revisions as CriticMarkup, comments inline
track_changes: off                 # w:trackRevisions not set; new edits are not tracked unless edit sets it
revisions: 12                      # 1 mark shown as 1 change: 1 insertion; 7 in header1; 4 in footer1
comments: 0
authors:
  document_owner: none             # docProps/core.xml names no dc:creator or cp:lastModifiedBy
  ??: ""                           # 2 revisions, 1970-01-01T00:00:00Z
  AL: Ann Lee                      # 8 revisions, 2026-03-02T10:00:00Z
  BC: Bo Chen                      # 2 revisions, 2026-03-02T10:00:00Z
body: p0-p0, 0 tables, 1 page      # pages from layout
page: Letter portrait, margins 1in, header/footer 0.5in
styles:
  Normal: Calibri 11pt, after 8pt, line 1.08, left  # default; unannotated paragraphs use it
headers:
  default: {id: header1, text: "Sub {~~old~>new~~}{>>header1.r91+90 @AL<<}. Runs {~~CD~>AB~~}{>>header1.r94+95+92+93 @AL<<}. Keep {++mine++}{>>header1.r3 @??<<}"}
footers:
  default: {id: footer1, text: ["Page {--{PAGE}1{/}--}{>>footer1.r31 @AL<<} \\{PAGE}", ""], rev: break-ins footer1.r60 @AL in footer1; break-del footer1.r61 @BC in footer1; break-del footer1.r40 @BC in footer1.p1}
---

<!-- p0 -->
Keep {++new++}{>>r5 @??<<} it.

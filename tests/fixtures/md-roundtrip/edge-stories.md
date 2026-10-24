---
view: tracked                      # revisions as CriticMarkup, comments inline
track_changes: off                 # w:trackRevisions not set; new edits are not tracked unless edit sets it
revisions: 6                       # 3 in header1; 3 in footer1
comments: 0
authors:
  document_owner: none             # no docProps/core.xml
  AL: Ann Lee                      # 4 revisions, 2026-03-02T10:00:00Z
  BC: Bo Chen                      # 2 revisions, 2026-03-02T10:00:00Z
body: p0-p0, 0 tables, 1 page      # pages from layout
page: Letter portrait, margins 1in, header/footer 0.5in
styles:
  Normal: Calibri 11pt, after 8pt, line 1.08, left  # default; unannotated paragraphs use it
headers:
  default: {id: header1, text: "Kept {--gone--}{>>header1.r20 @AL<<}{++added++}{>>header1.r21 @AL<<}", fmt: header1.r22 @AL in header1}
footers:
  default:
    id: footer1
    p0: {text: "Page {++{PAGE}1{/}++}{>>footer1.r23 @AL<<}", center}
    p1: {text: "{--Draft--}{>>footer1.r24 @BC<<} v2"}
    rev: break-del footer1.r25 @BC in footer1.p1
---

<!-- p0 -->
Body text.

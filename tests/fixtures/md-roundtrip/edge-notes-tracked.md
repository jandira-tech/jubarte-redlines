---
view: tracked                      # revisions as CriticMarkup, comments inline
track_changes: off                 # w:trackRevisions not set; new edits are not tracked unless edit sets it
revisions: 11                      # 3 marks shown as 3 changes: 2 insertions, 1 deletion; 8 in footnotes
comments: 0
authors:
  document_owner: none             # no docProps/core.xml
  AL: Ann Lee                      # 11 revisions, 2026-03-02T10:00:00Z
body: p0-p1, 0 tables, 1 page      # pages from layout
page: Letter portrait, margins 1in, header/footer 0.5in
styles:
  Normal: Calibri 11pt, after 8pt, line 1.08, left  # default; unannotated paragraphs use it
---

<!-- p0 -->
Added{++[^1]++}{>>r10 @AL<<} and cut{--[^2]--}{>>r20 @AL<<}.

<!-- p1 -->
Long{++[^3]++}{>>r30 @AL<<}.

<!-- footnotes.p0 -->
{++[^1]: An inserted note.++}{>>r10 @AL<<} <!-- rev footnotes.r11 @AL; footnotes.r12 @AL -->
<!-- footnotes.p1 -->
{--[^2]: A deleted note.--}{>>r20 @AL<<} <!-- rev-del footnotes.r21 @AL; footnotes.r22 @AL -->
<!-- footnotes.p2-p3 -->
{++[^3]: Two paragraphs,<br>inserted.++}{>>r30 @AL<<} <!-- rev footnotes.r31 @AL; footnotes.r32 @AL; footnotes.r33 @AL; footnotes.r34 @AL -->

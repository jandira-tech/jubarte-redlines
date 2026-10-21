---
view: tracked                      # revisions as CriticMarkup, comments inline
track_changes: off                 # w:trackRevisions not set; new edits are not tracked unless edit sets it
revisions: 16                      # 16 marks shown as 13 changes: 5 insertions, 5 deletions, 3 substitutions
comments: 0
authors:
  document_owner: none             # no docProps/core.xml
  AL: Ann Lee                      # 13 revisions, 2026-03-02T10:00:00Z
  BC: Bo Chen                      # 3 revisions, 2026-03-02T10:00:00Z
body: p0-p5, 0 tables, 1 page      # pages from layout
page: Letter portrait, margins 1in, header/footer 0.5in
styles:
  Normal: Calibri 11pt, after 8pt, line 1.08, left  # default; unannotated paragraphs use it
---

<!-- p0 -->
Del then ins: {--old--}{>>r50 @AL<<}{++new++}{>>r51 @AL<<}.

<!-- p1 -->
Ins then del: {~~old~>new~~}{>>r53+52 @AL<<}.

<!-- p2 -->
Ins ins del del: {++one ++}{>>r54 @AL<<}{~~three ~>two ~~}{>>r56 @AL<<}{>>r55 @BC<<}{--four--}{>>r57 @BC<<}.

<!-- p3 -->
Two authors: {--hers--}{>>r58 @AL<<}{++his++}{>>r59 @BC<<}.

<!-- p4 -->
Side by side: {--a--}{>>r60 @AL<<}{~~c~>b~~}{>>r62+61 @AL<<}{++d++}{>>r63 @AL<<}.

<!-- p5 -->
Spaced: {--x--}{>>r64 @AL<<} {++y++}{>>r65 @AL<<}.

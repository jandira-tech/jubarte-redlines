---
view: tracked                      # revisions as CriticMarkup, comments inline
track_changes: on                  # w:trackRevisions set; edits are tracked
revisions: 22                      # 22 marks shown as 21 changes: 6 insertions, 8 deletions, 1 substitution, 6 paragraph marks
comments: 0
authors:
  document_owner: none             # docProps/core.xml names no dc:creator or cp:lastModifiedBy
  AL: Ana Lima                     # 7 revisions, 2026-10-01..2026-10-01
  BC: Bo Chen                      # 13 revisions, 2026-10-01..2026-10-01
  CD: Carla Duarte                 # 1 revision, 2026-10-01T00:15:00Z
  DP: Dev Patel                    # 1 revision, 2026-10-01T00:20:00Z
body: p0-p16, 0 tables, 1 page     # pages from layout
page: A4 portrait, margins 0.7875in, header/footer 0in, starts nextPage
styles:
  Normal: Liberation Serif 12pt, left  # default; unannotated paragraphs use it
  "#": Heading1, Liberation Sans bold 18pt, before 12pt, after 6pt, keep-next, named Heading 1
---

<!-- p0 left, hanging 0in, left 0in -->
# Adjacent and spanning changes

<!-- p1 Normal left -->
Back to back: {++one++}{>>r0 @AL<<}{~~three~>two~~}{>>r2 @CD<<}{>>r1 @BC<<}{--four--}{>>r3 @DP<<}.

<!-- p2 Normal left -->
{++Start of paragraph ++}{>>r4 @AL<<}middle{-- end of paragraph--}{>>r5 @BC<<}

<!-- p3 Normal left -->
Replaced by two people: {--old--}{>>r6 @AL<<}{++new++}{>>r7 @BC<<}.

<!-- p4 Normal left, break-del r9 @BC -->
Deletion across paragraphs: alpha{-- beta--}{>>r8 @BC<<}

<!-- p5 Normal left -->
{--gamma--}{>>r10 @BC<<} delta.

<!-- p6 Normal left, break-ins r12 @AL -->
Insertion across paragraphs: one{++ two++}{>>r11 @AL<<}

<!-- p7 Normal left -->
{++three++}{>>r13 @AL<<} four.

<!-- p8 Normal left -->
Before three cut paragraphs.

<!-- p9 Normal left, break-del r15 @BC -->
{--Cut one.--}{>>r14 @BC<<}

<!-- p10 Normal left, break-del r17 @BC -->
{--Cut two.--}{>>r16 @BC<<}

<!-- p11 Normal left, break-del r19 @BC -->
{--Cut three.--}{>>r18 @BC<<}

<!-- p12 Normal left -->
After them.

<!-- p13 Normal empty, left -->

<!-- p14 Normal left, break-del r20 @BC -->
An empty paragraph above

<!-- p15 Normal left -->
&#32;and a joined one.

<!-- p16 Normal left -->
Last{++ words++}{>>r21 @AL<<}

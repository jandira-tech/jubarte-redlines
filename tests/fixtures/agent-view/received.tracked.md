---
source: received.docx
view: tracked                      # revisions as CriticMarkup, comments inline
track_changes: off                 # w:trackRevisions not set; new edits are not tracked unless edit sets it
revisions: 5                       # 1 insertion, 4 substitutions (9 Word marks)
comments: 2 threads open           # 3 comments: c5 (+ reply c6), c11
authors:
  document_owner: Arthur Souza Rodrigues  # cp:lastModifiedBy; no dc:creator
  AC: Ann Counsel                  # 5 revisions, 2 comments, 2026-10-01T09:00:00Z
  AS: Arthur Souza Rodrigues       # 1 comment, 2026-10-09T16:13:00Z
body: p0-p20, 1 table, 2 pages     # pages from layout
page: Letter portrait, margins 1in, header/footer 0.5in
styles:
  Normal: Calibri 11pt, after 8pt, line 1.08, left  # default; unannotated paragraphs use it
  "#": Heading1, Calibri bold 16pt, before 12pt, after 4pt, keep-next
  "##": Heading2, Calibri bold 14pt, before 12pt, after 4pt, keep-next
  table: TableGrid, all borders 0.5pt
headers:
  first: {id: header2, text: DRAFT}  # page 1 only (different first page)
  default: {id: header1, text: SIGNATURE PAGE, right}
footers:
  first: none                      # page 1 shows no page number
  default: {id: footer2, text: "{PAGE}"}
  even: {id: footer1, text: "{PAGE}", inactive}  # defined, but even/odd headers are off
---

<!-- page 1 of 2 -->

<!-- p0 center -->
# Consulting Agreement

<!-- p1 justify -->
This Consulting Agreement (the “Agreement”) is made between Harbor Point Analytics LLC ("Consultant") and Juniper Freight Co. ("Client").

<!-- p2 -->
## 1. Services

<!-- p3 -->
Consultant will deliver the services in each Statement of Work, including {++quarterly++}{>>#0 @AC<<} route-cost reports and a {~~weekly~>monthly~~}{>>#1+2 @AC<<} review call.

<!-- p4 -->
## 2. Fees

<!-- p5 -->
Client shall pay each invoice within {~~thirty~>forty-five~~}{>>#3+4 @AC<<} days of receipt. {==Late amounts accrue interest at one percent per month.==}{>>#c5 @AC: Is one percent the statutory cap in Delaware?<<}{>>#c6 @AS re #c5: Disagree.<<}

<!-- p6 -->
## 3. Term

<!-- p7 -->
This Agreement begins on March 1, 2026 and continues until either party ends it with {~~ninety days~>sixty days~~}{>>#7+8 @AC<<} written notice.

<!-- t0 center 3x3, cells p8-p16 by row, header row repeats -->
|Deliverable|Due|Owner|
|-|-|-|
|Route-cost report|{~~Day 10~>Day 15~~}{>>#9+10 @AC<<}|Consultant|
|Review call|Monthly|Both|

<!-- p17 -->
## 4. Confidentiality

<!-- p18 first-line 0.5in -->
Each party shall keep the other's Confidential Information **<u>secret</u>**{>>#c11 @AC: Add a three-year survival period?<<}.

<!-- p19 page-break -->

<!-- page 2 of 2 -->

<!-- p20 center -->
[To be included]

---
source: input.docx
sha256: 0d36dc75661f1b85faec19957f4b5229742ddc4a9388bc5853667b8bcf989159
view: tracked                      # revisions as CriticMarkup, comments inline
track_changes: off                 # w:trackRevisions not set; new edits are not tracked unless edit sets it
revisions: 0
comments: 0
authors:
  document_owner: python-docx      # dc:creator
body: p0-p30, 1 table, 2 pages     # pages from layout
page: Letter portrait, margins top 1in, right 1.25in, bottom 1in, left 1.25in, header/footer 0.5in
styles:
  Normal: Cambria 11pt, after 10pt, line 1.15, left  # default; unannotated paragraphs use it
  "#": Heading1, Calibri bold 14pt, before 24pt, keep-next
  "##": Heading2, Calibri bold 13pt, before 10pt, keep-next
  ListBullet: Cambria 11pt, named List Bullet
  table: TableGrid, all borders 0.5pt
headers:
  default: {id: header1, text: "Jandira Technologies, LLC - Professional Services Agreement (Draft 3)"}
footers:
  default: {id: footer1, text: Confidential draft - do not circulate}
---

<!-- page 1 of 2 -->

<!-- p0 -->
# Professional Services Agreement

<!-- p1 -->
This Agreement is made between Jandira Technologies, LLC ("the Company") and Northwind Traders Ltd. ("the Client"), effective as of 2 March 2026.

<!-- p2 -->
## 1. Scope of Services

<!-- p3 -->
**Scope.** The Company will design, implement and document the document-generation pipeline described in Statement of Work 1, including acceptance tests.

<!-- p4 -->
The deliverables are:

<!-- p5 ListBullet bullet -->
- a written architecture summary

<!-- p6 ListBullet bullet -->
- the pipeline source and its tests

<!-- p7 ListBullet bullet -->
- a runbook for the operations team

<!-- p8 ListBullet bullet -->
- one training session of up to three hours

<!-- p9 -->
## 2. Fees and Payment

<!-- p10 -->
**Fees.** _Invoices are payable_ net thirty (30) days, by wire transfer to the account stated on each invoice.[^1]

<!-- p11 -->
An invoice that is not disputed within ten business days is deemed accepted.

<!-- p12 -->
Payment schedule:

<!-- t0 4x3, cells p13-p24 by row -->
|**Milestone**|**Deliverable**|**Fee (USD)**|
|-|-|-|
|M1|Architecture summary accepted|12,000|
|M2|Pipeline passes acceptance tests|30,000|
|M3|Runbook and training delivered|8,000|

<!-- p25 -->
## 3. Confidentiality

<!-- p26 -->
Each party keeps the other's non-public information secret and uses it only to perform this Agreement. The obligation survives termination by three years.

<!-- p27 -->
## 4. Termination

<!-- p28 -->
Either party may terminate for material breach on fifteen days' written notice if the breach remains uncured. The Company is paid for work performed up to the termination date.

<!-- p29 -->
Signed for the Company: \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_

<!-- page 2 of 2 -->

<!-- p30 -->
Signed for the Client:  \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_

<!-- footnotes.p0 -->
[^1]: Net thirty (30) days from the invoice date.

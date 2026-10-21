---
view: tracked                      # revisions as CriticMarkup, comments inline
track_changes: off                 # w:trackRevisions not set; new edits are not tracked unless edit sets it
revisions: 6                       # 6 marks shown as 4 changes: 1 insertion, 1 deletion, 2 substitutions
comments: 1 thread open, 1 resolved  # 3 comments: c3 (+ reply c4), c8
authors:
  document_owner: none             # docProps/core.xml names no dc:creator or cp:lastModifiedBy
  AC: Ann Counsel                  # 3 revisions, 2 comments, 2026-10-01T09:00:00Z
  JD: Jo Doe                       # 3 revisions, 1 comment, 2026-10-02T10:30:00Z
body: p0-p26, 1 table, 2 pages     # page count estimated from breaks
page: Letter portrait, margins 1in, header/footer 0.5in
styles:
  Normal: Calibri 11pt, after 8pt, line 1.08, left  # default; unannotated paragraphs use it
  "#": Heading1, Calibri bold 16pt, before 12pt, after 4pt, keep-next
  "##": Heading2, Calibri bold 14pt, before 12pt, after 4pt, keep-next
  ListParagraph: Calibri 11pt, indent 0.5in, contextual spacing, named List Paragraph
headers:
  default: {id: header1, text: Northwind / Contoso, right}
footers:
  default: {id: footer1, text: "Page {PAGE} of {NUMPAGES}"}
---

<!-- p0 center -->
# Master Services Agreement

<!-- p1 justify -->
This Master Services Agreement is made between Northwind Logistics LLC ("Provider") and Contoso Freight Inc. ("Customer"), effective as of the date in Section 4.[^1]

<!-- p2 -->
## 1. Definitions

<!-- p3 ListParagraph num "1." -->
1. "Services" means the work in each {++signed ++}{>>r0 @AC<<}Statement of Work.

<!-- p4 ListParagraph num "2." -->
2. "Deliverables" means the reports listed in Section 3.

   <!-- p5 ListParagraph bullet -->
   - Monthly route reports.

   <!-- p6 ListParagraph bullet -->
   - Quarterly cost reviews.

<!-- p7 -->
## 2. Fees

<!-- p8 first-line 0.5in -->
Customer shall pay each invoice within {~~thirty~>forty-five~~}{>>r1+2 @AC<<} days. {==Late amounts accrue interest at one percent per month.==}{>>c3 @AC: Is one percent the cap?<<}{>>c4 @JD re c3: It is in Delaware.<<}

<!-- p9 -->
Payments go to the account in [the payment portal](https://pay.example.com/northwind "Northwind payment portal"). {--Cheques are not accepted.--}{>>r5 @JD<<}

<!-- p10-p11 empty -->

<!-- p12 -->
## 3. Deliverables

<!-- t0 center 3x3, cells p13-p21 by row, header row repeats -->
|Deliverable|Due|Owner|
|-|-|-|
|Route report|Day {~~10~>15~~}{>>r6+7 @JD<<}|Provider|
|Cost review|Quarterly|_Both_|

<!-- p22 -->
## 4. Term

<!-- p23 -->
<a id="term"></a>This Agreement begins on {DATE \@ "MMMM d, yyyy"}March 1, 2026{/} and runs for <mark>two years</mark>, as Section 2 sets out (see [Fees](#term)).{>>c8 @AC resolved: Confirm the start date.<<}

<!-- p24 -->
~~Superseded terms~~ are **not** <u>incorporated</u>.[^2]

<!-- p25 page-break -->

<!-- p26 right -->
Signature page follows.

<!-- footnotes.p0 -->
[^1]: The effective date is the later of the two signatures.
<!-- endnotes.p0 -->
[^2]: Earlier drafts are withdrawn.

---
view: tracked                      # revisions as CriticMarkup, comments inline
track_changes: off                 # w:trackRevisions not set; new edits are not tracked unless edit sets it
revisions: 0
comments: 0
authors:
  document_owner: jharrop2         # dc:creator
body: p0-p4, 0 tables, 3 pages     # pages from Word's cached layout
page: A4 portrait, margins 1in, header/footer 0.49in
styles:
  Normal: Calibri 11pt, after 10pt, line 1.15, left  # default; unannotated paragraphs use it
headers:
  first: {id: header2, text: My first page header}  # page 1 only (different first page)
  default: {id: header1, text: My header}
footers:
  first: none                      # page 1 shows no page number
  default: none
sections:
  2: {p4-p4, headers: {first: {id: header3, text: Section 2 - My first page header}}, footers: {first: none}}
---

<!-- p0 -->
header-no-rels

<!-- p1 page-break -->

<!-- p2 -->
Second page

<!-- p3 section-break -->

<!-- p4 -->
Some content in the second section.... with just an empty p, this section isn't rendered?

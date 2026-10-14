# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only
"""Write the round-trip edge documents: small, hand-made docx files, one per
behaviour the Markdown view must carry both ways (docx -> md -> docx -> md and
md -> docx -> md). Run from anywhere: `python3 make_edges.py`; it rewrites the
`.docx` files beside it, byte for byte the same each time."""

import pathlib
import zipfile

HERE = pathlib.Path(__file__).resolve().parent
W = 'xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"'
REL = "http://schemas.openxmlformats.org/officeDocument/2006/relationships"
PKG = "http://schemas.openxmlformats.org/package/2006/relationships"
CT = "application/vnd.openxmlformats-officedocument.wordprocessingml"
D1 = "2026-03-02T10:00:00Z"
D2 = "2026-03-04T16:30:00Z"

STYLES = f"""<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles {W}><w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:ascii="Calibri" w:hAnsi="Calibri" w:cs="Calibri"/><w:sz w:val="22"/><w:szCs w:val="22"/></w:rPr></w:rPrDefault><w:pPrDefault><w:pPr><w:spacing w:after="160" w:line="259" w:lineRule="auto"/></w:pPr></w:pPrDefault></w:docDefaults><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:qFormat/></w:style><w:style w:type="character" w:default="1" w:styleId="DefaultParagraphFont"><w:name w:val="Default Paragraph Font"/></w:style><w:style w:type="table" w:default="1" w:styleId="TableNormal"><w:name w:val="Normal Table"/><w:tblPr><w:tblInd w:w="0" w:type="dxa"/><w:tblCellMar><w:top w:w="0" w:type="dxa"/><w:left w:w="108" w:type="dxa"/><w:bottom w:w="0" w:type="dxa"/><w:right w:w="108" w:type="dxa"/></w:tblCellMar></w:tblPr></w:style>{{extra}}</w:styles>"""

SECT = '<w:sectPr>{refs}<w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720" w:gutter="0"/>{change}</w:sectPr>'


def p(*runs, ppr=""):
    return f"<w:p>{'<w:pPr>' + ppr + '</w:pPr>' if ppr else ''}{''.join(runs)}</w:p>"


def r(text, rpr=""):
    return f'<w:r>{"<w:rPr>" + rpr + "</w:rPr>" if rpr else ""}<w:t xml:space="preserve">{text}</w:t></w:r>'


def dr(text, rpr=""):
    return f'<w:r>{"<w:rPr>" + rpr + "</w:rPr>" if rpr else ""}<w:delText xml:space="preserve">{text}</w:delText></w:r>'


def ins(i, who, body, date=D1):
    return f'<w:ins w:id="{i}" w:author="{who}" w:date="{date}">{body}</w:ins>'


def dele(i, who, body, date=D1):
    return f'<w:del w:id="{i}" w:author="{who}" w:date="{date}">{body}</w:del>'


def table(rows, tblpr='<w:tblW w:w="0" w:type="auto"/>', grid=(2400, 2400, 2400)):
    cols = "".join(f'<w:gridCol w:w="{w}"/>' for w in grid)
    return f"<w:tbl><w:tblPr>{tblpr}</w:tblPr><w:tblGrid>{cols}</w:tblGrid>{''.join(rows)}</w:tbl>"


def tr(*cells, trpr=""):
    return f"<w:tr>{'<w:trPr>' + trpr + '</w:trPr>' if trpr else ''}{''.join(cells)}</w:tr>"


def tc(*paras, tcpr='<w:tcW w:w="2400" w:type="dxa"/>'):
    return f"<w:tc><w:tcPr>{tcpr}</w:tcPr>{''.join(paras) or '<w:p/>'}</w:tc>"


def package(name, body, *, sect_refs="", sect_change="", parts=None, rels=None, types=None,
            styles_extra="", core=None, settings=""):
    """Write HERE/name: `body` inside w:body, then the final sectPr."""
    parts = dict(parts or {})
    rels = list(rels or [])
    types = list(types or [])
    document = (f'<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n<w:document {W}><w:body>{body}'
                f'{SECT.format(refs=sect_refs, change=sect_change)}</w:body></w:document>')
    parts["word/document.xml"] = document
    if "word/comments.xml" in parts:
        styles_extra += ('<w:style w:type="paragraph" w:styleId="CommentText"><w:name w:val="annotation text"/>'
                         '<w:basedOn w:val="Normal"/><w:rPr><w:sz w:val="20"/><w:szCs w:val="20"/></w:rPr></w:style>')
    parts["word/styles.xml"] = STYLES.replace("{extra}", styles_extra)
    parts["word/settings.xml"] = f'<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n<w:settings {W}>{settings}</w:settings>'
    rels = [("rIdStyles", f"{REL}/styles", "styles.xml"), ("rIdSettings", f"{REL}/settings", "settings.xml")] + rels
    types = [("/word/document.xml", f"{CT}.document.main+xml"),
             ("/word/styles.xml", f"{CT}.styles+xml"),
             ("/word/settings.xml", f"{CT}.settings+xml")] + types
    package_rels = [("rId1", f"{REL}/officeDocument", "word/document.xml")]
    if core is not None:
        parts["docProps/core.xml"] = (
            '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n<cp:coreProperties '
            'xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" '
            'xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" '
            f'xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">{core}</cp:coreProperties>')
        package_rels.append(("rId2", "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties", "docProps/core.xml"))
        types.append(("/docProps/core.xml", "application/vnd.openxmlformats-package.core-properties+xml"))
    overrides = "".join(f'<Override PartName="{n}" ContentType="{t}"/>' for n, t in types)
    parts["[Content_Types].xml"] = (
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">'
        '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>'
        f'<Default Extension="xml" ContentType="application/xml"/>{overrides}</Types>')
    parts["_rels/.rels"] = (f'<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n<Relationships xmlns="{PKG}">'
                            + "".join(f'<Relationship Id="{i}" Type="{t}" Target="{g}"/>' for i, t, g in package_rels)
                            + "</Relationships>")
    parts["word/_rels/document.xml.rels"] = (
        f'<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n<Relationships xmlns="{PKG}">'
        + "".join(f'<Relationship Id="{i}" Type="{t}" Target="{g}"{" TargetMode=\"External\"" if g.startswith("http") else ""}/>'
                  for i, t, g in rels)
        + "</Relationships>")
    order = ["[Content_Types].xml", "_rels/.rels"] + sorted(k for k in parts if k not in ("[Content_Types].xml", "_rels/.rels"))
    with zipfile.ZipFile(HERE / name, "w", zipfile.ZIP_DEFLATED) as z:
        for part in order:
            info = zipfile.ZipInfo(part, date_time=(1980, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            z.writestr(info, parts[part])


def story(kind, body):
    root = "hdr" if kind == "header" else "ftr"
    return f'<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n<w:{root} {W}>{body}</w:{root}>'


def comments(*items):
    # Word's comment paragraph: the annotation text style.
    body = "".join(f'<w:comment w:id="{i}" w:author="{who}" w:date="{D1}" w:initials="{who[0]}">'
                   f'{p(r(text), ppr=chr(60) + "w:pStyle w:val=" + chr(34) + "CommentText" + chr(34) + "/>")}</w:comment>'
                   for i, who, text in items)
    return f'<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n<w:comments {W}>{body}</w:comments>'


COMMENTS = (("rIdComments", f"{REL}/comments", "comments.xml"), ("/word/comments.xml", f"{CT}.comments+xml"))


def page_field(code="PAGE", result="1"):
    return (f'<w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> {code} </w:instrText></w:r>'
            f'<w:r><w:fldChar w:fldCharType="separate"/></w:r>{r(result)}<w:r><w:fldChar w:fldCharType="end"/></w:r>')


# 1. Revision clauses (fmt, rev) and comment ranges (in cN), on paragraphs,
#    tables, rows, cells and the section.
def edge_clauses():
    ppr_change = f'<w:pPrChange w:id="10" w:author="Ann Lee" w:date="{D1}"><w:pPr><w:jc w:val="left"/></w:pPr></w:pPrChange>'
    rpr_change = f'<w:rPrChange w:id="11" w:author="Ann Lee" w:date="{D1}"><w:rPr/></w:rPrChange>'
    mark_ins = f'<w:rPr><w:ins w:id="12" w:author="Ann Lee" w:date="{D1}"/></w:rPr>'
    body = (
        p(r("A paragraph whose alignment changed."), ppr='<w:jc w:val="center"/>' + ppr_change)
        + p(r("Bold"), r(" after a bold change.", "<w:b/>" + rpr_change))
        + p(r("Whitespace inserted:"), ins(13, "Ann Lee", r(" ")), r("unshown."))
        + p(r("A paragraph mark inserted."), ppr=mark_ins)
        + '<w:commentRangeStart w:id="0"/>'
        + p(r("First paragraph of a comment range."))
        + p(r("Second paragraph of the range."))
        + table([
            tr(tc(p(r("a1"))), tc(p(r("b1"))), tc(p(r("c1"))),
               trpr=f'<w:trPrChange w:id="14" w:author="Ann Lee" w:date="{D1}"><w:trPr/></w:trPrChange>'),
            tr(tc(p(r("a2")), tcpr=f'<w:tcW w:w="2400" w:type="dxa"/><w:tcPrChange w:id="15" w:author="Ann Lee" w:date="{D1}"><w:tcPr><w:tcW w:w="2000" w:type="dxa"/></w:tcPr></w:tcPrChange>'),
               tc(p(r("b2"))), tc(p(r("c2")))),
        ], tblpr=f'<w:tblW w:w="0" w:type="auto"/><w:tblPrChange w:id="16" w:author="Ann Lee" w:date="{D1}"><w:tblPr><w:tblW w:w="5000" w:type="pct"/></w:tblPr></w:tblPrChange>')
        + p(r("Last paragraph of the range."), '<w:commentRangeEnd w:id="0"/>',
            '<w:r><w:commentReference w:id="0"/></w:r>')
        + p(r("Outside: "), '<w:commentRangeStart w:id="1"/>', r("inside"), '<w:commentRangeEnd w:id="1"/>',
            '<w:r><w:commentReference w:id="1"/></w:r>', r(" outside again."))
    )
    sect_change = f'<w:sectPrChange w:id="17" w:author="Ann Lee" w:date="{D1}"><w:sectPr><w:pgMar w:top="1000" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720" w:gutter="0"/></w:sectPr></w:sectPrChange>'
    package("edge-clauses.docx", body, sect_change=sect_change,
            parts={"word/comments.xml": comments((0, "Bo Chen", "Spans paragraphs and a table."), (1, "Bo Chen", "Inline."))},
            rels=[COMMENTS[0]], types=[COMMENTS[1]])


# 2. Header and footer tracked changes: insertions, deletions, a deleted and
#    an inserted field, a formatting change, a paragraph-mark change, two
#    paragraphs in one footer.
def edge_stories():
    header = story("header", p(r("Kept "), dele(20, "Ann Lee", dr("gone")), ins(21, "Ann Lee", r("added")),
                               ppr='<w:pStyle w:val="Header"/>' + f'<w:pPrChange w:id="22" w:author="Ann Lee" w:date="{D1}"><w:pPr><w:pStyle w:val="Header"/><w:jc w:val="right"/></w:pPr></w:pPrChange>'))
    footer = story("footer",
                   p(r("Page "), ins(23, "Ann Lee", page_field()), ppr='<w:pStyle w:val="Footer"/><w:jc w:val="center"/>')
                   + p(dele(24, "Bo Chen", dr("Draft")), r(" v2"),
                       ppr=f'<w:pStyle w:val="Footer"/><w:rPr><w:del w:id="25" w:author="Bo Chen" w:date="{D1}"/></w:rPr>'))
    styles = ('<w:style w:type="paragraph" w:styleId="Header"><w:name w:val="header"/><w:basedOn w:val="Normal"/></w:style>'
              '<w:style w:type="paragraph" w:styleId="Footer"><w:name w:val="footer"/><w:basedOn w:val="Normal"/></w:style>')
    package("edge-stories.docx", p(r("Body text.")),
            sect_refs='<w:headerReference w:type="default" r:id="rIdH1"/><w:footerReference w:type="default" r:id="rIdF1"/>',
            parts={"word/header1.xml": header, "word/footer1.xml": footer},
            rels=[("rIdH1", f"{REL}/header", "header1.xml"), ("rIdF1", f"{REL}/footer", "footer1.xml")],
            types=[("/word/header1.xml", f"{CT}.header+xml"), ("/word/footer1.xml", f"{CT}.footer+xml")],
            styles_extra=styles)


# 3. The document owner: dc:creator; and a file that names only the last
#    editor (cp:lastModifiedBy).
def edge_owner():
    package("edge-owner-creator.docx", p(r("Owned.")),
            core="<dc:creator>Jones, SA</dc:creator><cp:lastModifiedBy>Someone Else</cp:lastModifiedBy>")
    package("edge-owner-modifier.docx", p(r("Edited.")),
            core="<cp:lastModifiedBy>Last Editor</cp:lastModifiedBy>")


# 4. Table styles: none, the built-in Table Grid, and a custom style.
def edge_table_styles():
    grid = ('<w:style w:type="table" w:styleId="TableGrid"><w:name w:val="Table Grid"/><w:basedOn w:val="TableNormal"/>'
            '<w:tblPr><w:tblBorders><w:top w:val="single" w:sz="4" w:space="0" w:color="auto"/><w:left w:val="single" w:sz="4" w:space="0" w:color="auto"/>'
            '<w:bottom w:val="single" w:sz="4" w:space="0" w:color="auto"/><w:right w:val="single" w:sz="4" w:space="0" w:color="auto"/>'
            '<w:insideH w:val="single" w:sz="4" w:space="0" w:color="auto"/><w:insideV w:val="single" w:sz="4" w:space="0" w:color="auto"/></w:tblBorders></w:tblPr></w:style>')
    mine = ('<w:style w:type="table" w:styleId="Ledger"><w:name w:val="Ledger"/><w:basedOn w:val="TableNormal"/>'
            '<w:tblPr><w:tblBorders><w:bottom w:val="double" w:sz="6" w:space="0" w:color="auto"/></w:tblBorders></w:tblPr></w:style>')
    rows = [tr(tc(p(r("x"))), tc(p(r("y"))), tc(p(r("z"))))]
    body = (p(r("No style:"))
            + table(rows)
            + p(r("Table Grid:"))
            + table(rows, tblpr='<w:tblStyle w:val="TableGrid"/><w:tblW w:w="0" w:type="auto"/>')
            + p(r("Custom:"))
            + table(rows, tblpr='<w:tblStyle w:val="Ledger"/><w:tblW w:w="0" w:type="auto"/>')
            + p(r("After.")))
    package("edge-table-styles.docx", body, styles_extra=grid + mine)


# 5. Notes: footnotes and endnotes, separators, a change inside a note and
#    a tracked change on a separator.
def edge_notes():
    sep = '<w:footnote w:type="separator" w:id="-1">' + p('<w:r><w:separator/></w:r>') + '</w:footnote><w:footnote w:type="continuationSeparator" w:id="0">' + p('<w:r><w:continuationSeparator/></w:r>') + '</w:footnote>'
    fn = (f'<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n<w:footnotes {W}>{sep}'
          '<w:footnote w:id="1">' + p('<w:r><w:rPr><w:rStyle w:val="FootnoteReference"/></w:rPr><w:footnoteRef/></w:r>', r(" A footnote "), ins(30, "Ann Lee", r("with an insertion")), r("."), ppr='<w:pStyle w:val="FootnoteText"/>') + '</w:footnote></w:footnotes>')
    esep = sep.replace("footnote", "endnote")
    en = (f'<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n<w:endnotes {W}>{esep}'
          '<w:endnote w:id="1">' + p('<w:r><w:rPr><w:rStyle w:val="EndnoteReference"/></w:rPr><w:endnoteRef/></w:r>', r(" An endnote."), ppr='<w:pStyle w:val="EndnoteText"/>') + '</w:endnote>'
          '<w:endnote w:id="2">' + p('<w:r><w:rPr><w:rStyle w:val="EndnoteReference"/></w:rPr><w:endnoteRef/></w:r>', r(" A second endnote."), ppr='<w:pStyle w:val="EndnoteText"/>') + '</w:endnote></w:endnotes>')
    styles = ('<w:style w:type="paragraph" w:styleId="FootnoteText"><w:name w:val="footnote text"/><w:basedOn w:val="Normal"/><w:pPr><w:spacing w:after="0" w:line="240" w:lineRule="auto"/></w:pPr><w:rPr><w:sz w:val="20"/></w:rPr></w:style>'
              '<w:style w:type="character" w:styleId="FootnoteReference"><w:name w:val="footnote reference"/><w:rPr><w:vertAlign w:val="superscript"/></w:rPr></w:style>'
              '<w:style w:type="paragraph" w:styleId="EndnoteText"><w:name w:val="endnote text"/><w:basedOn w:val="Normal"/><w:pPr><w:spacing w:after="0" w:line="240" w:lineRule="auto"/></w:pPr><w:rPr><w:sz w:val="20"/></w:rPr></w:style>'
              '<w:style w:type="character" w:styleId="EndnoteReference"><w:name w:val="endnote reference"/><w:rPr><w:vertAlign w:val="superscript"/></w:rPr></w:style>')
    body = p(r("Foot"), '<w:r><w:rPr><w:rStyle w:val="FootnoteReference"/></w:rPr><w:footnoteReference w:id="1"/></w:r>',
             r(" and end"), '<w:r><w:rPr><w:rStyle w:val="EndnoteReference"/></w:rPr><w:endnoteReference w:id="1"/></w:r>',
             r(" and end again"), '<w:r><w:rPr><w:rStyle w:val="EndnoteReference"/></w:rPr><w:endnoteReference w:id="2"/></w:r>', r("."))
    settings = ('<w:footnotePr><w:footnote w:id="-1"/><w:footnote w:id="0"/></w:footnotePr>'
                '<w:endnotePr><w:endnote w:id="-1"/><w:endnote w:id="0"/></w:endnotePr>')
    package("edge-notes.docx", body,
            parts={"word/footnotes.xml": fn, "word/endnotes.xml": en},
            rels=[("rIdFn", f"{REL}/footnotes", "footnotes.xml"), ("rIdEn", f"{REL}/endnotes", "endnotes.xml")],
            types=[("/word/footnotes.xml", f"{CT}.footnotes+xml"), ("/word/endnotes.xml", f"{CT}.endnotes+xml")],
            styles_extra=styles, settings=settings)


# 6. Table cells: a row that starts and ends short (gridBefore, gridAfter),
#    spans across and down, an empty cell inside a span, cells of several
#    paragraphs.
def edge_cells():
    w = '<w:tcW w:w="2400" w:type="dxa"/>'
    rows = [
        tr(tc(p(r("h1"))), tc(p(r("h2"))), tc(p(r("h3"))), tc(p(r("h4")))),
        tr(tc(p(r("starts late"))), tc(p(r("x"))), tc(p(r("y"))), trpr='<w:gridBefore w:val="1"/><w:wBefore w:w="2400" w:type="dxa"/>'),
        tr(tc(p(r("ends early"))), tc(p(r("u"))), tc(p(r("v"))), trpr='<w:gridAfter w:val="1"/><w:wAfter w:w="2400" w:type="dxa"/>'),
        tr(tc(p(r("across two")), tcpr=w + '<w:gridSpan w:val="2"/>'), tc(p(r("down"), ), tcpr=w + '<w:vMerge w:val="restart"/>'), tc(p(r("two paragraphs")), p(r("in one cell")))),
        tr(tc(tcpr=w + '<w:gridSpan w:val="2"/>'), tc(tcpr=w + '<w:vMerge/>'), tc(p(r("last")))),
    ]
    package("edge-cells.docx", p(r("Cells:")) + table(rows, grid=(2400, 2400, 2400, 2400)) + p(r("After.")))


# 7. One change, several marks: a deletion a comment range cuts (its
#    reference right after the range's end, where Word puts it), two
#    adjacent deletions by one author that differ in formatting, an
#    insertion inside a link, an insertion across two paragraphs.
def edge_change_ids():
    body = (
        p(r("Before "), dele(40, "Ann Lee", dr("cut ")), '<w:commentRangeStart w:id="0"/>', dele(41, "Ann Lee", dr("by a")),
          '<w:commentRangeEnd w:id="0"/>', '<w:r><w:commentReference w:id="0"/></w:r>', dele(42, "Ann Lee", dr(" comment")), r(" after."))
        + p(r("Joined: "), dele(43, "Ann Lee", dr("plain ")), dele(44, "Ann Lee", dr("italic", "<w:i/>")), r(" end."))
        + p(r("Link: "), '<w:hyperlink r:id="rIdLink">' + ins(45, "Ann Lee", r("site", '<w:rStyle w:val="Hyperlink"/>')) + '</w:hyperlink>', r("."))
        + p(r("Across "), ins(46, "Ann Lee", r("two")), ppr=f'<w:rPr><w:ins w:id="47" w:author="Ann Lee" w:date="{D1}"/></w:rPr>')
        + p(ins(48, "Ann Lee", r("paragraphs")), r(" here."))
    )
    styles = '<w:style w:type="character" w:styleId="Hyperlink"><w:name w:val="Hyperlink"/><w:rPr><w:color w:val="0563C1"/><w:u w:val="single"/></w:rPr></w:style>'
    package("edge-change-ids.docx", body,
            parts={"word/comments.xml": comments((0, "Bo Chen", "Cuts the deletion."))},
            rels=[COMMENTS[0], ("rIdLink", f"{REL}/hyperlink", "https://example.com/")],
            types=[COMMENTS[1]], styles_extra=styles)


# 8. Revision order: deletion then insertion, insertion then deletion, two
#    of each, two authors, two substitutions side by side.
def edge_order():
    a, b = "Ann Lee", "Bo Chen"
    body = (
        p(r("Del then ins: "), dele(50, a, dr("old")), ins(51, a, r("new")), r("."))
        + p(r("Ins then del: "), ins(52, a, r("new")), dele(53, a, dr("old")), r("."))
        + p(r("Ins ins del del: "), ins(54, a, r("one ")), ins(55, b, r("two ")), dele(56, a, dr("three ")), dele(57, b, dr("four")), r("."))
        + p(r("Two authors: "), dele(58, a, dr("hers")), ins(59, b, r("his")), r("."))
        + p(r("Side by side: "), dele(60, a, dr("a")), ins(61, a, r("b")), dele(62, a, dr("c")), ins(63, a, r("d")), r("."))
        + p(r("Spaced: "), dele(64, a, dr("x")), r(" "), ins(65, a, r("y")), r("."))
    )
    package("edge-order.docx", body)


# 9. Author dates: one author over several days, one at one time, one on
#    one day at two times, one undated.
def edge_dates():
    body = (
        p(r("Range: "), ins(70, "Ann Lee", r("one "), D1), ins(71, "Ann Lee", r("two"), D2), r("."))
        + p(r("Single: "), ins(72, "Bo Chen", r("once"), "2026-03-03T09:15:00Z"), r("."))
        + p(r("Same day: "), ins(73, "Cy Diaz", r("morning "), "2026-03-05T08:00:00Z"), ins(74, "Cy Diaz", r("evening"), "2026-03-05T20:00:00Z"), r("."))
        + p(r("Undated: "), '<w:ins w:id="75" w:author="Di Eng">' + r("whenever") + "</w:ins>", r("."))
    )
    package("edge-dates.docx", body)


# 10. Links to bookmarks and the bookmarks they name; reference fields.
def edge_anchors():
    hl = '<w:style w:type="character" w:styleId="Hyperlink"><w:name w:val="Hyperlink"/><w:rPr><w:color w:val="0563C1"/><w:u w:val="single"/></w:rPr></w:style>'
    body = (
        p('<w:bookmarkStart w:id="0" w:name="exec_summary"/>', r("Executive Summary"), '<w:bookmarkEnd w:id="0"/>',
          '<w:bookmarkStart w:id="1" w:name="unused"/><w:bookmarkEnd w:id="1"/>')
        + p(r("Back to "), '<w:hyperlink w:anchor="exec_summary" w:tooltip="Go up" w:history="1">', r("the summary", '<w:rStyle w:val="Hyperlink"/>'), "</w:hyperlink>", r("."))
        + p(r("Jump "), '<w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> HYPERLINK \\l "exec_summary" \\h </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r>',
            r("up", '<w:rStyle w:val="Hyperlink"/>'), '<w:r><w:fldChar w:fldCharType="end"/></w:r>', r("."))
        + p('<w:bookmarkStart w:id="2" w:name="_Ref100"/>', r("Clause 4"), '<w:bookmarkEnd w:id="2"/>', r(" sets the fee."))
        + p(r("See "), '<w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> REF _Ref100 \\h </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r>',
            r("Clause 4"), '<w:r><w:fldChar w:fldCharType="end"/></w:r>', r(" on page "),
            '<w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> PAGEREF _Ref100 \\h </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r>',
            r("1"), '<w:r><w:fldChar w:fldCharType="end"/></w:r>', r("."))
        + p(r("Outside: "), '<w:hyperlink r:id="rIdLink" w:anchor="part" w:tooltip="External">', r("there", '<w:rStyle w:val="Hyperlink"/>'), "</w:hyperlink>", r("."))
    )
    package("edge-anchors.docx", body, rels=[("rIdLink", f"{REL}/hyperlink", "https://example.com/a")], styles_extra=hl)


if __name__ == "__main__":
    for make in (edge_clauses, edge_stories, edge_owner, edge_table_styles, edge_notes, edge_cells,
                 edge_change_ids, edge_order, edge_dates, edge_anchors):
        make()
    print("\n".join(sorted(p.name for p in HERE.glob("*.docx"))))

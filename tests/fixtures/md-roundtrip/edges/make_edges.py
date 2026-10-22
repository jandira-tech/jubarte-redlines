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
    rels = [("rIdStyles", f"{REL}/styles", "styles.xml"), ("rIdSettings", f"{REL}/settings", "settings.xml"), *rels]
    types = [("/word/document.xml", f"{CT}.document.main+xml"), ("/word/styles.xml", f"{CT}.styles+xml"), ("/word/settings.xml", f"{CT}.settings+xml"), *types]
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
    order = ["[Content_Types].xml", "_rels/.rels", *sorted(k for k in parts if k not in ("[Content_Types].xml", "_rels/.rels"))]
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


# 1b. More clauses: three format changes on one paragraph; an inserted
#     empty row, an inserted cell, a cell paragraph's format change and two
#     table format changes (properties and grid); an inserted row whose text
#     carries no marks of its own; a comment range from one cell to another;
#     two comment ranges over one table; an empty mark in an empty cell.
def edge_clauses_more():
    def fmt(tag, i, inner):
        return f'<w:{tag} w:id="{i}" w:author="Ann Lee" w:date="{D1}">{inner}</w:{tag}>'
    w = '<w:tcW w:w="2400" w:type="dxa"/>'
    body = (
        p(r("Three", "<w:b/>" + fmt("rPrChange", 101, "<w:rPr/>")),
          r(" format changes.", "<w:i/>" + fmt("rPrChange", 102, "<w:rPr/>")),
          ppr='<w:jc w:val="center"/>' + fmt("pPrChange", 100, '<w:pPr><w:jc w:val="left"/></w:pPr>'))
        + p(r("Changed table:"))
        + table([
            tr(tc(p()), tc(p()), trpr=f'<w:ins w:id="103" w:author="Ann Lee" w:date="{D1}"/>'),
            tr(tc(p(r("a2"), ppr='<w:jc w:val="center"/>' + fmt("pPrChange", 105, "<w:pPr/>"))),
               tc(p(r("b2")), tcpr=w + f'<w:cellIns w:id="104" w:author="Ann Lee" w:date="{D1}"/>')),
        ], tblpr='<w:tblW w:w="0" w:type="auto"/>' + fmt("tblPrChange", 106, '<w:tblPr><w:tblW w:w="5000" w:type="pct"/></w:tblPr>'),
           grid=(2400, 2400)).replace(
            '</w:tblGrid>', '<w:tblGridChange w:id="107"><w:tblGrid><w:gridCol w:w="2000"/><w:gridCol w:w="2800"/></w:tblGrid></w:tblGridChange>' + '</w:tblGrid>')
        + p(r("Inserted row:"))
        + table([
            tr(tc(p(r("x0"))), tc(p(r("y0")))),
            tr(tc(p(r("x1"))), tc(p(r("y1"))), trpr=f'<w:ins w:id="108" w:author="Ann Lee" w:date="{D1}"/>'),
        ], grid=(2400, 2400))
        + p(r("Cell to cell:"))
        + table([
            tr(tc(p('<w:commentRangeStart w:id="0"/>', r("a1"))), tc(p(r("b1")))),
            tr(tc(p(r("a2"))), tc(p(r("b2"), '<w:commentRangeEnd w:id="0"/>', '<w:r><w:commentReference w:id="0"/></w:r>'))),
        ], grid=(2400, 2400))
        + '<w:commentRangeStart w:id="1"/><w:commentRangeStart w:id="2"/>'
        + table([tr(tc(p(r("m1"))), tc(p(r("n1"))))], grid=(2400, 2400))
        + p(r("After two ranges."), '<w:commentRangeEnd w:id="1"/>', '<w:r><w:commentReference w:id="1"/></w:r>',
            '<w:commentRangeEnd w:id="2"/>', '<w:r><w:commentReference w:id="2"/></w:r>')
        + table([tr(tc(p(r("e1"))), tc(p(f'<w:ins w:id="109" w:author="Ann Lee" w:date="{D1}"/>')))], grid=(2400, 2400))
        + p(r("End."))
    )
    package("edge-clauses-more.docx", body,
            parts={"word/comments.xml": comments((0, "Bo Chen", "From cell to cell."), (1, "Bo Chen", "First over the table."), (2, "Bo Chen", "Second over the table."))},
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
    # Someone called none, and a title; XML specials in both.
    package("edge-owner-none.docx", p(r("Nobody?")),
            core='<dc:title>Quarterly &amp; &lt;Report&gt; "Q"</dc:title><dc:creator>none</dc:creator>')
    package("edge-owner-specials.docx", p(r("Specials.")),
            core='<dc:title>none</dc:title><dc:creator xml:space="preserve">Ada &amp; Bo &lt;C&gt; "Q"</dc:creator>')


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


# 11. More of one change in several marks: a comment range and its
#     reference inside one deletion; a substitution whose deleted side is
#     two runs (`r5+6+7`); more marks than characters (empty marks); one id
#     on both sides of a substitution (`r43+43`).
def edge_change_ids_more():
    ref = '<w:r><w:commentReference w:id="0"/></w:r>'
    body = (
        p(r("Inside "), dele(40, "Ann Lee", dr("cut ") + '<w:commentRangeStart w:id="0"/>' + dr("by a")
                             + '<w:commentRangeEnd w:id="0"/>' + ref + dr(" comment")), r(" after."))
        + p(r("Sub "), ins(7, "Ann Lee", r("x")), dele(5, "Ann Lee", dr("plain ")),
            dele(6, "Ann Lee", dr("italic", "<w:i/>")), r(" end."))
        + p(r("Empty "), ins(8, "Ann Lee", r("a")), '<w:ins w:id="9" w:author="Ann Lee" w:date="2026-03-02T10:00:00Z"/>',
            '<w:ins w:id="10" w:author="Ann Lee" w:date="2026-03-02T10:00:00Z"/>', r(" end."))
    )
    package("edge-change-ids-more.docx", body,
            parts={"word/comments.xml": comments((0, "Bo Chen", "Inside the deletion."))},
            rels=[COMMENTS[0]], types=[COMMENTS[1]])


# 12. Cells the view leaves out: a row short of the grid without gridAfter,
#     gridAfter past the last column with text, a vertical merge in a
#     column with no text, a table of empty cells.
def edge_cells_more():
    w = '<w:tcW w:w="2400" w:type="dxa"/>'
    after = '<w:gridAfter w:val="1"/><w:wAfter w:w="2400" w:type="dxa"/>'
    short = table([tr(tc(p(r("a"))), tc(p(r("b"))), tc(p(r("c")))),
                   tr(tc(p(r("d"))), tc(p(r("e")))),
                   tr(tc(p(r("g"))), tc(p(r("h"))), tc(p(r("i"))))])
    past = table([tr(tc(p(r("j"))), tc(p(r("k"))), tc(p(r("l"))), trpr=after),
                  tr(tc(p(r("m"))), tc(p(r("n"))), tc(p(r("o"))), trpr=after)],
                 grid=(2400, 2400, 2400, 2400))
    merged = table([tr(tc(p(r("q"))), tc(tcpr=w + '<w:vMerge w:val="restart"/>')),
                    tr(tc(p(r("s"))), tc(tcpr=w + '<w:vMerge/>'))], grid=(2400, 2400))
    empty = table([tr(tc(), tc()), tr(tc(), tc())], grid=(2400, 2400))
    package("edge-cells-more.docx", p(r("Short:")) + short + p(r("Past:")) + past + p(r("Merged:")) + merged
            + p(r("Empty:")) + empty + p(r("After.")))


# 13. Revision order around other things: a substitution inside a link, a
#     bookmark a link names between an insertion and a deletion, a comment
#     range between them, insertions then deletions by one author numbered
#     in document order, a substitution whose sides differ in day.
def edge_order_more():
    a = "Ann Lee"
    hl = '<w:rStyle w:val="Hyperlink"/>'
    body = (
        p(r("Link: "), '<w:hyperlink r:id="rIdLink">' + ins(80, a, r("new", hl)) + dele(81, a, dr("old", hl)) + '</w:hyperlink>', r("."))
        + p(r("Mark: "), ins(82, a, r("new")), '<w:bookmarkStart w:id="0" w:name="mark"/><w:bookmarkEnd w:id="0"/>',
            dele(83, a, dr("old")), r(" "), '<w:hyperlink w:anchor="mark">' + r("go", hl) + '</w:hyperlink>', r("."))
        + p(r("Range: "), ins(84, a, r("new")), '<w:commentRangeStart w:id="0"/>', dele(85, a, dr("old")),
            '<w:commentRangeEnd w:id="0"/>', '<w:r><w:commentReference w:id="0"/></w:r>', r("."))
        + p(r("Runs: "), ins(86, a, r("AA ")), ins(87, a, r("BB ")), dele(88, a, dr("CC ")), dele(89, a, dr("DD")), r("."))
        + p(r("Days: "), ins(90, "Eve Fox", r("new"), D1), dele(91, "Eve Fox", dr("old"), D2), r("."))
    )
    styles = '<w:style w:type="character" w:styleId="Hyperlink"><w:name w:val="Hyperlink"/><w:rPr><w:color w:val="0563C1"/><w:u w:val="single"/></w:rPr></w:style>'
    package("edge-order-more.docx", body,
            parts={"word/comments.xml": comments((0, "Bo Chen", "On the deletion."))},
            rels=[COMMENTS[0], ("rIdLink", f"{REL}/hyperlink", "https://example.com/")],
            types=[COMMENTS[1]], styles_extra=styles)


# 15. Notes inserted and deleted with their references, in Word's shape:
#     each note paragraph's mark is a change of its own, and one w:ins or
#     w:del holds the reference mark and the text (file_170.docx, Word 16).
def edge_notes_tracked():
    a = "Ann Lee"
    ref = '<w:r><w:rPr><w:rStyle w:val="FootnoteReference"/></w:rPr><w:footnoteRef/></w:r>'

    def mark(kind, i):
        return f'<w:pStyle w:val="FootnoteText"/><w:rPr><w:{kind} w:id="{i}" w:author="{a}" w:date="{D1}"/></w:rPr>'

    sep = '<w:footnote w:type="separator" w:id="-1">' + p('<w:r><w:separator/></w:r>') + '</w:footnote><w:footnote w:type="continuationSeparator" w:id="0">' + p('<w:r><w:continuationSeparator/></w:r>') + '</w:footnote>'
    fn = (f'<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n<w:footnotes {W}>{sep}'
          '<w:footnote w:id="1">' + p(ins(12, a, ref + r(" An inserted note.")), ppr=mark("ins", 11)) + '</w:footnote>'
          '<w:footnote w:id="2">' + p(dele(22, a, ref + dr(" A deleted note.")), ppr=mark("del", 21)) + '</w:footnote>'
          '<w:footnote w:id="3">' + p(ins(32, a, ref + r(" Two paragraphs,")), ppr=mark("ins", 31))
          + p(ins(34, a, r("inserted.")), ppr=mark("ins", 33)) + '</w:footnote></w:footnotes>')
    styles = ('<w:style w:type="paragraph" w:styleId="FootnoteText"><w:name w:val="footnote text"/><w:basedOn w:val="Normal"/><w:pPr><w:spacing w:after="0" w:line="240" w:lineRule="auto"/></w:pPr><w:rPr><w:sz w:val="20"/></w:rPr></w:style>'
              '<w:style w:type="character" w:styleId="FootnoteReference"><w:name w:val="footnote reference"/><w:rPr><w:vertAlign w:val="superscript"/></w:rPr></w:style>')
    note = '<w:r><w:rPr><w:rStyle w:val="FootnoteReference"/></w:rPr><w:footnoteReference w:id="{}"/></w:r>'
    body = (p(r("Added"), ins(10, a, note.format(1)), r(" and cut"),
              dele(20, a, note.format(2)), r("."))
            + p(r("Long"), ins(30, a, note.format(3)), r(".")))
    settings = '<w:footnotePr><w:footnote w:id="-1"/><w:footnote w:id="0"/></w:footnotePr>'
    package("edge-notes-tracked.docx", body,
            parts={"word/footnotes.xml": fn},
            rels=[("rIdFn", f"{REL}/footnotes", "footnotes.xml")],
            types=[("/word/footnotes.xml", f"{CT}.footnotes+xml")],
            styles_extra=styles, settings=settings)


# 16. A table inside a footnote: its text must reach the view.
def edge_notes_table():
    ref = '<w:r><w:rPr><w:rStyle w:val="FootnoteReference"/></w:rPr><w:footnoteRef/></w:r>'
    sep = '<w:footnote w:type="separator" w:id="-1">' + p('<w:r><w:separator/></w:r>') + '</w:footnote><w:footnote w:type="continuationSeparator" w:id="0">' + p('<w:r><w:continuationSeparator/></w:r>') + '</w:footnote>'
    ppr = '<w:pStyle w:val="FootnoteText"/>'
    grid = '<w:tblGrid><w:gridCol w:w="2400"/><w:gridCol w:w="2400"/></w:tblGrid>'
    def cell(text):
        return '<w:tc><w:tcPr><w:tcW w:w="2400" w:type="dxa"/></w:tcPr>' + p(r(text), ppr=ppr) + '</w:tc>'
    tbl = ('<w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/></w:tblPr>' + grid
           + '<w:tr>' + cell("fn-a") + cell("fn-b") + '</w:tr><w:tr>' + cell("fn-c") + cell("fn-d") + '</w:tr></w:tbl>')
    fn = (f'<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n<w:footnotes {W}>{sep}'
          '<w:footnote w:id="1">' + p(ref, r(" Note"), ppr=ppr) + tbl + p(r("After."), ppr=ppr) + '</w:footnote></w:footnotes>')
    styles = ('<w:style w:type="paragraph" w:styleId="FootnoteText"><w:name w:val="footnote text"/><w:basedOn w:val="Normal"/><w:pPr><w:spacing w:after="0" w:line="240" w:lineRule="auto"/></w:pPr><w:rPr><w:sz w:val="20"/></w:rPr></w:style>'
              '<w:style w:type="character" w:styleId="FootnoteReference"><w:name w:val="footnote reference"/><w:rPr><w:vertAlign w:val="superscript"/></w:rPr></w:style>')
    body = p(r("Table"), '<w:r><w:rPr><w:rStyle w:val="FootnoteReference"/></w:rPr><w:footnoteReference w:id="1"/></w:r>', r("."))
    settings = '<w:footnotePr><w:footnote w:id="-1"/><w:footnote w:id="0"/></w:footnotePr>'
    package("edge-notes-table.docx", body,
            parts={"word/footnotes.xml": fn},
            rels=[("rIdFn", f"{REL}/footnotes", "footnotes.xml")],
            types=[("/word/footnotes.xml", f"{CT}.footnotes+xml")],
            styles_extra=styles, settings=settings)


if __name__ == "__main__":
    for make in (edge_clauses, edge_clauses_more, edge_stories, edge_owner, edge_table_styles, edge_notes, edge_cells,
                 edge_change_ids, edge_order, edge_dates, edge_anchors, edge_change_ids_more, edge_cells_more,
                 edge_order_more, edge_notes_tracked, edge_notes_table):
        make()
    print("\n".join(sorted(p.name for p in HERE.glob("*.docx"))))

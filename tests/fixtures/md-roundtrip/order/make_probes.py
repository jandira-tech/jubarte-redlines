# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only
"""Write the revision-order probes: small docx files where something sits
between, inside or across the two marks of a substitution (a comment range,
a bookmark, a hidden run, a soft hyphen), in the body, a table cell, a
header and a footnote. `tests/md_order_probes.rs` reads them. Run from
anywhere: `python3 make_probes.py`; it rewrites the `.docx` files beside
it, byte for byte the same each time."""
import pathlib
import zipfile

HERE = pathlib.Path(__file__).resolve().parent
W = 'xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"'
REL = "http://schemas.openxmlformats.org/officeDocument/2006/relationships"
PKG = "http://schemas.openxmlformats.org/package/2006/relationships"
CT = "application/vnd.openxmlformats-officedocument.wordprocessingml"
D1 = "2026-03-02T10:00:00Z"

STYLES = f"""<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles {W}><w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:ascii="Calibri" w:hAnsi="Calibri" w:cs="Calibri"/><w:sz w:val="22"/><w:szCs w:val="22"/></w:rPr></w:rPrDefault><w:pPrDefault><w:pPr><w:spacing w:after="160" w:line="259" w:lineRule="auto"/></w:pPr></w:pPrDefault></w:docDefaults><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:qFormat/></w:style><w:style w:type="character" w:default="1" w:styleId="DefaultParagraphFont"><w:name w:val="Default Paragraph Font"/></w:style><w:style w:type="table" w:default="1" w:styleId="TableNormal"><w:name w:val="Normal Table"/><w:tblPr><w:tblInd w:w="0" w:type="dxa"/><w:tblCellMar><w:top w:w="0" w:type="dxa"/><w:left w:w="108" w:type="dxa"/><w:bottom w:w="0" w:type="dxa"/><w:right w:w="108" w:type="dxa"/></w:tblCellMar></w:tblPr></w:style>{{extra}}</w:styles>"""

SECT = '<w:sectPr>{refs}<w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720" w:gutter="0"/></w:sectPr>'

EXTRA = (
    '<w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/><w:basedOn w:val="Normal"/>'
    '<w:pPr><w:outlineLvl w:val="0"/></w:pPr></w:style>'
    '<w:style w:type="character" w:styleId="Vanish"><w:name w:val="Vanish"/>'
    '<w:rPr><w:vanish/></w:rPr></w:style>'
    '<w:style w:type="character" w:styleId="VanishChild"><w:name w:val="Vanish Child"/>'
    '<w:basedOn w:val="Vanish"/></w:style>'
    '<w:style w:type="paragraph" w:styleId="Header"><w:name w:val="header"/><w:basedOn w:val="Normal"/></w:style>'
    '<w:style w:type="paragraph" w:styleId="Footer"><w:name w:val="footer"/><w:basedOn w:val="Normal"/></w:style>'
    '<w:style w:type="paragraph" w:styleId="FootnoteText"><w:name w:val="footnote text"/><w:basedOn w:val="Normal"/></w:style>'
    '<w:style w:type="character" w:styleId="FootnoteReference"><w:name w:val="footnote reference"/>'
    '<w:rPr><w:vertAlign w:val="superscript"/></w:rPr></w:style>'
    '<w:style w:type="paragraph" w:styleId="CommentText"><w:name w:val="annotation text"/>'
    '<w:basedOn w:val="Normal"/></w:style>'
)


def p(*runs, ppr=""):
    return f"<w:p>{'<w:pPr>' + ppr + '</w:pPr>' if ppr else ''}{''.join(runs)}</w:p>"


def r(text, rpr=""):
    return f'<w:r>{"<w:rPr>" + rpr + "</w:rPr>" if rpr else ""}<w:t xml:space="preserve">{text}</w:t></w:r>'


def dr(text, rpr=""):
    return f'<w:r>{"<w:rPr>" + rpr + "</w:rPr>" if rpr else ""}<w:delText xml:space="preserve">{text}</w:delText></w:r>'


def raw_run(inner):
    return f"<w:r>{inner}</w:r>"


def ins(i, body, who="Ann Lee"):
    return f'<w:ins w:id="{i}" w:author="{who}" w:date="{D1}">{body}</w:ins>'


def dele(i, body, who="Ann Lee"):
    return f'<w:del w:id="{i}" w:author="{who}" w:date="{D1}">{body}</w:del>'


def bm(i, name):
    return f'<w:bookmarkStart w:id="{i}" w:name="{name}"/><w:bookmarkEnd w:id="{i}"/>'


def link(anchor, text="see"):
    return f'<w:hyperlink w:anchor="{anchor}">{r(text)}</w:hyperlink>'


def cref(i):
    return f'<w:r><w:rPr><w:rStyle w:val="CommentReference"/></w:rPr><w:commentReference w:id="{i}"/></w:r>'


def sub_around(mid, new="new", old="old", i=1, j=2):
    """Insertion, then `mid`, then deletion. Word Compare order."""
    return ins(i, r(new)) + mid + dele(j, dr(old))


def package(name, body, *, sect_refs="", parts=None, rels=None, types=None, styles_extra="", settings=""):
    parts = dict(parts or {})
    rels = list(rels or [])
    types = list(types or [])
    document = (
        f'<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n'
        f'<w:document {W}><w:body>{body}{SECT.format(refs=sect_refs)}</w:body></w:document>'
    )
    parts["word/document.xml"] = document
    parts["word/styles.xml"] = STYLES.replace("{extra}", EXTRA + styles_extra)
    parts["word/settings.xml"] = (
        f'<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n<w:settings {W}>{settings}</w:settings>'
    )
    rels = [
        ("rIdStyles", f"{REL}/styles", "styles.xml"),
        ("rIdSettings", f"{REL}/settings", "settings.xml"),
    ] + rels
    types = [
        ("/word/document.xml", f"{CT}.document.main+xml"),
        ("/word/styles.xml", f"{CT}.styles+xml"),
        ("/word/settings.xml", f"{CT}.settings+xml"),
    ] + types
    package_rels = [("rId1", f"{REL}/officeDocument", "word/document.xml")]
    overrides = "".join(f'<Override PartName="{n}" ContentType="{t}"/>' for n, t in types)
    parts["[Content_Types].xml"] = (
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n'
        '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">'
        '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>'
        f'<Default Extension="xml" ContentType="application/xml"/>{overrides}</Types>'
    )
    parts["_rels/.rels"] = (
        f'<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n<Relationships xmlns="{PKG}">'
        + "".join(f'<Relationship Id="{i}" Type="{t}" Target="{g}"/>' for i, t, g in package_rels)
        + "</Relationships>"
    )
    parts["word/_rels/document.xml.rels"] = (
        f'<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n<Relationships xmlns="{PKG}">'
        + "".join(
            f'<Relationship Id="{i}" Type="{t}" Target="{g}"/>' for i, t, g in rels
        )
        + "</Relationships>"
    )
    order = ["[Content_Types].xml", "_rels/.rels"] + sorted(
        k for k in parts if k not in ("[Content_Types].xml", "_rels/.rels")
    )
    with zipfile.ZipFile(HERE / name, "w", zipfile.ZIP_DEFLATED) as z:
        for part in order:
            info = zipfile.ZipInfo(part, date_time=(1980, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            z.writestr(info, parts[part])


def comments(*items):
    cstyle = '<w:pStyle w:val="CommentText"/>'
    body = "".join(
        f'<w:comment w:id="{i}" w:author="{who}" w:date="{D1}" w:initials="{who[0]}">'
        f"{p(r(text), ppr=cstyle)}</w:comment>"
        for i, who, text in items
    )
    return f'<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n<w:comments {W}>{body}</w:comments>'


COMMENTS = (("rIdComments", f"{REL}/comments", "comments.xml"), ("/word/comments.xml", f"{CT}.comments+xml"))


def with_comment(body, note="note"):
    package_kwargs = dict(
        parts={"word/comments.xml": comments((0, "Ann Lee", note))},
        rels=[COMMENTS[0]],
        types=[COMMENTS[1]],
    )
    return package_kwargs


def story(kind, body):
    root = "hdr" if kind == "header" else "ftr"
    return f'<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n<w:{root} {W}>{body}</w:{root}>'


def table_cell(inner, heading=False):
    ppr = '<w:pStyle w:val="Heading1"/>' if heading else ""
    return (
        "<w:tbl><w:tblPr><w:tblW w:w=\"0\" w:type=\"auto\"/></w:tblPr>"
        "<w:tblGrid><w:gridCol w:w=\"2400\"/></w:tblGrid><w:tr><w:tc>"
        f"<w:tcPr><w:tcW w:w=\"2400\" w:type=\"dxa\"/></w:tcPr>{p(inner, ppr=ppr)}"
        "</w:tc></w:tr></w:tbl>"
    )


def fn_part(inner, heading=False):
    ppr = '<w:pStyle w:val="Heading1"/>' if heading else '<w:pStyle w:val="FootnoteText"/>'
    sep = (
        '<w:footnote w:type="separator" w:id="-1">' + p('<w:r><w:separator/></w:r>') + "</w:footnote>"
        '<w:footnote w:type="continuationSeparator" w:id="0">'
        + p('<w:r><w:continuationSeparator/></w:r>')
        + "</w:footnote>"
    )
    return (
        f'<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n<w:footnotes {W}>{sep}'
        f'<w:footnote w:id="1">{p(inner, ppr=ppr)}</w:footnote></w:footnotes>'
    )


FN_RELS = [("rIdFn", f"{REL}/footnotes", "footnotes.xml")]
FN_TYPES = [("/word/footnotes.xml", f"{CT}.footnotes+xml")]
FN_SETTINGS = '<w:footnotePr><w:footnote w:id="-1"/><w:footnote w:id="0"/></w:footnotePr>'
FN_REF = '<w:r><w:rPr><w:rStyle w:val="FootnoteReference"/></w:rPr><w:footnoteReference w:id="1"/></w:r>'


def h1(inner):
    return p(inner, ppr='<w:pStyle w:val="Heading1"/>')


# --- body separators between ins and del ---
CASES = {}

def add(name, body, **kw):
    CASES[name] = (body, kw)


add("direct-vanish", p(r("A "), sub_around(raw_run("<w:rPr><w:vanish/></w:rPr><w:t>HIDDEN</w:t>")), r(" Z")))
add("style-vanish", p(r("A "), sub_around(raw_run('<w:rPr><w:rStyle w:val="Vanish"/></w:rPr><w:t>HIDDEN</w:t>')), r(" Z")))
add("style-basedon", p(r("A "), sub_around(raw_run('<w:rPr><w:rStyle w:val="VanishChild"/></w:rPr><w:t>HIDDEN</w:t>')), r(" Z")))
add("style-vanish-tab", p(r("A "), sub_around(raw_run('<w:rPr><w:rStyle w:val="Vanish"/></w:rPr><w:tab/>')), r(" Z")))
add("lrp-only", p(r("A "), sub_around(raw_run("<w:lastRenderedPageBreak/>")), r(" Z")))
add("empty-t-tab", p(r("A "), sub_around(raw_run('<w:t xml:space="preserve"></w:t><w:tab/>')), r(" Z")))
add("empty-t-tab-lrp", p(r("A "), sub_around(raw_run('<w:lastRenderedPageBreak/><w:t xml:space="preserve"></w:t><w:tab/>')), r(" Z")))
add("page-br", p(r("A "), sub_around(raw_run('<w:br w:type="page"/>')), r(" Z")))
add("soft-hyphen", p(r("A "), sub_around(raw_run("<w:softHyphen/>")), r(" Z")))
add("sym", p(r("A "), sub_around(raw_run('<w:sym w:font="Wingdings" w:char="F0E0"/>')), r(" Z")))
add("empty-run", p(r("A "), sub_around(raw_run("<w:rPr><w:b/></w:rPr>")), r(" Z")))

# heading slug bookmarks
add(
    "h-own-slug",
    h1(ins(1, r("Title")) + bm(9, "title") + dele(2, dr("Old")))
    + p(link("title", "go")),
)
add(
    "h-suffix-1",
    h1(ins(1, r("Title")) + bm(9, "title-1") + dele(2, dr("Old")))
    + p(link("title-1", "go")),
)
add(
    "h-suffix-01",
    h1(ins(1, r("Title")) + bm(9, "title-01") + dele(2, dr("Old")))
    + p(link("title-01", "go")),
)
add(
    "h-suffix-1-1",
    h1(ins(1, r("Intro 1")) + bm(9, "intro-1-1") + dele(2, dr("Old")))
    + p(link("intro-1-1", "go")),
)
add(
    "h-two",
    h1(r("Title"))
    + h1(ins(1, r("Title")) + bm(9, "title-1") + dele(2, dr("Old")))
    + p(link("title-1", "go")),
)
add(
    "h-foreign",
    h1(ins(1, r("Title")) + bm(9, "other") + dele(2, dr("Old")))
    + p(link("other", "go")),
)
add(
    "plain-slug",
    p(ins(1, r("Title")) + bm(9, "title") + dele(2, dr("Old")))
    + p(link("title", "go")),
)

# comments
inside = (
    ins(1, '<w:commentRangeStart w:id="0"/>' + r("new") + '<w:commentRangeEnd w:id="0"/>')
    + dele(2, dr("old"))
    + cref(0)
)
cross = (
    ins(1, '<w:commentRangeStart w:id="0"/>' + r("new"))
    + dele(2, dr("old") + '<w:commentRangeEnd w:id="0"/>')
    + cref(0)
)
between = (
    ins(1, r("new"))
    + '<w:commentRangeStart w:id="0"/>'
    + dele(2, dr("old"))
    + '<w:commentRangeEnd w:id="0"/>'
    + cref(0)
)
add("c-inside", p(r("A "), inside, r(" Z")), **with_comment(None))
add("c-cross", p(r("A "), cross, r(" Z")), **with_comment(None))
add("c-between", p(r("A "), between, r(" Z")), **with_comment(None))

# tables
add(
    "t-plain",
    p(r("Before")) + table_cell(sub_around("")) + p(r("After")),
)
add(
    "t-style-vanish",
    table_cell(sub_around(raw_run('<w:rPr><w:rStyle w:val="Vanish"/></w:rPr><w:t>HIDDEN</w:t>'))),
)
add(
    "t-tab",
    table_cell(sub_around(raw_run('<w:t xml:space="preserve"></w:t><w:tab/>'))),
)
add(
    "t-lrp",
    table_cell(sub_around(raw_run("<w:lastRenderedPageBreak/>"))),
)
add(
    "t-heading",
    p(link("title", "go"))
    + table_cell(ins(1, r("Title")) + bm(9, "title") + dele(2, dr("Old")), heading=True),
)
add(
    "t-heading-1",
    p(link("title-1", "go"))
    + table_cell(ins(1, r("Title")) + bm(9, "title-1") + dele(2, dr("Old")), heading=True),
)
add("t-cross", table_cell(cross), **with_comment(None))
add("t-inside", table_cell(inside), **with_comment(None))

# headers
def header_doc(name, inner, comment=False):
    # Comments outside the body take id 7. The validator looks for the
    # comments part beside the header or footnotes part, so the four probes
    # with one (hdr-cross, hdr-between, fn-cross, fn-inside) fail it; Word
    # keeps that part on the document, as here.
    if comment:
        inner = inner.replace('w:id="0"', 'w:id="7"')
    hdr = story("header", p(inner, ppr='<w:pStyle w:val="Header"/>'))
    kw = dict(
        sect_refs='<w:headerReference w:type="default" r:id="rIdH1"/>',
        parts={"word/header1.xml": hdr},
        rels=[("rIdH1", f"{REL}/header", "header1.xml")],
        types=[("/word/header1.xml", f"{CT}.header+xml")],
    )
    if comment:
        kw["parts"]["word/comments.xml"] = comments((7, "Ann Lee", "note"))
        kw["rels"].append(COMMENTS[0])
        kw["types"].append(COMMENTS[1])
    add(name, p(r("Body.")), **kw)


header_doc("hdr-direct-vanish", sub_around(raw_run("<w:rPr><w:vanish/></w:rPr><w:t>HIDDEN</w:t>")))
header_doc("hdr-style-vanish", sub_around(raw_run('<w:rPr><w:rStyle w:val="Vanish"/></w:rPr><w:t>HIDDEN</w:t>')))
header_doc("hdr-tab", sub_around(raw_run('<w:t xml:space="preserve"></w:t><w:tab/>')))
header_doc("hdr-lrp", sub_around(raw_run("<w:lastRenderedPageBreak/>")))
header_doc("hdr-bm", ins(1, r("Title")) + bm(9, "title") + dele(2, dr("Old")))
header_doc("hdr-cross", cross, comment=True)
header_doc("hdr-between", between, comment=True)
header_doc("hdr-plain", sub_around(""))

# footnotes
def foot_doc(name, inner, heading=False, comment=False, link_to=None):
    if comment:
        inner = inner.replace('w:id="0"', 'w:id="7"')
    parts = {"word/footnotes.xml": fn_part(inner, heading=heading)}
    rels = list(FN_RELS)
    types = list(FN_TYPES)
    if comment:
        parts["word/comments.xml"] = comments((7, "Ann Lee", "note"))
        rels.append(COMMENTS[0])
        types.append(COMMENTS[1])
    body = p(r("See"), FN_REF, r("."))
    if link_to:
        body += p(link(link_to, "go"))
    add(name, body, parts=parts, rels=rels, types=types, settings=FN_SETTINGS)


foot_doc("fn-plain", sub_around(""))
foot_doc("fn-direct-vanish", sub_around(raw_run("<w:rPr><w:vanish/></w:rPr><w:t>HIDDEN</w:t>")))
foot_doc("fn-style-vanish", sub_around(raw_run('<w:rPr><w:rStyle w:val="Vanish"/></w:rPr><w:t>HIDDEN</w:t>')))
foot_doc("fn-tab", sub_around(raw_run('<w:t xml:space="preserve"></w:t><w:tab/>')))
foot_doc("fn-lrp", sub_around(raw_run("<w:lastRenderedPageBreak/>")))
foot_doc("fn-soft", sub_around(raw_run("<w:softHyphen/>")))
foot_doc("fn-heading", ins(1, r("Title")) + bm(9, "title") + dele(2, dr("Old")), heading=True, link_to="title")
foot_doc("fn-heading-1", ins(1, r("Title")) + bm(9, "title-1") + dele(2, dr("Old")), heading=True, link_to="title-1")
foot_doc("fn-cross", cross, comment=True)
foot_doc("fn-inside", inside, comment=True)

for name, (body, kw) in CASES.items():
    package(f"{name}.docx", body, **kw)
    print(name)

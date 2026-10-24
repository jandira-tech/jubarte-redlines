# Minimal .docx builder for the #1680 round-9 repro pairs (mirrors
# tests/common/docx.rs). Usage from Python: write(path, body, notes=..., ...).
import zipfile

W = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
R = "http://schemas.openxmlformats.org/officeDocument/2006/relationships"
NS = f'xmlns:w="{W}" xmlns:r="{R}" xmlns:v="urn:schemas-microsoft-com:vml"'
CT = "application/vnd.openxmlformats-officedocument.wordprocessingml."
REL = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/"


def p(text="", ppr=""):
    run = f'<w:r><w:t xml:space="preserve">{text}</w:t></w:r>' if text else ""
    return f"<w:p>{ppr}{run}</w:p>"


def tc(*blocks, w=2000):
    return f'<w:tc><w:tcPr><w:tcW w:w="{w}" w:type="dxa"/></w:tcPr>{"".join(blocks)}</w:tc>'


def tr(*cells):
    return f"<w:tr>{''.join(cells)}</w:tr>"


def tbl(*rows, cols=1, w=2000):
    grid = "".join(f'<w:gridCol w:w="{w}"/>' for _ in range(cols))
    return (f'<w:tbl><w:tblPr><w:tblW w:w="{w * cols}" w:type="dxa"/></w:tblPr>'
            f"<w:tblGrid>{grid}</w:tblGrid>{''.join(rows)}</w:tbl>")


def sdt(*blocks):
    return f"<w:sdt><w:sdtPr/><w:sdtContent>{''.join(blocks)}</w:sdtContent></w:sdt>"


def textbox(*blocks):
    return ('<w:r><w:pict><v:shape id="tb1" style="width:200pt;height:100pt">'
            f"<v:textbox><w:txbxContent>{''.join(blocks)}</w:txbxContent></v:textbox>"
            "</v:shape></w:pict></w:r>")


def write(path, body, footnotes=None, endnotes=None, header=None, footer=None):
    overrides = [("/word/document.xml", CT + "document.main+xml")]
    rels, sect = [], ""
    parts = {}
    if footnotes is not None:
        parts["word/footnotes.xml"] = f'<w:footnotes {NS}><w:footnote w:id="1">{footnotes}</w:footnote></w:footnotes>'
        overrides.append(("/word/footnotes.xml", CT + "footnotes+xml"))
        rels.append(("rIdF", "footnotes", "footnotes.xml"))
    if endnotes is not None:
        parts["word/endnotes.xml"] = f'<w:endnotes {NS}><w:endnote w:id="1">{endnotes}</w:endnote></w:endnotes>'
        overrides.append(("/word/endnotes.xml", CT + "endnotes+xml"))
        rels.append(("rIdE", "endnotes", "endnotes.xml"))
    if header is not None:
        parts["word/header1.xml"] = f"<w:hdr {NS}>{header}</w:hdr>"
        overrides.append(("/word/header1.xml", CT + "header+xml"))
        rels.append(("rIdH", "header", "header1.xml"))
        sect += '<w:headerReference w:type="default" r:id="rIdH"/>'
    if footer is not None:
        parts["word/footer1.xml"] = f"<w:ftr {NS}>{footer}</w:ftr>"
        overrides.append(("/word/footer1.xml", CT + "footer+xml"))
        rels.append(("rIdT", "footer", "footer1.xml"))
        sect += '<w:footerReference w:type="default" r:id="rIdT"/>'
    sect_pr = (f'<w:sectPr>{sect}<w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" '
               'w:bottom="1440" w:left="1440" w:header="720" w:footer="720" w:gutter="0"/></w:sectPr>')
    doc = f'<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n<w:document {NS}><w:body>{body}{sect_pr}</w:body></w:document>'
    ct = ('<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">'
          '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>'
          '<Default Extension="xml" ContentType="application/xml"/>'
          + "".join(f'<Override PartName="{n}" ContentType="{t}"/>' for n, t in overrides) + "</Types>")
    pkg = ('<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
           f'<Relationship Id="rId1" Type="{REL}officeDocument" Target="word/document.xml"/></Relationships>')
    drels = ('<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
             + "".join(f'<Relationship Id="{i}" Type="{REL}{t}" Target="{g}"/>' for i, t, g in rels) + "</Relationships>")
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as z:
        z.writestr("[Content_Types].xml", ct)
        z.writestr("_rels/.rels", pkg)
        z.writestr("word/document.xml", doc)
        z.writestr("word/_rels/document.xml.rels", drels)
        for n, x in parts.items():
            z.writestr(n, x)

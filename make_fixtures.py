"""Synthetic NDA fixtures: build original.docx + modified.docx, run jubarte over them."""
from pathlib import Path
import jubarte_redlines as jb

OUT = Path(__file__).parent / "fixtures-delivery"

def para(text, bold=False):
    b = "<w:b/>" if bold else ""
    return (f'<w:p><w:pPr></w:pPr><w:r>{"<w:rPr>" + b + "</w:rPr>" if b else ""}'
            f'<w:t xml:space="preserve">{text}</w:t></w:r></w:p>')

def docx(body, header=None):
    hdr_ref = '<w:headerReference w:type="default" r:id="rIdH1"/>' if header else ""
    document = (
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
        f'<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" '
        f'xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><w:body>{body}'
        f'<w:sectPr>{hdr_ref}<w:pgSz w:w="12240" w:h="15840"/>'
        '<w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" '
        'w:header="720" w:footer="720" w:gutter="0"/></w:sectPr></w:body></w:document>'
    )
    hdr_override = ('<Override PartName="/word/header1.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml"/>' if header else "")
    ct = ('<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
          '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">'
          '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>'
          '<Default Extension="xml" ContentType="application/xml"/>'
          '<Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>'
          f'{hdr_override}</Types>')
    rels = ('<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
            '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
            '<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>'
            '</Relationships>')
    doc_rels = ('<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
                '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
                + (f'<Relationship Id="rIdH1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/header" Target="header1.xml"/>' if header else "")
                + '</Relationships>')
    from io import BytesIO
    from zipfile import ZipFile, ZIP_DEFLATED
    buf = BytesIO()
    with ZipFile(buf, "w", ZIP_DEFLATED) as zf:
        zf.writestr("[Content_Types].xml", ct)
        zf.writestr("_rels/.rels", rels)
        zf.writestr("word/document.xml", document)
        zf.writestr("word/_rels/document.xml.rels", doc_rels)
        if header:
            zf.writestr("word/header1.xml", f'<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
                         f'<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p><w:r>'
                         f'<w:t xml:space="preserve">{header}</w:t></w:r></w:p></w:hdr>')
    return buf.getvalue()

original = docx(
    para("MUTUAL NON-DISCLOSURE AGREEMENT", bold=True)
    + para("This Agreement is entered into as of January 1, 2026 between Acme Corp. and Globex Inc.")
    + para("The Receiving Party shall hold all Confidential Information in strict confidence for a period of two (2) years.")
    + para("Confidential Information means any non-public technical or business information disclosed by the Disclosing Party.")
    + para("This Agreement shall be governed by the laws of the State of New York.")
    + para("IN WITNESS WHEREOF, the parties have executed this Agreement."),
    header="ACME CORP. \u2013 CONFIDENTIAL",
)

modified = docx(
    para("MUTUAL NON-DISCLOSURE AGREEMENT", bold=True)
    + para("This Agreement is entered into as of January 1, 2026 between Acme Corp. and Globex Inc.")
    + para("The Receiving Party shall hold all Confidential Information in strict confidence for a period of three (3) years.")
    + para("Confidential Information means any non-public technical, business or financial information disclosed by the Disclosing Party.")
    + para("The Receiving Party may not disclose Confidential Information to any third party without prior written consent.")
    + para("This Agreement shall be governed by the laws of the State of New York.")
    + para("IN WITNESS WHEREOF, the parties have executed this Agreement."),
    header="ACME CORP. \u2013 CONFIDENTIAL",
)

OUT.mkdir(exist_ok=True)
(OUT / "original.docx").write_bytes(original)
(OUT / "modified.docx").write_bytes(modified)

# 1) Word-mode redline
redline = jb.compare_documents(original, modified, author="Counsel", date="2026-02-01T09:00:00Z")
(OUT / "redline.docx").write_bytes(redline)

# 2) Accept-all / reject-all
(OUT / "accepted.docx").write_bytes(jb.accept_revisions(redline))
(OUT / "rejected.docx").write_bytes(jb.reject_revisions(redline))

# 3) PDF renders
(OUT / "original.pdf").write_bytes(jb.docx_to_pdf(original))
(OUT / "modified.pdf").write_bytes(jb.docx_to_pdf(modified))
(OUT / "redline.pdf").write_bytes(jb.docx_to_pdf(redline))

# 4) Revision report
import json
revs = json.loads(jb.get_revisions_json(redline))
(OUT / "revisions.json").write_text(json.dumps(revs, indent=2))
print("revisions:")
for r in revs:
    print(" ", r.get("type"), repr((r.get("text") or "")[:60]))

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Write form.docx, a synthetic intake form with six content controls.

Standard library only: ``python3 make_form.py`` (or ``uv run make_form.py``).
The output is byte-for-byte deterministic, so the plan's ``source_sha256`` can
be bound once and stays valid.
"""

from __future__ import annotations

import sys
import zipfile
from pathlib import Path
from xml.sax.saxutils import escape

W = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
W14 = "http://schemas.microsoft.com/office/word/2010/wordml"
MC = "http://schemas.openxmlformats.org/markup-compatibility/2006"

PLACEHOLDER = '<w:rPr><w:rStyle w:val="PlaceholderText"/></w:rPr>'


def run(text: str, rpr: str = "") -> str:
    return f'<w:r>{rpr}<w:t xml:space="preserve">{escape(text)}</w:t></w:r>'


def label(text: str) -> str:
    return run(text, "<w:rPr><w:b/><w:bCs/></w:rPr>")


def inline(pr: str, content: str) -> str:
    return f"<w:sdt><w:sdtPr>{pr}</w:sdtPr><w:sdtContent>{content}</w:sdtContent></w:sdt>"


def paragraph(body: str, style: str = "") -> str:
    ppr = f'<w:pPr><w:pStyle w:val="{style}"/></w:pPr>' if style else ""
    return f"<w:p>{ppr}{body}</w:p>"


BODY = "".join(
    [
        paragraph(run("Client Intake Form"), "Title"),
        paragraph(
            label("Full name: ")
            + inline(
                '<w:alias w:val="Full name"/><w:tag w:val="Name"/><w:id w:val="101"/>'
                "<w:showingPlcHdr/><w:text/>",
                run("Click or tap here to enter text.", PLACEHOLDER),
            )
        ),
        paragraph(
            label("Country: ")
            + inline(
                '<w:alias w:val="Country"/><w:tag w:val="Country"/><w:id w:val="102"/><w:showingPlcHdr/>'
                '<w:dropDownList><w:listItem w:displayText="Choose an item." w:value=""/>'
                '<w:listItem w:displayText="Brazil" w:value="BR"/>'
                '<w:listItem w:displayText="Chile" w:value="CL"/>'
                '<w:listItem w:displayText="Portugal" w:value="PT"/></w:dropDownList>',
                run("Choose an item.", PLACEHOLDER),
            )
        ),
        paragraph(
            inline(
                '<w:tag w:val="Consent"/><w:id w:val="103"/>'
                '<w14:checkbox><w14:checked w14:val="0"/>'
                '<w14:checkedState w14:val="2612" w14:font="MS Gothic"/>'
                '<w14:uncheckedState w14:val="2610" w14:font="MS Gothic"/></w14:checkbox>',
                run(
                    "☐",
                    '<w:rPr><w:rFonts w:ascii="MS Gothic" w:eastAsia="MS Gothic" w:hAnsi="MS Gothic" w:hint="eastAsia"/></w:rPr>',
                ),
            )
            + run(" I consent to the engagement terms below.")
        ),
        paragraph(
            label("Signed on: ")
            + inline(
                '<w:alias w:val="Signature date"/><w:tag w:val="Signed"/><w:id w:val="104"/><w:showingPlcHdr/>'
                '<w:date><w:dateFormat w:val="d MMMM yyyy"/><w:lid w:val="en-US"/>'
                '<w:storeMappedDataAs w:val="dateTime"/><w:calendar w:val="gregorian"/></w:date>',
                run("Click or tap to enter a date.", PLACEHOLDER),
            )
        ),
        paragraph(label("Matter description"), "Heading1"),
        inline(
            '<w:alias w:val="Matter"/><w:tag w:val="Matter"/><w:id w:val="105"/><w:showingPlcHdr/><w:richText/>',
            paragraph(run("Describe the matter in a few sentences.", PLACEHOLDER))
            + paragraph(run("Add parties and deadlines.", PLACEHOLDER)),
        ),
        paragraph(
            label("Form reference: ")
            + inline(
                '<w:tag w:val="Ref"/><w:id w:val="106"/><w:lock w:val="sdtContentLocked"/><w:text/>',
                run("INTAKE-2026-v3"),
            )
        ),
    ]
)

STYLES = f"""<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="{W}"><w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:ascii="Calibri" w:hAnsi="Calibri" w:cs="Calibri"/><w:sz w:val="22"/><w:szCs w:val="22"/><w:lang w:val="en-US"/></w:rPr></w:rPrDefault><w:pPrDefault><w:pPr><w:spacing w:after="160" w:line="259" w:lineRule="auto"/></w:pPr></w:pPrDefault></w:docDefaults><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:qFormat/></w:style><w:style w:type="paragraph" w:styleId="Title"><w:name w:val="Title"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:rPr><w:sz w:val="40"/><w:szCs w:val="40"/></w:rPr></w:style><w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/><w:spacing w:before="240" w:after="80"/><w:outlineLvl w:val="0"/></w:pPr><w:rPr><w:sz w:val="28"/><w:szCs w:val="28"/></w:rPr></w:style><w:style w:type="character" w:styleId="PlaceholderText"><w:name w:val="Placeholder Text"/><w:uiPriority w:val="99"/><w:semiHidden/><w:rPr><w:color w:val="808080"/></w:rPr></w:style></w:styles>"""

CONTENT_TYPES = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/><Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/></Types>"""

ROOT_RELS = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"""

DOC_RELS = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/></Relationships>"""


def document() -> str:
    sect = '<w:sectPr><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720" w:gutter="0"/></w:sectPr>'
    return (
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n'
        f'<w:document xmlns:w="{W}" xmlns:w14="{W14}" xmlns:mc="{MC}" mc:Ignorable="w14">'
        f"<w:body>{BODY}{sect}</w:body></w:document>"
    )


def main() -> None:
    out = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).with_name("form.docx")
    parts = [
        ("[Content_Types].xml", CONTENT_TYPES),
        ("_rels/.rels", ROOT_RELS),
        ("word/document.xml", document()),
        ("word/_rels/document.xml.rels", DOC_RELS),
        ("word/styles.xml", STYLES),
    ]
    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as z:
        for name, xml in parts:
            info = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            z.writestr(info, xml)
    print(out)


if __name__ == "__main__":
    main()

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Write letter.docx, the synthetic customer-disclosure letter plan.json edits.

Standard library only: ``python3 make_letter.py`` (or ``uv run make_letter.py``).
The output is byte-for-byte deterministic, so the plan's ``source_sha256`` can
be bound once and stays valid.
"""

from __future__ import annotations

import sys
import zipfile
from pathlib import Path
from xml.sax.saxutils import escape

W = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"

# (style, [(text, bold)]) per body paragraph, in document order.
PARAGRAPHS: list[tuple[str, list[tuple[str, bool]]]] = [
    ("Heading1", [("3. Restricted Use of the Information", False)]),
    (
        "Normal",
        [
            ("(f) Notice of Inability to Comply. ", True),
            (
                "If you determine that you can no longer meet your obligations under this Section 3, "
                "you will notify us in writing within five business days and stop processing the Information.",
                False,
            ),
        ],
    ),
    ("Heading1", [("4. Onward Transfers", False)]),
    (
        "Normal",
        [
            ("(c) Subprocessors. ", True),
            ("You will not engage a subprocessor for the Information without our prior written consent.", False),
        ],
    ),
    (
        "Normal",
        [
            ("(d) Onward Disclosure. ", True),
            (
                "You will not disclose the Information to any third party other than your attorneys, "
                "retained experts, and process servers, and you remain responsible for their handling of it.",
                False,
            ),
        ],
    ),
    ("Heading1", [("5. Confidentiality and Security", False)]),
    (
        "Normal",
        [
            ("(a) Confidentiality. ", True),
            (
                "You will keep the Information confidential and may disclose it only to your attorneys, "
                "retained experts, and process servers who need it to pursue the Permitted Purpose, "
                "and only after you give each of them the notice Section 5(d) requires.",
                False,
            ),
        ],
    ),
    ("Heading1", [("11. General", False)]),
    (
        "Normal",
        [
            ("(e) Survival. ", True),
            ("Sections 1(g), 2(e), 3, 4, 5, 6, 8, 9, 10, and 11 survive termination of this letter.", False),
        ],
    ),
    (
        "Normal",
        [
            ("(f) Notices. ", True),
            (
                "Notices under this letter go by email to legal@acme.example in our case and to the "
                "address in the heading of this letter in your case, with a copy by a nationally "
                "recognized overnight courier. A notice takes effect on the business day the recipient receives it.",
                False,
            ),
        ],
    ),
    ("Normal", [("(g) Waiver of Jury Trial.", True)]),
    (
        "Normal",
        [
            (
                "Each party waives, to the fullest extent the law allows, any right to a trial by jury "
                "in any action arising out of or relating to this letter.",
                False,
            )
        ],
    ),
    (
        "DraftingNote",
        [
            (
                "[Drafting note: the requester signs individually as well as for its organization, "
                "so the signature block below keeps both capacities.]",
                False,
            )
        ],
    ),
    (
        "Normal",
        [
            (
                "The individual who signs this letter for the requester named above also signs in "
                "his or her individual capacity.",
                False,
            )
        ],
    ),
]

STYLES = f"""<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="{W}"><w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:cs="Times New Roman"/><w:sz w:val="22"/><w:szCs w:val="22"/><w:lang w:val="en-US"/></w:rPr></w:rPrDefault><w:pPrDefault><w:pPr><w:spacing w:after="120" w:line="264" w:lineRule="auto"/><w:jc w:val="both"/></w:pPr></w:pPrDefault></w:docDefaults><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:qFormat/></w:style><w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/><w:spacing w:before="240" w:after="120"/><w:jc w:val="left"/><w:outlineLvl w:val="0"/></w:pPr><w:rPr><w:b/><w:bCs/></w:rPr></w:style><w:style w:type="paragraph" w:customStyle="1" w:styleId="DraftingNote"><w:name w:val="Drafting Note"/><w:basedOn w:val="Normal"/><w:pPr><w:spacing w:after="240" w:line="360" w:lineRule="auto"/><w:ind w:left="720"/></w:pPr><w:rPr><w:i/><w:iCs/><w:color w:val="1F4E79"/></w:rPr></w:style></w:styles>"""

CONTENT_TYPES = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/><Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/></Types>"""

ROOT_RELS = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"""

DOC_RELS = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/></Relationships>"""


def paragraph(style: str, runs: list[tuple[str, bool]]) -> str:
    ppr = "" if style == "Normal" else f'<w:pPr><w:pStyle w:val="{style}"/></w:pPr>'
    body = "".join(
        f'<w:r>{"<w:rPr><w:b/><w:bCs/></w:rPr>" if bold else ""}<w:t xml:space="preserve">{escape(text)}</w:t></w:r>'
        for text, bold in runs
    )
    return f"<w:p>{ppr}{body}</w:p>"


def document() -> str:
    body = "".join(paragraph(style, runs) for style, runs in PARAGRAPHS)
    sect = '<w:sectPr><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720" w:gutter="0"/></w:sectPr>'
    return f'<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n<w:document xmlns:w="{W}"><w:body>{body}{sect}</w:body></w:document>'


def main() -> None:
    out = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).with_name("letter.docx")
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

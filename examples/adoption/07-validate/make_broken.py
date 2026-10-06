#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Write three broken .docx files, one defect each, standard library only.

- broken_a_no_ins_id.docx      a w:ins with no w:id
- broken_b_dangling_rid.docx   document.xml with a dangling r:id image
- broken_c_no_ct_override.docx [Content_Types].xml without the override for
                               /word/document.xml

The base package is the minimum a reader expects: content types, the root
relationship, and word/document.xml with two plain paragraphs.
"""

from __future__ import annotations

import zipfile
from pathlib import Path

HERE = Path(__file__).resolve().parent

W = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
R = "http://schemas.openxmlformats.org/officeDocument/2006/relationships"

CONTENT_TYPES = (
    '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n'
    '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">'
    '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>'
    '<Default Extension="xml" ContentType="application/xml"/>'
    '<Override PartName="/word/document.xml" ContentType="application/vnd'
    ".openxmlformats-officedocument.wordprocessingml.document.main+xml\"/>"
    "</Types>"
)

ROOT_RELS = (
    '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n'
    '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
    '<Relationship Id="rId1" Type="http://schemas.openxmlformats.org'
    '/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>'
    "</Relationships>"
)

DOCUMENT_OPEN = (
    '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n'
    f'<w:document xmlns:w="{W}" xmlns:r="{R}"'
    ' xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing">'
    "<w:body>"
    "<w:p><w:r><w:t>First paragraph.</w:t></w:r></w:p>"
    "<w:p><w:r><w:t>Second paragraph.</w:t></w:r></w:p>"
)
DOCUMENT_CLOSE = "</w:body></w:document>"

# (a) a tracked insertion with no w:id, no w:author, no w:date
INS_NO_ID = (
    "<w:p><w:ins><w:r><w:t>Inserted without an id.</w:t></w:r></w:ins></w:p>"
)

# (b) an inline picture whose blip points at rIdImage9, a relationship the
# package never defines
BLIP_DANGLING = (
    "<w:p><w:r><w:drawing>"
    '<wp:inline distT="0" distB="0" distL="0" distR="0">'
    '<wp:extent cx="914400" cy="914400"/>'
    '<wp:docPr id="1" name="Picture 1"/>'
    '<a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">'
    '<a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture">'
    '<pic:pic xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture">'
    '<pic:nvPicPr><pic:cNvPr id="1" name="p"/><pic:cNvPicPr/></pic:nvPicPr>'
    '<pic:blipFill><a:blip r:embed="rIdImage9"/>'
    "<a:stretch><a:fillRect/></a:stretch></pic:blipFill>"
    '<pic:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="914400" cy="914400"/></a:xfrm>'
    '<a:prstGeom prst="rect"><a:avLst/></a:prstGeom></pic:spPr>'
    "</pic:pic></a:graphicData></a:graphic></wp:inline>"
    "</w:drawing></w:r></w:p>"
)


def write_docx(path: Path, document: str, content_types: str = CONTENT_TYPES) -> None:
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as z:
        z.writestr("[Content_Types].xml", content_types)
        z.writestr("_rels/.rels", ROOT_RELS)
        z.writestr("word/document.xml", document)


def main() -> None:
    write_docx(HERE / "broken_a_no_ins_id.docx",
               DOCUMENT_OPEN + INS_NO_ID + DOCUMENT_CLOSE)
    write_docx(HERE / "broken_b_dangling_rid.docx",
               DOCUMENT_OPEN + BLIP_DANGLING + DOCUMENT_CLOSE)
    # (c) the same package, but the content-type override for the main part
    # is missing, so /word/document.xml falls under Default xml
    write_docx(HERE / "broken_c_no_ct_override.docx",
               DOCUMENT_OPEN + DOCUMENT_CLOSE,
               CONTENT_TYPES.replace(
                   '<Override PartName="/word/document.xml" ContentType="application/vnd'
                   ".openxmlformats-officedocument.wordprocessingml.document.main+xml\"/>",
                   "",
               ))
    for p in sorted(HERE.glob("broken_*.docx")):
        print(f"wrote {p.name}")


if __name__ == "__main__":
    main()

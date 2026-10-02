# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Write the accept-spacer-paragraph documents: a numbered item deleted whole.

Standard library only. ``python3 make_redline.py`` writes ``redline.docx``
(the ``base`` case); ``python3 make_redline.py --all DIR`` writes every case
in ``CASES`` to ``DIR/<name>.docx``. The bytes are deterministic.

Each case deletes a numbered list item outright: its run and its paragraph
mark are tracked deletions, so accepting them must remove the paragraph and
leave no empty numbered paragraph. Anthropic's docx skill
(``skills/docx/SKILL.md``, fetched 2026-10-02) says its LibreOffice path
``accept_changes.py`` "joins them correctly, except when the deleted
paragraph is followed by an empty spacer paragraph"; these cases put a
spacer (or its variants) after the deleted item.
"""

from __future__ import annotations

import argparse
import zipfile
from pathlib import Path

W = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
STAMP = 'w:author="Reviewer" w:date="2026-10-02T00:00:00Z"'


def numbered(mark_rpr: str = "") -> str:
    return f'<w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="1"/></w:numPr>{mark_rpr}</w:pPr>'


def keep(text: str) -> str:
    """A numbered list item."""
    return f"<w:p>{numbered()}<w:r><w:t>{text}</w:t></w:r></w:p>"


def deleted(text: str, rev: int) -> str:
    """A numbered list item whose run and paragraph mark are deleted."""
    mark = f'<w:rPr><w:del w:id="{rev}" {STAMP}/></w:rPr>'
    return (
        f'<w:p>{numbered(mark)}<w:del w:id="{rev + 1}" {STAMP}>'
        f"<w:r><w:delText>{text}</w:delText></w:r></w:del></w:p>"
    )


def plain(text: str) -> str:
    return f"<w:p><w:r><w:t>{text}</w:t></w:r></w:p>"


SPACER = "<w:p/>"

# name -> body paragraphs. `base` is the case the skill describes.
CASES: dict[str, str] = {
    "base": keep("Keep this item")
    + deleted("Delete this item", 1)
    + SPACER
    + plain("Next section"),
    "spacer_with_spacing": keep("Keep this item")
    + deleted("Delete this item", 1)
    + '<w:p><w:pPr><w:spacing w:after="0"/></w:pPr></w:p>'
    + plain("Next section"),
    "numbered_item_after": keep("Keep this item")
    + deleted("Delete this item", 1)
    + SPACER
    + keep("Third item"),
    "two_spacers": keep("Keep this item")
    + deleted("Delete this item", 1)
    + SPACER
    + SPACER
    + plain("Next section"),
    "first_item_deleted": deleted("Delete this item", 1)
    + SPACER
    + plain("Next section"),
    "spacer_last_in_body": keep("Keep this item")
    + deleted("Delete this item", 1)
    + SPACER,
    "two_items_deleted": keep("Keep this item")
    + deleted("Delete this item", 1)
    + deleted("And this one", 3)
    + SPACER
    + plain("Next section"),
}

SECT = (
    '<w:sectPr><w:pgSz w:w="12240" w:h="15840"/>'
    '<w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" '
    'w:header="720" w:footer="720" w:gutter="0"/></w:sectPr>'
)

NUMBERING = (
    '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n'
    f'<w:numbering xmlns:w="{W}">'
    '<w:abstractNum w:abstractNumId="0"><w:multiLevelType w:val="singleLevel"/>'
    '<w:lvl w:ilvl="0"><w:start w:val="1"/><w:numFmt w:val="decimal"/>'
    '<w:lvlText w:val="%1."/><w:lvlJc w:val="left"/>'
    '<w:pPr><w:ind w:left="720" w:hanging="360"/></w:pPr></w:lvl></w:abstractNum>'
    '<w:num w:numId="1"><w:abstractNumId w:val="0"/></w:num>'
    "</w:numbering>"
)

CONTENT_TYPES = (
    '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n'
    '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">'
    '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>'
    '<Default Extension="xml" ContentType="application/xml"/>'
    '<Override PartName="/word/document.xml" '
    'ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>'
    '<Override PartName="/word/numbering.xml" '
    'ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml"/>'
    "</Types>"
)

PACKAGE_RELS = (
    '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n'
    '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
    '<Relationship Id="rId1" '
    'Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" '
    'Target="word/document.xml"/></Relationships>'
)

DOCUMENT_RELS = (
    '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n'
    '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
    '<Relationship Id="rId1" '
    'Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering" '
    'Target="numbering.xml"/></Relationships>'
)


def write(path: Path, body: str) -> None:
    """Write one package with fixed timestamps so the bytes never change."""
    document = (
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n'
        f'<w:document xmlns:w="{W}"><w:body>{body}{SECT}</w:body></w:document>'
    )
    parts = [
        ("[Content_Types].xml", CONTENT_TYPES),
        ("_rels/.rels", PACKAGE_RELS),
        ("word/document.xml", document),
        ("word/_rels/document.xml.rels", DOCUMENT_RELS),
        ("word/numbering.xml", NUMBERING),
    ]
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as zf:
        for name, data in parts:
            info = zipfile.ZipInfo(name, date_time=(2026, 10, 2, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            zf.writestr(info, data.encode("utf-8"))


def main(argv: list[str] | None = None) -> None:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "out",
        nargs="?",
        type=Path,
        default=Path(__file__).with_name("redline.docx"),
        help="where to write the base case (default: redline.docx here)",
    )
    parser.add_argument(
        "--all",
        metavar="DIR",
        type=Path,
        help="write every case to DIR/<name>.docx instead",
    )
    args = parser.parse_args(argv)
    if args.all is not None:
        args.all.mkdir(parents=True, exist_ok=True)
        for case, case_body in CASES.items():
            write(args.all / f"{case}.docx", case_body)
            print(args.all / f"{case}.docx")
    else:
        write(args.out, CASES["base"])
        print(args.out)


if __name__ == "__main__":
    main()

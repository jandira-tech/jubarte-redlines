# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""A .doc (or encrypted document) is LEGACY_DOC and RTF is UNSUPPORTED_PACKAGE,
code first, on every Document method."""

import pytest

import jubarte_redlines as jubarte
from test_document import make_document

OLE = b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1" + bytes(4088)


@pytest.mark.parametrize(
    "call",
    [
        lambda d, o: d.compare(o, author="A"),
        lambda d, o: o.compare(d, author="A"),
        lambda d, o: d.accept(),
        lambda d, o: d.reject(),
        lambda d, o: d.changes(),
        lambda d, o: d.revisions(),
        lambda d, o: d.to_pdf(),
        lambda d, o: d.to_png(dpi=40),
        lambda d, o: d.render(pdf=False),
        lambda d, o: d.inspect(),
        lambda d, o: d.markdown(),
        lambda d, o: o.append(d),
    ],
    ids=[
        "compare-original",
        "compare-modified",
        "accept",
        "reject",
        "changes",
        "revisions",
        "to_pdf",
        "to_png",
        "render",
        "inspect",
        "markdown",
        "append",
    ],
)
def test_every_document_method_names_a_legacy_doc(call) -> None:
    doc = jubarte.Document.from_bytes(OLE)
    other = jubarte.Document.from_bytes(make_document("y"))
    with pytest.raises(jubarte.JubarteError, match=r"^LEGACY_DOC: .*save it as \.docx"):
        call(doc, other)


def test_comments_name_a_legacy_doc_with_the_hint() -> None:
    with pytest.raises(jubarte.JubarteError, match=r"save it as \.docx"):
        jubarte.Document.from_bytes(OLE).comments()


def test_rtf_is_unsupported_package_code_first() -> None:
    rtf = jubarte.Document.from_bytes(b"{\\rtf1\\ansi hello}")
    with pytest.raises(jubarte.JubarteError, match=r"^UNSUPPORTED_PACKAGE: .*an RTF file"):
        rtf.to_pdf()
    with pytest.raises(jubarte.JubarteError, match=r"^UNSUPPORTED_PACKAGE: "):
        rtf.compare(jubarte.Document.from_bytes(make_document("y")), author="A")

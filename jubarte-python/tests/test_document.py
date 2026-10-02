# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""All unit fixtures stay in memory; filesystem reads are integration tests."""

from dataclasses import FrozenInstanceError
from io import BytesIO
from xml.etree import ElementTree
from xml.sax.saxutils import escape
from zipfile import ZipFile, ZipInfo

import pytest

import jubarte_redlines as jubarte

W = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"


def make_document(text: str) -> bytes:
    parts = {
        "[Content_Types].xml": (
            '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">'
            '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>'
            '<Default Extension="xml" ContentType="application/xml"/>'
            '<Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>'
            '</Types>'
        ),
        "_rels/.rels": (
            '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
            '<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>'
            '</Relationships>'
        ),
        "word/document.xml": (
            f'<w:document xmlns:w="{W}"><w:body><w:p><w:r><w:t>'
            f'{escape(text)}</w:t></w:r></w:p><w:sectPr/></w:body></w:document>'
        ),
    }
    data = BytesIO()
    with ZipFile(data, "w") as package:
        for path, content in parts.items():
            # ZipInfo's fixed DOS epoch avoids the wall clock in unit fixtures.
            package.writestr(ZipInfo(path), content)
    return data.getvalue()


def visible_text(data: bytes) -> str:
    with ZipFile(BytesIO(data)) as package:
        root = ElementTree.fromstring(package.read("word/document.xml"))
    return "".join(node.text or "" for node in root.iter(f"{{{W}}}t"))


def test_document_is_an_immutable_snapshot_with_safe_repr():
    data = make_document("private contract")
    document = jubarte.Document.from_bytes(data)
    assert document.to_bytes() is data
    assert repr(document) == f"Document(size={len(data)})"
    assert "private contract" not in repr(document)
    with pytest.raises(FrozenInstanceError):
        document._data = b"different"


@pytest.mark.parametrize("value", ["contract.docx", bytearray(b"zip"), memoryview(b"zip")])
def test_document_rejects_paths_and_mutable_buffers_as_bytes(value):
    with pytest.raises(TypeError, match="requires bytes"):
        jubarte.Document.from_bytes(value)


def test_compare_round_trip_preserves_both_inputs_and_revision_metadata():
    old = jubarte.Document.from_bytes(make_document("The price is ten dollars."))
    new = jubarte.Document.from_bytes(make_document("The price is twenty dollars."))
    old_bytes, new_bytes = old.to_bytes(), new.to_bytes()
    redline = old.compare(
        new, author="Reviewer", options=jubarte.CompareOptions(date="2026-09-26T14:30:00Z")
    )
    assert visible_text(redline.accept().to_bytes()) == "The price is twenty dollars."
    assert visible_text(redline.reject().to_bytes()) == "The price is ten dollars."
    revisions = redline.revisions()
    assert {row.kind for row in revisions} >= {"Inserted", "Deleted"}
    assert {row.author for row in revisions} == {"Reviewer"}
    assert {row.date for row in revisions} == {"2026-09-26T14:30:00Z"}
    assert old.to_bytes() == old_bytes
    assert new.to_bytes() == new_bytes


@pytest.mark.integration
def test_byte_api_round_trips_a_rewritten_and_an_inserted_paragraph():
    # PR #253's NDA fixture run: several paragraphs, one word changed, one
    # paragraph rewritten, one inserted. Accept-all reads as the modified
    # text and reject-all as the original, through the byte functions.
    from docx_fixture import docx, para

    original = docx(
        para("The term is two years.") + para("Confidential means technical data.") + para("Kept."),
        header="Mutual NDA",
    )
    modified = docx(
        para("The term is three years.")
        + para("Confidential means technical, business and financial data.")
        + para("No disclosure to third parties.")
        + para("Kept."),
        header="Mutual NDA",
    )
    redline = jubarte.compare_documents(original, modified, author="Counsel")
    markdown = lambda data: jubarte.Document.from_bytes(data).markdown()  # noqa: E731
    assert markdown(jubarte.accept_revisions(redline)) == markdown(modified)
    assert markdown(jubarte.reject_revisions(redline)) == markdown(original)
    revisions = jubarte.get_revisions(redline)
    assert revisions and {row["author"] for row in revisions} == {"Counsel"}


def test_compare_default_is_repeatable_and_keeps_legacy_fixed_date():
    old = jubarte.Document.from_bytes(make_document("The payment is due today."))
    new = jubarte.Document.from_bytes(make_document("The payment is due tomorrow."))
    first = old.compare(new, author="Reviewer")
    second = old.compare(new, author="Reviewer")
    with ZipFile(BytesIO(first.to_bytes())) as left, ZipFile(BytesIO(second.to_bytes())) as right:
        assert {name: left.read(name) for name in left.namelist()} == {
            name: right.read(name) for name in right.namelist()
        }
    assert first.revisions() == second.revisions()
    assert {row.date for row in first.revisions()} == {"1970-01-01T00:00:00Z"}


@pytest.mark.parametrize(
    ("kwargs", "error", "message"),
    [
        ({"modified": b"not a Document", "author": "A"}, TypeError, "Document"),
        ({"author": None}, TypeError, "author"),
        ({"author": "  "}, ValueError, "author"),
        ({"author": "A", "options": {}}, TypeError, "CompareOptions"),
    ],
)
def test_compare_rejects_bad_arguments_before_computing(kwargs, error, message):
    document = jubarte.Document.from_bytes(make_document("unchanged"))
    args = {"modified": document, **kwargs}
    with pytest.raises(error, match=message):
        document.compare(**args)


@pytest.mark.integration
def test_pdf_uses_native_renderer_without_writing_a_file():
    document = jubarte.Document.from_bytes(make_document("A PDF rendered in memory."))
    assert document.to_pdf().startswith(b"%PDF-")
    assert document.to_pdf(options=jubarte.PdfOptions(compress=True, revisions="word")).startswith(b"%PDF-")
    with pytest.raises(TypeError, match="PdfOptions"):
        document.to_pdf(options={})


def test_native_document_errors_keep_the_public_exception():
    document = jubarte.Document.from_bytes(b"not an OPC package")
    with pytest.raises(jubarte.JubarteError):
        document.accept()
    with pytest.raises(jubarte.JubarteError):
        document.to_pdf()


@pytest.mark.integration
def test_legacy_byte_and_dictionary_api_remains_available():
    data = make_document("unchanged")
    assert isinstance(jubarte.accept_revisions(data), bytes)
    assert isinstance(jubarte.reject_revisions(data), bytes)
    assert jubarte.get_revisions(data) == []
    assert jubarte.get_revisions_json(data) == "[]"
    assert isinstance(jubarte.compare_documents(data, data), bytes)
    assert jubarte.docx_to_pdf(data).startswith(b"%PDF-")


@pytest.mark.integration
def test_read_path_and_top_level_read(tmp_path):
    data = make_document("Read me.")
    source = tmp_path / "input.docx"
    source.write_bytes(data)
    assert jubarte.Document.read(source).to_bytes() == data
    assert jubarte.read(str(source)).to_bytes() == data
    with pytest.raises(FileNotFoundError):
        jubarte.read(tmp_path / "missing.docx")


def with_extra_entries(package: bytes, count: int) -> bytes:
    out = BytesIO()
    with ZipFile(BytesIO(package)) as source, ZipFile(out, "w") as target:
        for item in source.infolist():
            target.writestr(item, source.read(item.filename))
        for index in range(count):
            target.writestr(ZipInfo(f"word/media/p{index}.bin"), b"x")
    return out.getvalue()


def test_compare_refuses_a_package_with_too_many_entries():
    # The entry count is read from the central directory before anything is
    # inflated; the refusal is a catchable JubarteError, not an abort.
    crowded = with_extra_entries(make_document("hello"), 10_010)
    with pytest.raises(jubarte.JubarteError, match="INPUT_LIMIT"):
        jubarte.compare_documents(make_document("hello"), crowded)
    with pytest.raises(jubarte.JubarteError, match="INPUT_LIMIT"):
        jubarte.compare_documents(crowded, make_document("hello"))


def test_compare_still_accepts_a_well_formed_pair():
    redline = jubarte.compare_documents(make_document("hello"), make_document("hello there"))
    assert redline.startswith(b"PK")

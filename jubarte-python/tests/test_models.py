# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

import json
from dataclasses import FrozenInstanceError
from datetime import datetime, timedelta, timezone

import pytest

from jubarte_redlines import CompareOptions, FormatChange, PdfOptions, Revision
from jubarte_redlines.models import _decode_revisions


def test_compare_defaults_keep_engine_timestamp():
    assert CompareOptions().native_date() is None


@pytest.mark.parametrize(
    "value",
    [
        "2026-09-26T10:30:00-04:00",
        "2026-09-26T14:30:00Z",
        datetime(2026, 9, 26, 10, 30, tzinfo=timezone(timedelta(hours=-4))),
    ],
)
def test_explicit_dates_are_normalized_without_reading_the_clock(value):
    options = CompareOptions(date=value)
    assert options.native_date() == "2026-09-26T14:30:00Z"


def test_timestamp_preserves_fractional_seconds():
    assert CompareOptions(date="2026-09-26T14:30:00.123456Z").native_date() == (
        "2026-09-26T14:30:00.123456Z"
    )


@pytest.mark.parametrize("value", ["yesterday", "2026-02-30T12:00:00Z"])
def test_bad_timestamp_is_value_error(value):
    with pytest.raises(ValueError, match="ISO-8601"):
        CompareOptions(date=value)


@pytest.mark.parametrize("value", ["2026-09-26", datetime(2026, 9, 26)])
def test_naive_timestamp_is_rejected(value):
    with pytest.raises(ValueError, match="offset"):
        CompareOptions(date=value)


def test_non_timestamp_is_type_error():
    with pytest.raises(TypeError, match="date"):
        CompareOptions(date=123)


def test_options_are_frozen():
    options = CompareOptions()
    with pytest.raises(FrozenInstanceError):
        options.date = "2026-09-26T14:30:00Z"
    pdf = PdfOptions()
    with pytest.raises(FrozenInstanceError):
        pdf.compress = True


def test_pdf_defaults_preserve_current_behavior():
    assert PdfOptions() == PdfOptions(compress=False, revisions="conventional")
    assert PdfOptions(revisions="word").revision_palette is None
    assert PdfOptions(revisions="custom", revision_palette="deleted=#AA0000:strike").revisions == "custom"


@pytest.mark.parametrize(
    ("options", "error", "message"),
    [
        ({"compress": 1}, TypeError, "compress"),
        ({"revisions": "hidden"}, ValueError, "revisions"),
        ({"revision_palette": 1}, TypeError, "revision_palette"),
        ({"revisions": "custom"}, ValueError, "require"),
        ({"revision_palette": "deleted=#AA0000:strike"}, ValueError, "requires"),
    ],
)
def test_pdf_options_fail_before_native_work(options, error, message):
    with pytest.raises(error, match=message):
        PdfOptions(**options)


def test_revision_decoding_preserves_metadata_and_freezes_nested_properties():
    payload = json.dumps([
        {
            "type": "FormatChanged",
            "author": "Reviewer",
            "date": "producer-specific-date",
            "part": "word/document.xml",
            "text": "changed\ttext\n\"quoted\"",
            "moveGroupId": None,
            "isMoveSource": None,
            "formatChange": {"changedProperties": ["bold", "spacing"]},
        },
        {
            "type": "Moved",
            "author": "",
            "date": "",
            "part": "word/footnotes.xml",
            "text": "moved",
            "moveGroupId": 7,
            "isMoveSource": False,
            "formatChange": None,
        },
    ])
    records = _decode_revisions(payload)
    assert records == (
        Revision(
            kind="FormatChanged", author="Reviewer", date="producer-specific-date",
            part="word/document.xml", text="changed\ttext\n\"quoted\"",
            move_group_id=None, is_move_source=None,
            format_change=FormatChange(("bold", "spacing")),
        ),
        Revision(
            kind="Moved", author="", date="", part="word/footnotes.xml", text="moved",
            move_group_id=7, is_move_source=False, format_change=None,
        ),
    )
    with pytest.raises(FrozenInstanceError):
        records[0].author = "Other"
    with pytest.raises(FrozenInstanceError):
        records[0].format_change.changed_properties = ()
    assert _decode_revisions("[]") == ()


def test_native_schema_mismatch_is_not_silently_dropped():
    with pytest.raises(KeyError):
        _decode_revisions('[{"type":"Inserted"}]')


def test_compare_options_input_limits_are_an_immutable_copy() -> None:
    source = {"max_entries": 5}
    options = CompareOptions(input_limits=source)
    source["max_entries"] = 9
    assert options.native_input_limits() == {"max_entries": 5}
    assert CompareOptions().native_input_limits() is None
    hash(options)


@pytest.mark.parametrize("value", [{"max_entries": -1}, {"max_entries": True}, {1: 2}, [1]])
def test_compare_options_input_limits_reject_bad_values(value: object) -> None:
    with pytest.raises((TypeError, ValueError)):
        CompareOptions(input_limits=value)  # type: ignore[arg-type]

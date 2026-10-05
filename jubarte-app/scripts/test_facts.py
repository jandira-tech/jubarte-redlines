# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
"""facts.py keeps data/facts.jsonl an append-only, uuidv7-ordered log.

Run with:
    uv run --python 3.14 --with pytest pytest scripts/test_facts.py -q
"""

import json
import uuid
from pathlib import Path

import pytest
from facts import FACTS, append, current, listing_gaps, main, problems, read, stamp


def test_records_carry_a_uuidv7_and_its_own_time(tmp_path: Path) -> None:
    log = tmp_path / "facts.jsonl"
    fresh = append({"engine.version": "0.10.1", "app.free_uses": 5}, "test", log)
    assert [r["key"] for r in fresh] == ["engine.version", "app.free_uses"]
    for rec in read(log):
        u = uuid.UUID(rec["id"])
        assert u.version == 7
        assert rec["ts"] == stamp(u)
        assert rec["ts"].endswith("Z") and len(rec["ts"]) == len("2026-10-02T12:00:00.000Z")
    assert problems(read(log)) == []
    # One compact JSON object a line, fields in a fixed order.
    first = log.read_text().splitlines()[0]
    assert first.startswith('{"id":"') and list(json.loads(first)) == ["id", "ts", "key", "value", "source"]


def test_an_unchanged_value_adds_nothing(tmp_path: Path) -> None:
    log = tmp_path / "facts.jsonl"
    append({"engine.version": "0.10.1"}, "test", log)
    assert append({"engine.version": "0.10.1"}, "test", log) == []
    assert len(read(log)) == 1


def test_the_latest_record_wins_and_null_retires_a_key(tmp_path: Path) -> None:
    log = tmp_path / "facts.jsonl"
    append({"engine.version": "0.10.1", "site.limit_per_minute": 60}, "test", log)
    append({"engine.version": "0.10.2"}, "release", log)
    assert current(read(log)) == {"engine.version": "0.10.2", "site.limit_per_minute": 60}
    append({"site.limit_per_minute": None}, "gone", log)
    assert current(read(log)) == {"engine.version": "0.10.2"}
    # Retiring a key that is not there writes nothing.
    assert append({"nothing.here": None}, "test", log) == []
    assert len(read(log)) == 4


def test_check_finds_every_kind_of_damage(tmp_path: Path) -> None:
    log = tmp_path / "facts.jsonl"
    append({"a.one": 1, "a.two": 2}, "test", log)
    first, second = read(log)

    late = dict(first, ts="2020-01-01T00:00:00.000Z")
    assert "is not the id's time" in problems([late])[0]
    assert "is not after the line before it" in problems([second, first])[0]
    v4 = str(uuid.uuid4())
    assert "is not a lowercase uuidv7" in problems([dict(first, id=v4)])[0]
    assert "dotted lowercase" in problems([dict(first, key="Engine")])[0]
    assert "has no source" in problems([dict(first, source=" ")])[0]
    assert "fields" in problems([{**first, "extra": 1}])[0]


def test_append_refuses_a_damaged_log_and_a_bad_key(tmp_path: Path) -> None:
    log = tmp_path / "facts.jsonl"
    append({"a.one": 1}, "test", log)
    rec = read(log)[0]
    log.write_text(json.dumps(dict(rec, ts="2020-01-01T00:00:00.000Z")) + "\n")
    with pytest.raises(ValueError, match="unsound"):
        append({"a.two": 2}, "test", tmp_path / "facts.jsonl")
    with pytest.raises(ValueError, match="dotted lowercase"):
        append({"nodot": 1}, "test", tmp_path / "other.jsonl")
    with pytest.raises(ValueError, match="source"):
        append({"a.one": 1}, "", tmp_path / "other.jsonl")


def test_the_cli_sets_gets_and_checks(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    log = str(tmp_path / "facts.jsonl")
    assert main(["--file", log, "set", "app_store.price", '"$99.99"', "--source", "listing"]) == 0
    assert main(["--file", log, "get", "app_store.price"]) == 0
    assert capsys.readouterr().out.strip().endswith('"$99.99"')
    assert main(["--file", log, "get", "app_store.nothing"]) == 1
    assert main(["--file", log, "check"]) == 0


def test_the_repository_log_is_sound() -> None:
    records = read(FACTS)
    assert records, "data/facts.jsonl is empty"
    assert problems(records) == []


def test_a_submission_waits_for_the_listing_review(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    # Every app update: the App Store text and screenshots are read against
    # the version before it goes to Apple, and the review is a fact.
    log = tmp_path / "facts.jsonl"
    checklist = ["Screenshots show this version's window"]
    append({"app_store.listing.checklist": checklist,
            "app_store.listing.description": "old text",
            "app_store.listing.draft.description": "new text"}, "test", log)
    assert listing_gaps(current(read(log)), "0.11.2") == [
        "app_store.listing.reviewed_for is not 0.11.2",
        "app_store.listing.draft.description differs from app_store.listing.description",
    ]
    assert main(["--file", str(log), "listing", "0.11.2"]) == 1
    err = capsys.readouterr().err
    assert "Screenshots show this version's window" in err and "reviewed_for" in err
    append({"app_store.listing.reviewed_for": "0.11.2",
            "app_store.listing.description": "new text"}, "test", log)
    assert listing_gaps(current(read(log)), "0.11.2") == []
    assert main(["--file", str(log), "listing", "0.11.2"]) == 0


def test_the_repository_listing_names_the_submitted_version() -> None:
    now = current(read(FACTS))
    assert now["app.release.version"] == now.get("app_store.listing.version")
    assert now["app_store.listing.checklist"]

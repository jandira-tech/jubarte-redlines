# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Integration checks for the native filesystem publication boundary."""

import warnings

import pytest
from jubarte_redlines import _native

pytestmark = pytest.mark.integration


def test_publish_returns_none_and_installs_bytes(tmp_path):
    root = _native._OutputRoot(str(tmp_path))
    with warnings.catch_warnings(record=True) as caught:
        warnings.simplefilter("always")
        assert root.publish([("a.txt", b"Alice")], False) is None
        assert root.publish([("a.txt", b"Bob")], True) is None
    assert caught == []
    assert root.read("a.txt") == b"Bob"
    assert sorted(p.name for p in tmp_path.iterdir()) == ["a.txt"]


def test_publish_raises_oserror_and_preserves_existing_output(tmp_path):
    root = _native._OutputRoot(str(tmp_path))
    root.publish([("a.txt", b"Alice")], False)
    with pytest.raises(OSError, match="committed outputs"):
        root.publish([("a.txt", b"Bob")], False)
    assert root.read("a.txt") == b"Alice"
    assert sorted(p.name for p in tmp_path.iterdir()) == ["a.txt"]

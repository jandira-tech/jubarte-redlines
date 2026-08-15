# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
"""Train-matching tests for asc-new-version.py build selection.

The App Store Connect Build resource's `attributes.version` is the
CFBundleVersion (build number). The marketing train
(CFBundleShortVersionString) lives on the included preReleaseVersion.
Selecting the first VALID build attaches the wrong train.

Run with:
    uv run --with pytest,cryptography pytest scripts/test_asc_new_version.py -q
"""

from importlib.machinery import SourceFileLoader
from pathlib import Path

MOD_PATH = Path(__file__).resolve().parent / "asc-new-version.py"


def load_script():
    return SourceFileLoader("asc_new_version_under_test", str(MOD_PATH)).load_module()


def build(build_id, build_number, train_id, state="VALID"):
    return {
        "id": build_id,
        "type": "builds",
        "attributes": {"version": build_number, "processingState": state},
        "relationships": {
            "preReleaseVersion": {
                "data": {"type": "preReleaseVersions", "id": train_id}
            }
        },
    }


def train(train_id, version):
    return {
        "id": train_id,
        "type": "preReleaseVersions",
        "attributes": {"version": version, "platform": "MAC_OS"},
    }


def payload(*builds, included):
    return {"data": list(builds), "included": list(included)}


def test_selects_matching_train_not_newest_valid():
    """Newest VALID build is 0.6.2; requesting 0.7.0 must not attach it."""
    mod = load_script()
    body = payload(
        build("BUILD-NEWEST", "99", "TRAIN-662"),
        build("BUILD-070", "12", "TRAIN-070"),
        included=[train("TRAIN-662", "0.6.2"), train("TRAIN-070", "0.7.0")],
    )

    chosen = mod.select_build_for_version(body, "0.7.0")

    assert chosen is not None
    assert chosen[0] == "BUILD-070"


def test_rejects_when_no_valid_build_on_requested_train():
    mod = load_script()
    body = payload(
        build("BUILD-662", "8", "TRAIN-662"),
        included=[train("TRAIN-662", "0.6.2")],
    )

    assert mod.select_build_for_version(body, "0.7.0") is None


def test_skips_non_valid_build_on_the_requested_train():
    mod = load_script()
    body = payload(
        build("BUILD-PROC", "13", "TRAIN-070", state="PROCESSING"),
        build("BUILD-OLD", "11", "TRAIN-070"),
        included=[train("TRAIN-070", "0.7.0")],
    )

    chosen = mod.select_build_for_version(body, "0.7.0")

    assert chosen is not None
    assert chosen[0] == "BUILD-OLD"


def test_skips_train_match_with_missing_build_number():
    """VALID + matching train is not enough if CFBundleVersion is absent."""
    mod = load_script()
    body = payload(
        build("BUILD-NAKED", None, "TRAIN-070"),
        build("BUILD-EMPTY", "", "TRAIN-070"),
        build("BUILD-OK", "14", "TRAIN-070"),
        included=[train("TRAIN-070", "0.7.0")],
    )

    chosen = mod.select_build_for_version(body, "0.7.0")

    assert chosen is not None
    assert chosen[0] == "BUILD-OK"
    assert chosen[1] == "14"

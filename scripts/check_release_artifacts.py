#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Refuse to publish a wheel set that lacks an advertised platform.

`python3 scripts/check_release_artifacts.py dist/pypi --version 0.11.0`
exits 1 and names every missing tag; scripts/release.sh runs it on the CI
wheels it downloaded, before `uv publish`. The required set is the README's
"Supported environments" table ("Python release wheels"); change both
together, and the `wheels` matrix in .github/workflows/release.yml with them.
"""

from __future__ import annotations

import argparse
import pathlib
import re
import sys

REQUIRED_WHEEL_TAGS: tuple[str, ...] = (
    "macosx_10_12_x86_64",
    "macosx_11_0_arm64",
    "manylinux_2_28_x86_64",
    "manylinux_2_28_aarch64",
    "musllinux_1_2_x86_64",
    "musllinux_1_2_aarch64",
    "win_amd64",
)

# abi3-py310 (jubarte-python/Cargo.toml): one wheel per platform, tagged
# cp310-abi3, covers every CPython >= 3.10.
_WHEEL = re.compile(r"^jubarte_redlines-(?P<version>[^-]+)-cp310-abi3-(?P<tag>.+)\.whl$")


def missing_platforms(filenames: list[str], *, version: str) -> set[str]:
    """Required tags with no wheel of exactly `version`.

    A manylinux_2_34 wheel does not satisfy the 2_28 requirement: pip on a
    glibc 2.28 host (RHEL 8, Debian 10, Ubuntu 20.04) refuses it. Wheels of
    another version, the sdist and checksum files are ignored.
    """
    present: set[str] = set()
    for name in filenames:
        m = _WHEEL.match(pathlib.PurePosixPath(name).name)
        if m and m.group("version") == version:
            present.add(m.group("tag"))
    return {tag for tag in REQUIRED_WHEEL_TAGS if tag not in present}


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("dist", type=pathlib.Path, help="directory holding the wheels to publish")
    parser.add_argument("--version", required=True, help="the version being released, e.g. 0.11.0")
    args = parser.parse_args(argv)
    names = [p.name for p in args.dist.iterdir()]
    missing = missing_platforms(names, version=args.version)
    if missing:
        print("missing wheels for " + args.version + ": " + ", ".join(sorted(missing)), file=sys.stderr)
        return 1
    print(f"all {len(REQUIRED_WHEEL_TAGS)} advertised wheel platforms present for {args.version}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

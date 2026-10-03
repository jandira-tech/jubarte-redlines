#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""The release gate refuses a wheel set that misses an advertised platform.

`scripts/check_release_artifacts.py` is the last step before `uv publish`
in scripts/release.sh; its required set is the README's platform table.
"""

from __future__ import annotations

import contextlib
import io
import tempfile
import unittest
from pathlib import Path

from check_release_artifacts import REQUIRED_WHEEL_TAGS, main, missing_platforms

# The four wheels PyPI holds for 0.10.1, plus the sdist, as listed on
# https://pypi.org/project/jubarte-redlines/0.10.1/#files on 2026-10-02.
PUBLISHED_0_10_1 = [
    "jubarte_redlines-0.10.1-cp310-abi3-macosx_10_12_x86_64.whl",
    "jubarte_redlines-0.10.1-cp310-abi3-macosx_11_0_arm64.whl",
    "jubarte_redlines-0.10.1-cp310-abi3-manylinux_2_34_aarch64.whl",
    "jubarte_redlines-0.10.1-cp310-abi3-manylinux_2_34_x86_64.whl",
    "jubarte_redlines-0.10.1.tar.gz",
]


def complete_set(version: str) -> list[str]:
    return [f"jubarte_redlines-{version}-cp310-abi3-{tag}.whl" for tag in REQUIRED_WHEEL_TAGS]


class MissingPlatforms(unittest.TestCase):
    def test_0_10_1_set_misses_windows_musl_and_the_2_28_floor(self) -> None:
        missing = missing_platforms(PUBLISHED_0_10_1, version="0.10.1")
        self.assertEqual(
            missing,
            {
                "win_amd64",
                "manylinux_2_28_x86_64",
                "manylinux_2_28_aarch64",
                "musllinux_1_2_x86_64",
                "musllinux_1_2_aarch64",
            },
        )

    def test_a_complete_set_passes_and_a_foreign_version_is_ignored(self) -> None:
        names = complete_set("0.11.0")
        names += ["jubarte_redlines-0.11.0.tar.gz", "jubarte_redlines-0.10.1-cp310-abi3-win_amd64.whl"]
        self.assertEqual(missing_platforms(names, version="0.11.0"), set())

    def test_a_2_34_wheel_does_not_satisfy_the_2_28_floor(self) -> None:
        names = complete_set("0.11.0")
        names.remove("jubarte_redlines-0.11.0-cp310-abi3-manylinux_2_28_x86_64.whl")
        names.append("jubarte_redlines-0.11.0-cp310-abi3-manylinux_2_34_x86_64.whl")
        self.assertEqual(missing_platforms(names, version="0.11.0"), {"manylinux_2_28_x86_64"})

    def test_a_wheel_of_another_version_does_not_count(self) -> None:
        names = complete_set("0.11.0")
        names.remove("jubarte_redlines-0.11.0-cp310-abi3-win_amd64.whl")
        names.append("jubarte_redlines-0.11.0rc1-cp310-abi3-win_amd64.whl")
        self.assertEqual(missing_platforms(names, version="0.11.0"), {"win_amd64"})

    def test_paths_are_reduced_to_their_file_name(self) -> None:
        names = [f"dist/pypi/{name}" for name in complete_set("0.11.0")]
        self.assertEqual(missing_platforms(names, version="0.11.0"), set())

    def test_every_required_tag_is_a_pypi_tag(self) -> None:
        # The gate must never ask for a tag PyPI refuses (plain `linux_x86_64`).
        for tag in REQUIRED_WHEEL_TAGS:
            self.assertRegex(tag, r"^(macosx_\d+_\d+|manylinux_\d+_\d+|musllinux_\d+_\d+)_[a-z0-9_]+$|^win_amd64$")


class CommandLine(unittest.TestCase):
    """`check_release_artifacts.py DIST --version VER` exits 1 and names every
    missing tag, so release.sh can die on it."""

    def setUp(self) -> None:
        self.dist = Path(tempfile.mkdtemp(prefix="check_release_artifacts_"))

    def run_main(self, *argv: str) -> tuple[int, str, str]:
        out, err = io.StringIO(), io.StringIO()
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            code = main(list(argv))
        return code, out.getvalue(), err.getvalue()

    def test_an_incomplete_dist_exits_1_and_names_the_missing_tags(self) -> None:
        for name in PUBLISHED_0_10_1:
            (self.dist / name).write_bytes(b"")
        code, out, err = self.run_main(str(self.dist), "--version", "0.10.1")
        self.assertEqual(code, 1)
        self.assertEqual(out, "")
        self.assertIn("missing wheels for 0.10.1:", err)
        for tag in ("manylinux_2_28_aarch64", "manylinux_2_28_x86_64", "musllinux_1_2_aarch64", "musllinux_1_2_x86_64", "win_amd64"):
            self.assertIn(tag, err)
        self.assertNotIn("macosx", err)

    def test_a_complete_dist_exits_0(self) -> None:
        for name in complete_set("0.11.0") + ["jubarte_redlines-0.11.0.tar.gz", "SHA256SUMS.txt"]:
            (self.dist / name).write_bytes(b"")
        code, out, err = self.run_main(str(self.dist), "--version", "0.11.0")
        self.assertEqual(code, 0, err)
        self.assertIn(f"all {len(REQUIRED_WHEEL_TAGS)} advertised wheel platforms present for 0.11.0", out)
        self.assertEqual(err, "")

    def test_an_empty_dist_misses_everything(self) -> None:
        code, _out, err = self.run_main(str(self.dist), "--version", "0.11.0")
        self.assertEqual(code, 1)
        self.assertEqual(err.count(", ") + 1, len(REQUIRED_WHEEL_TAGS))


if __name__ == "__main__":
    unittest.main()

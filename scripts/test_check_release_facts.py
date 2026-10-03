#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""The release refuses to finish while jubarte-app's facts name another version.

`scripts/check_release_facts.py` runs after the downstream step of
scripts/release.sh: jubarte.pro and the Mac app print the engine's version,
files and date from jubarte-app/data/facts.jsonl, so a release that left the
log on the previous version would ship pages and an About window that lie.
"""

from __future__ import annotations

import contextlib
import io
import json
import shutil
import subprocess
import tempfile
import time
import unittest
import uuid
from pathlib import Path

from check_release_artifacts import REQUIRED_WHEEL_TAGS
from check_release_facts import main, problems

VER = "0.10.2"
DATE = "2026-10-09"
CHANGELOG = f"""# Changelog

## [{VER}] - {DATE}

> **Summary.** Faster.

## [0.10.1] - 2026-09-30
"""

_last_ms = 0
_seq = 0


def uuid7() -> str:
    """A uuidv7 for any Python 3 (uuid.uuid7 is 3.14+), increasing per call."""
    global _last_ms, _seq
    ms = max(int(time.time() * 1000), _last_ms)
    _seq = _seq + 1 if ms == _last_ms else 0
    _last_ms = ms
    value = (ms << 80) | (0x7 << 76) | (_seq << 64) | (0b10 << 62) | 0x1234
    return str(uuid.UUID(int=value))


def release_facts(ver: str = VER, date: str = DATE) -> dict[str, object]:
    wheels = [
        {"target": tag, "file": f"jubarte_redlines-{ver}-cp310-abi3-{tag}.whl", "size": 1}
        for tag in REQUIRED_WHEEL_TAGS
    ]
    wheels.append({"target": "source", "file": f"jubarte_redlines-{ver}.tar.gz", "size": 1})
    return {
        "engine.version": ver,
        "engine.released": date,
        "release.archives": [
            {"target": "Linux x86_64", "file": f"jubarte-{ver}-linux-x86_64.tar.gz", "size": 1}
        ],
        "release.wheels": wheels,
        "release.history": [
            {"v": ver, "d": date, "t": "Faster."},
            {"v": "0.10.1", "d": "2026-09-30", "t": "Older."},
        ],
    }


class ReleaseFacts(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp(prefix="release_facts_"))
        self.app = self.tmp / "jubarte-app"
        (self.app / "data").mkdir(parents=True)
        self.log = self.app / "data" / "facts.jsonl"
        self.changelog = self.tmp / "CHANGELOG.md"
        self.changelog.write_text(CHANGELOG)

    def tearDown(self) -> None:
        shutil.rmtree(self.tmp, ignore_errors=True)

    def append(self, values: dict[str, object]) -> None:
        with self.log.open("a") as f:
            for key, value in values.items():
                rec = {"id": uuid7(), "ts": "", "key": key, "value": value, "source": "test"}
                f.write(json.dumps(rec) + "\n")

    def found(self, ver: str = VER) -> list[str]:
        return problems(ver, self.app, self.changelog)

    def test_a_log_on_the_release_passes(self) -> None:
        self.append(release_facts("0.10.1", "2026-09-30"))
        self.append(release_facts())
        self.assertEqual(self.found(), [])

    def test_a_log_left_on_the_previous_release_fails(self) -> None:
        self.append(release_facts("0.10.1", "2026-09-30"))
        found = "\n".join(self.found())
        self.assertIn("engine.version is 0.10.1, not 0.10.2", found)
        self.assertIn("release.history starts with 0.10.1", found)
        self.assertIn("jubarte-0.10.1-linux-x86_64.tar.gz", found)

    def test_the_date_must_be_the_changelogs(self) -> None:
        self.append(release_facts(date="2026-10-10"))
        self.assertIn(
            f"engine.released is 2026-10-10; CHANGELOG.md dates {VER} {DATE}", self.found()
        )

    def test_every_required_wheel_must_be_listed(self) -> None:
        facts = release_facts()
        facts["release.wheels"] = [
            w for w in facts["release.wheels"] if "musllinux" not in w["file"]
        ]
        self.append(facts)
        missing = [p for p in self.found() if "lists no wheel for" in p]
        self.assertEqual(len(missing), 2, missing)

    def test_no_archives_is_a_problem(self) -> None:
        facts = release_facts()
        facts["release.archives"] = []
        self.append(facts)
        self.assertIn("release.archives lists no CLI archive", self.found())

    def test_the_latest_record_wins_and_null_retires(self) -> None:
        self.append(release_facts())
        self.append({"engine.version": None})
        self.assertIn("data/facts.jsonl has no engine.version", self.found())

    def test_ids_must_be_uuidv7(self) -> None:
        self.append(release_facts())
        with self.log.open("a") as f:
            f.write(json.dumps({"id": str(uuid.uuid4()), "ts": "", "key": "x.y",
                                "value": 1, "source": "t"}) + "\n")
        self.assertTrue(any("is not a uuidv7" in p for p in self.found()))

    def test_an_uncommitted_log_fails(self) -> None:
        git = ["git", "-C", str(self.app)]
        subprocess.run([*git, "init", "-q"], check=True)
        self.append(release_facts())
        subprocess.run([*git, "add", "."], check=True)
        subprocess.run(
            [*git, "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-qm", "facts"],
            check=True,
        )
        self.assertEqual(self.found(), [])
        self.append({"site.url": "https://jubarte.pro"})
        self.assertTrue(any("uncommitted" in p for p in self.found()), self.found())

    def test_the_cli_exits_1_with_each_problem_and_2_without_a_log(self) -> None:
        self.append(release_facts("0.10.1", "2026-09-30"))
        args = [VER, "--app", str(self.app), "--changelog", str(self.changelog)]
        err = io.StringIO()
        with contextlib.redirect_stderr(err):
            self.assertEqual(main(args), 1)
        self.assertIn("engine.version is 0.10.1", err.getvalue())
        self.log.unlink()
        with contextlib.redirect_stderr(io.StringIO()):
            self.assertEqual(main(args), 2)
        with contextlib.redirect_stderr(io.StringIO()):
            self.assertEqual(main(["1.0", "--app", str(self.app)]), 2)


if __name__ == "__main__":
    unittest.main()

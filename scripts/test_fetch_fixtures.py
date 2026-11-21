#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: Apache-2.0

"""scripts/fetch_fixtures.py downloads the test documents kept outside git.

Each case serves a small dataset from a local HTTP server, so nothing here
reaches the network. The last case checks the tracked manifest itself.
"""

from __future__ import annotations

import contextlib
import hashlib
import http.server
import io
import shutil
import sys
import tempfile
import threading
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import fetch_fixtures

REVISION = "0123456789abcdef0123456789abcdef01234567"
FILES = {"a/one.docx": b"first document", "a/b c/two.docx": b"second document"}


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


class FetchFixtures(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp(prefix="fetch_fixtures_"))
        served = self.tmp / "served" / "datasets" / "owner" / "name" / "resolve" / REVISION
        for name, data in FILES.items():
            (served / name).parent.mkdir(parents=True, exist_ok=True)
            (served / name).write_bytes(data)
        self.served = served
        root = str(self.tmp / "served")

        class Quiet(http.server.SimpleHTTPRequestHandler):
            def __init__(self, *args, **kwargs):
                super().__init__(*args, directory=root, **kwargs)

            def log_message(self, *args) -> None:
                pass

        self.server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Quiet)
        threading.Thread(target=self.server.serve_forever, daemon=True).start()
        self.host = f"http://127.0.0.1:{self.server.server_address[1]}"
        self.dest = self.tmp / "dest"
        self.manifest = self.tmp / "manifest.sha256"
        self.write_manifest({name: digest(data) for name, data in FILES.items()})
        self.saved_attempts = fetch_fixtures.ATTEMPTS
        fetch_fixtures.ATTEMPTS = 1

    def tearDown(self) -> None:
        fetch_fixtures.ATTEMPTS = self.saved_attempts
        self.server.shutdown()
        self.server.server_close()
        shutil.rmtree(self.tmp, ignore_errors=True)

    def write_manifest(self, entries: dict[str, str], head: str | None = None) -> None:
        head = head if head is not None else f"# dataset: owner/name\n# revision: {REVISION}\n"
        self.manifest.write_text(head + "".join(f"{d}  {n}\n" for n, d in entries.items()))

    def run_script(self, *extra: str) -> tuple[int, str]:
        out = io.StringIO()
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(out):
            code = fetch_fixtures.main(
                ["--manifest", str(self.manifest), "--dest", str(self.dest), "--host", self.host, *extra]
            )
        return code, out.getvalue()

    def test_check_fails_before_a_fetch_and_passes_after(self) -> None:
        code, text = self.run_script("--check")
        self.assertEqual(code, 1, text)
        self.assertIn("python3 scripts/fetch_fixtures.py", text)
        code, text = self.run_script()
        self.assertEqual(code, 0, text)
        for name, data in FILES.items():
            self.assertEqual((self.dest / name).read_bytes(), data)
        code, text = self.run_script("--check")
        self.assertEqual(code, 0, text)

    def test_a_second_run_downloads_nothing(self) -> None:
        self.run_script()
        shutil.rmtree(self.served)
        code, text = self.run_script()
        self.assertEqual(code, 0, text)
        self.assertIn("(0 downloaded)", text)

    def test_a_download_with_another_hash_is_refused_and_not_kept(self) -> None:
        self.write_manifest({"a/one.docx": digest(b"something else")})
        code, text = self.run_script()
        self.assertEqual(code, 1, text)
        self.assertIn("the manifest says", text)
        self.assertFalse((self.dest / "a/one.docx").exists())
        self.assertEqual(list((self.dest / "a").glob(".fetch-*")), [])

    def test_a_changed_file_on_disk_is_replaced(self) -> None:
        self.run_script()
        (self.dest / "a/one.docx").write_bytes(b"edited")
        code, text = self.run_script("--check")
        self.assertEqual(code, 1, text)
        self.assertIn("differs", text)
        code, text = self.run_script()
        self.assertEqual(code, 0, text)
        self.assertEqual((self.dest / "a/one.docx").read_bytes(), FILES["a/one.docx"])

    def test_a_missing_remote_file_fails(self) -> None:
        self.write_manifest({"a/absent.docx": digest(b"x")})
        code, text = self.run_script()
        self.assertEqual(code, 1, text)
        self.assertIn("404", text)
        # the temporary file is closed and removed, also when the request fails
        self.assertEqual(list((self.dest / "a").glob(".fetch-*")), [])

    def test_a_path_that_leaves_the_folder_is_refused(self) -> None:
        for name in ("../escape.docx", "/abs.docx", "a\\b.docx", "C:drive.docx"):
            with self.subTest(name=name):
                self.write_manifest({name: digest(b"x")})
                code, text = self.run_script()
                self.assertEqual(code, 2, text)
                self.assertIn("unsafe path", text)

    def test_a_branch_name_is_not_a_revision(self) -> None:
        self.write_manifest({"a/one.docx": digest(b"x")}, head="# dataset: owner/name\n# revision: main\n")
        code, text = self.run_script()
        self.assertEqual(code, 2, text)
        self.assertIn("full commit hash", text)

    def test_the_tracked_manifest_is_sound(self) -> None:
        dataset, revision, files = fetch_fixtures.read_manifest(fetch_fixtures.MANIFEST)
        self.assertRegex(dataset, r"^[\w.-]+/[\w.-]+$")
        self.assertEqual(len(revision), 40)
        names = [name for _, name in files]
        self.assertEqual(len(names), len(set(names)))
        self.assertEqual(names, sorted(names))
        self.assertTrue(all(name.endswith(".docx") for name in names))


if __name__ == "__main__":
    unittest.main()

#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

# SPDX-License-Identifier: AGPL-3.0-only

"""scripts/identity.py: two jubarte binaries write the same bytes on the
release samples (#1752; checklist before#6, after#8).

The binaries are stubs that derive their output from their inputs, so a
test controls exactly which rows differ.
"""

from __future__ import annotations

import csv
import hashlib
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
SCRIPT = HERE / "identity.py"

# stub jubarte: --version, `convert SRC -o DEST …` and `BASE NEXT -o DEST …`;
# writes its inputs (and $SALT on rows whose inputs contain $SALT_ON) to DEST;
# no output at all on rows whose inputs contain $FAIL_ON.
STUB = r"""#!/bin/bash
[ "$1" = --version ] && { echo "jubarte 9.9.9 (${NAME})"; exit 0; }
echo "$*" >> "$ARGV_LOG"
dest=""; ins=""
while [ $# -gt 0 ]; do
  case "$1" in
    -o) dest=$2; shift 2 ;;
    -*) shift ;;
    convert) shift ;;
    *) ins="$ins $1"; shift ;;
  esac
done
sleep 0.05
case "$ins" in *"${FAIL_ON:-@none@}"*) exit 1 ;; esac
salt=""
case "$ins" in *"${SALT_ON:-@none@}"*) salt=${SALT:-} ;; esac
printf '%s%s\n' "$ins" "$salt" > "$dest"
"""


class Identity(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp(prefix="identity-")).resolve()
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)
        self.root = self.tmp / "bench"
        self.root.mkdir()
        self.out = self.tmp / "out"
        self.argv = self.tmp / "argv.log"

    def stub(self, name: str, **env: str) -> Path:
        prefix = "".join(f"export {k}='{v}'\n" for k, v in {"NAME": name, **env}.items())
        path = self.tmp / f"jubarte-{name}"
        path.write_text(STUB.replace("#!/bin/bash\n", f"#!/bin/bash\nexport ARGV_LOG='{self.argv}'\n{prefix}", 1))
        path.chmod(0o755)
        return path

    def redline_csv(self, n: int = 2) -> Path:
        path = self.root / "sample_redline_9.9.9_x.csv"
        with path.open("w", newline="") as f:
            w = csv.writer(f)
            w.writerow(["key", "base", "base_sha256", "next", "next_sha256", "id"])
            for i in range(n):
                w.writerow([f"pair{i}", f"corpus/base{i}.docx", "0" * 64, f"corpus/next{i}.docx", "0" * 64, ""])
                self.touch(f"corpus/base{i}.docx", f"corpus/next{i}.docx")
        return path

    def conversion_csv(self, stems: list[str]) -> Path:
        path = self.root / "sample_conversion_9.9.9_x.csv"
        with path.open("w", newline="") as f:
            w = csv.writer(f)
            w.writerow(["state", "stem", "docx", "docx_sha256"])
            for stem in stems:
                w.writerow(["clean", stem, f"corpus/{stem}.docx", "0" * 64])
                self.touch(f"corpus/{stem}.docx")
        return path

    def touch(self, *paths: str) -> None:
        for rel in paths:
            (self.root / rel).parent.mkdir(parents=True, exist_ok=True)
            (self.root / rel).write_text(rel)

    def run_identity(self, a: Path, b: Path, *csvs: Path, jobs: int = 4) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, str(SCRIPT), str(a), str(b), str(self.out), *map(str, csvs),
             "--root", str(self.root), "--jobs", str(jobs)],
            capture_output=True, text=True, timeout=120,
        )

    def test_two_binaries_that_agree_pass_and_are_named(self) -> None:
        a, b = self.stub("a"), self.stub("b")
        r = self.run_identity(a, b, self.redline_csv(), self.conversion_csv(["clean__x", "clean__y"]))
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        for binary in (a, b):
            self.assertIn(str(binary), r.stdout)
            self.assertIn(hashlib.sha256(binary.read_bytes()).hexdigest(), r.stdout)
        self.assertIn("jubarte 9.9.9 (a)", r.stdout)
        self.assertIn("jubarte 9.9.9 (b)", r.stdout)
        self.assertIn("4 jobs: 4 identical", r.stdout)
        self.assertEqual(sorted(p.name for p in self.out.iterdir()), ["identity.log"])
        self.assertIn("4 jobs: 4 identical", (self.out / "identity.log").read_text())

    def test_the_bench_flags_are_passed(self) -> None:
        r = self.run_identity(self.stub("a"), self.stub("b"), self.redline_csv(1), self.conversion_csv(["clean__x"]))
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        calls = self.argv.read_text()
        self.assertIn(f"{self.root / 'corpus/base0.docx'} {self.root / 'corpus/next0.docx'} -o ", calls)
        self.assertIn("--force --quiet -d 1970-01-01T00:00:00Z", calls)
        self.assertIn(f"convert {self.root / 'corpus/clean__x.docx'} -o ", calls)
        self.assertIn("--force --revisions word --compress", calls)

    def test_a_row_that_differs_is_named_and_kept(self) -> None:
        a, b = self.stub("a"), self.stub("b", SALT_ON="clean__y", SALT="!")
        r = self.run_identity(a, b, self.conversion_csv(["clean__x", "clean__y", "clean__z"]))
        self.assertEqual(r.returncode, 1, r.stdout)
        diffs = [line for line in r.stdout.splitlines() if line.startswith("DIFF")]
        self.assertEqual(len(diffs), 1, r.stdout)
        self.assertIn("clean__y", diffs[0])
        kept = sorted(p.name for p in self.out.iterdir() if p.name != "identity.log")
        self.assertEqual(len(kept), 2, kept)

    def test_long_stems_sharing_a_prefix_never_collide(self) -> None:
        # 0.13.0: outputs named stem[:60] under six threads made 25 false DIFFs
        prefix = "tracking_without_comments__8e534b49f0_00a9b37342f07e6b9bd5b8fffbe4dd777d31053171e69da2"
        stems = [f"{prefix}__vs__{i:02d}" for i in range(12)]
        r = self.run_identity(self.stub("a"), self.stub("b"), self.conversion_csv(stems), jobs=6)
        self.assertEqual(r.returncode, 0, r.stdout)
        self.assertIn("12 jobs: 12 identical", r.stdout)

    def test_output_from_one_binary_only_is_a_difference(self) -> None:
        r = self.run_identity(self.stub("a"), self.stub("b", FAIL_ON="clean__y"),
                              self.conversion_csv(["clean__x", "clean__y"]))
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("MISSING", r.stdout)

    def test_no_output_from_either_is_counted_not_hidden(self) -> None:
        r = self.run_identity(self.stub("a", FAIL_ON="clean__y"), self.stub("b", FAIL_ON="clean__y"),
                              self.conversion_csv(["clean__x", "clean__y"]))
        self.assertEqual(r.returncode, 0, r.stdout)
        self.assertIn("1 of them no output from either", r.stdout)

    def test_a_run_where_no_job_wrote_anything_proves_nothing(self) -> None:
        # a flag both binaries reject fails every row the same way
        r = self.run_identity(self.stub("a", FAIL_ON="clean__"), self.stub("b", FAIL_ON="clean__"),
                              self.conversion_csv(["clean__x", "clean__y"]))
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("no job wrote an output", r.stdout)

    def test_an_input_the_bench_root_lacks_is_refused(self) -> None:
        # a wrong --root fails every row on both sides: identical, and vacuous
        csv_path = self.conversion_csv(["clean__x", "clean__y"])
        (self.root / "corpus/clean__y.docx").unlink()
        r = self.run_identity(self.stub("a"), self.stub("b"), csv_path)
        self.assertEqual(r.returncode, 2, r.stdout)
        self.assertIn("corpus/clean__y.docx", r.stderr)
        self.assertFalse(self.argv.exists(), "no binary runs before every input is found")

    def test_release_sh_runs_these_tests_and_the_checklist_names_the_tool(self) -> None:
        text = (HERE / "release.sh").read_text()
        self.assertTrue('"${py312[@]}" scripts/test_identity.py' in text, "step 5 runs scripts/test_identity.py")
        items = text[text.index("checklist_items() {"):]
        items = items[:items.index("\n}\n")]
        self.assertGreaterEqual(items.count("scripts/identity.py"), 2, "before#6 and after#8")

    def test_a_csv_of_neither_sample_kind_is_refused(self) -> None:
        path = self.root / "other.csv"
        path.write_text("a,b\n1,2\n")
        r = self.run_identity(self.stub("a"), self.stub("b"), path)
        self.assertEqual(r.returncode, 2, r.stdout)
        self.assertIn("other.csv", r.stderr)


if __name__ == "__main__":
    unittest.main()

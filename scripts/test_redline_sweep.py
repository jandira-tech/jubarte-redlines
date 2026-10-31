#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Integration tests for the sweep and source staging, with fake external tools.

Execute the production filter alone: no Rust binary, Word or .NET required.
"""

import hashlib
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent


class BaselineKeys(unittest.TestCase):
    def keys(self, contents):
        # Exercise the actual pipeline, so removing the comment filter causes
        # a regression without running the expensive document generation stage.
        lines = (HERE / "redline-sweep.sh").read_text().splitlines()
        commands = [line.strip() for line in lines
                    if line.lstrip().startswith("awk ") and line.endswith('>"$OUT/.baseline_keys"')]
        self.assertEqual(len(commands), 1, "locate the production baseline filter")
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            baseline = root / "baseline with spaces.tsv"
            baseline.write_text(contents, encoding="utf-8")
            result = subprocess.run(
                ["bash", "-o", "pipefail", "-c", commands[0]],
                env=dict(os.environ, BASELINE=str(baseline), OUT=str(root), LC_ALL="C"),
                capture_output=True, text=True, timeout=5,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            return (root / ".baseline_keys").read_text().splitlines()

    def test_tabbed_comments_do_not_become_fixed_findings(self):
        self.assertEqual(self.keys(
            "# pair_stem\terror_id\tdescription\n"
            "#retired\tOldError\tno longer a baseline finding\n"
            "active\tKnownError\tdescription\n"
            "# trailing comment\tmetadata\n"
        ), ["active\tKnownError"])

    def test_real_keys_remain_sorted_unique_and_keep_embedded_hashes(self):
        self.assertEqual(self.keys(
            "zeta\tErrorB\told description\n"
            "#ignored\tCommentError\n"
            "alpha#revision\tErrorA\tdescription\n"
            "zeta\tErrorB\tnew description\n"
            "alpha#revision\tErrorC\n"
            "\n"
            "single-field\n"
        ), ["alpha#revision\tErrorA", "alpha#revision\tErrorC", "zeta\tErrorB"])

    def test_empty_or_comment_only_baseline_has_no_keys(self):
        for contents in ("", "# provenance\n# pair\terror\n\n"):
            with self.subTest(contents=contents):
                self.assertEqual(self.keys(contents), [])



# a stub jubarte: --version, or `BASE NEXT -d … -o DEST --force --quiet`; the
# redline names its base, so a stub Word can tell the pairs apart
STUB = """#!/bin/sh
[ "$1" = --version ] && { echo "jubarte 9.9.9"; exit 0; }
echo "$*" >> "$ARGV_LOG"
base=$1
while [ $# -gt 0 ]; do [ "$1" = -o ] && dest=$2; shift; done
echo "redline of $base" > "$dest"
"""
# the bench's word_pdf.py, stubbed: a document whose text says "crash" gets
# no PDF, one that says "neighbour" gets none beside another document, and
# every other one gets <stem>_byword.pdf, as the real script names it
WORD_PDF = """import argparse, pathlib, sys
p = argparse.ArgumentParser()
for flag in ("--src", "--out", "--timeout", "--log"):
    p.add_argument(flag)
a = p.parse_args()
src = pathlib.Path(a.src)
out = pathlib.Path(a.out or a.src)
out.mkdir(parents=True, exist_ok=True)
docs = sorted(src.glob("*.docx"))
with open(a.log, "a") as log:
    for d in docs:
        text = d.read_text()
        if "crash" in text or ("neighbour" in text and len(docs) > 1):
            log.write(f"FAIL {d.name}: Word crashed on it in 2 passes\\n")
            continue
        (out / f"{d.stem}_byword.pdf").write_text("%PDF")
        log.write(f"ok {d.name}\\n")
sys.exit(0)
"""
# `uv run --script FILE ARGS…` runs FILE with python3; every call is logged
UV = """#!/bin/sh
echo "uv $* (cwd $(pwd))" >> "$WORD_CALLS"
[ "$1" = run ] && [ "$2" = --script ] || exit 9
shift 2
exec python3 "$@"
"""
WATCHDOG = """#!/bin/sh
echo "watchdog WD_MAX=$WD_MAX $*" >> "$WORD_CALLS"
exec sleep 60
"""
RELEASE_HEADER = ("key,base,base_sha256,next,next_sha256,docx,docx_sha256,pdf,pdf_sha256,state,id,sets,oracle,"
                  "oracle_pdf,oracle_pdf_sha256,docxodus_pdf,docxodus_pdf_sha256,jubarte_docx,jubarte_docx_sha256,"
                  "jubarte_pdf,jubarte_pdf_sha256")


@unittest.skipUnless(shutil.which("zsh"), "redline-sweep.sh is a zsh script")
class Sweep(unittest.TestCase):
    """The whole sweep with a stub binary (#1757, before#29/#30): it reads the
    release_info sample layout as well as the bench mapping, names the binary
    it ran, and never passes a run that generated nothing."""

    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp(prefix="sweep-")).resolve()
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)
        self.src = self.tmp / "bench"
        (self.src / "corpus").mkdir(parents=True)
        for name in ("a.docx", "b.docx", "c.docx"):
            (self.src / "corpus" / name).write_text(name)
        self.bin = self.tmp / "jubarte"
        self.bin.write_text(STUB)
        self.bin.chmod(0o755)
        self.out = self.tmp / "out"
        self.argv = self.tmp / "argv.log"

    def sweep(self, csv_text: str) -> subprocess.CompletedProcess[str]:
        csv_path = self.tmp / "sample.csv"
        csv_path.write_bytes(csv_text.encode())
        return subprocess.run(["zsh", str(HERE / "redline-sweep.sh"), str(csv_path), str(self.src), str(self.out)],
                              env=dict(os.environ, JUBARTE_BIN=str(self.bin), ARGV_LOG=str(self.argv)),
                              capture_output=True, text=True, timeout=60)

    def release_row(self, key: str, base: str, nxt: str) -> str:
        return ",".join([key, base, "0" * 64, nxt, "0" * 64] + [""] * 16)

    def test_the_release_info_layout_is_read_crlf_and_all(self) -> None:
        rows = [RELEASE_HEADER, self.release_row("k1", "corpus/a.docx", "corpus/b.docx"),
                self.release_row("k2", "corpus/b.docx", "corpus/c.docx")]
        r = self.sweep("\r\n".join(rows) + "\r\n")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertIn("generation: ok=2 fail=0 skipped=0", r.stdout)
        self.assertEqual(sorted(p.name for p in self.out.glob("*.docx")), ["k1.docx", "k2.docx"])
        self.assertIn(f"{self.src}/corpus/a.docx {self.src}/corpus/b.docx -d 1970-01-01T00:00:00Z -o ",
                      self.argv.read_text())

    def test_the_binary_is_named_by_path_sha256_and_version(self) -> None:
        r = self.sweep(RELEASE_HEADER + "\n" + self.release_row("k1", "corpus/a.docx", "corpus/b.docx") + "\n")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        line = f"binary: {self.bin} sha256 {hashlib.sha256(self.bin.read_bytes()).hexdigest()} (jubarte 9.9.9)"
        self.assertIn(line, r.stdout)
        self.assertIn(line, (self.out / "sweep.log").read_text())

    def test_a_release_sample_source_that_is_missing_fails(self) -> None:
        r = self.sweep(RELEASE_HEADER + "\n" + self.release_row("k1", "corpus/a.docx", "corpus/b.docx") + "\n"
                       + self.release_row("k2", "corpus/a.docx", "corpus/gone.docx") + "\n")
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("k2", (self.out / "sweep.log").read_text())

    def test_the_bench_mapping_layout_still_works(self) -> None:
        r = self.sweep("pair_stem,base,next,origin,src_base,src_next\np1,x,y,o,corpus/a.docx,corpus/b.docx\n"
                       "p2,x,y,o,corpus/a.docx,corpus/gone.docx\n")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertIn("generation: ok=1 fail=0 skipped=1", r.stdout)

    def test_a_run_that_generated_nothing_fails(self) -> None:
        # 0.13.0: the release CSVs read as a mapping skipped every pair and exited 0
        r = self.sweep("pair_stem,base,next,origin,src_base,src_next\np1,x,y,o,corpus/gone.docx,corpus/b.docx\n")
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("no pair was generated", r.stderr)

    def test_a_csv_of_another_layout_is_refused(self) -> None:
        r = self.sweep("stem,docx\ns1,corpus/a.docx\n")
        self.assertEqual(r.returncode, 2, r.stdout)
        self.assertIn("sample.csv", r.stderr)


@unittest.skipUnless(shutil.which("zsh"), "redline-sweep.sh is a zsh script")
class Probe(unittest.TestCase):
    """--probe opens every redline through the bench's word_pdf.py, with the
    bench's watchdog on its log, and re-runs each failure alone under a fresh
    name before counting it (#1794: it drove Word through a hand-rolled
    osascript, against AGENTS.md)."""

    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp(prefix="probe-")).resolve()
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)
        self.bench = self.tmp / "bench"
        (self.bench / "corpus").mkdir(parents=True)
        (self.bench / "scripts").mkdir()
        for name in ("a", "b", "crash", "neighbour"):
            (self.bench / "corpus" / f"{name}.docx").write_text(name)
        (self.bench / "corpus" / "word_opens.docx").write_text("Word's redline")
        (self.bench / "corpus" / "word_crash.docx").write_text("Word's redline, crash")
        (self.bench / "scripts" / "word_pdf.py").write_text(WORD_PDF)
        self.write_exe(self.bench / "scripts" / "word_watchdog.sh", WATCHDOG)
        self.path = self.tmp / "path"
        self.path.mkdir()
        self.write_exe(self.path / "uv", UV)
        self.write_exe(self.tmp / "jubarte", STUB)
        self.out = self.tmp / "out"
        self.calls = self.tmp / "word_calls.log"

    @staticmethod
    def write_exe(path: Path, text: str) -> None:
        path.write_text(text)
        path.chmod(0o755)

    def row(self, key: str, base: str, word: str = "corpus/word_opens.docx") -> str:
        return ",".join([key, f"corpus/{base}.docx", "0" * 64, "corpus/b.docx", "0" * 64, word] + [""] * 15)

    def probe(self, *rows: str, **env: str) -> subprocess.CompletedProcess[str]:
        csv_path = self.tmp / "sample.csv"
        csv_path.write_text(RELEASE_HEADER + "\n" + "\n".join(rows) + "\n")
        full = dict(os.environ, JUBARTE_BIN=str(self.tmp / "jubarte"), ARGV_LOG=str(self.tmp / "argv.log"),
                    WORD_CALLS=str(self.calls), PATH=f"{self.path}:{os.environ['PATH']}")
        full.pop("NEUROTIC_DOCX_BENCH", None)
        full.update(env)
        return subprocess.run(["zsh", str(HERE / "redline-sweep.sh"), str(csv_path), str(self.bench), str(self.out),
                               "--probe"], env=full, capture_output=True, text=True, timeout=120)

    def test_every_redline_goes_through_the_bench_word_pdf_and_its_watchdog(self) -> None:
        r = self.probe(self.row("k1", "a"), self.row("k2", "b"))
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertIn("word-open probe: opened=2 failed=0", r.stdout)
        calls = self.calls.read_text()
        self.assertIn(f"uv run --script scripts/word_pdf.py --src {self.out}/.probe/batch", calls)
        self.assertIn("--timeout 300", calls)
        self.assertIn(f"(cwd {self.bench})", calls)
        self.assertIn("watchdog WD_MAX=300 ", calls)

    def test_a_redline_that_fails_alone_too_is_a_probe_failure(self) -> None:
        r = self.probe(self.row("k1", "a"), self.row("k2", "crash"))
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("word-open probe: opened=1 failed=1", r.stdout)
        log = (self.out / "probe.log").read_text()
        self.assertIn("PROBEFAIL k2", log)
        self.assertNotIn("PROBEFAIL k1", log)
        # the re-run is the file alone, under a short name of its own
        self.assertIn(f"--src {self.out}/.probe/alone/r1 ", self.calls.read_text())

    def test_a_redline_that_fails_only_beside_others_opens_alone(self) -> None:
        r = self.probe(self.row("k1", "a"), self.row("k2", "neighbour"))
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertIn("word-open probe: opened=2 failed=0", r.stdout)
        self.assertIn("k2 opened alone", (self.out / "probe.log").read_text())

    def test_a_failure_whose_word_redline_fails_too_says_so_and_still_fails(self) -> None:
        r = self.probe(self.row("k1", "crash", "corpus/word_crash.docx"), self.row("k2", "crash"))
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("word-open probe: opened=0 failed=2 (Word's own redline fails too: 1)", r.stdout)
        log = (self.out / "probe.log").read_text()
        self.assertIn("PROBEFAIL k1 (Word's own redline of the pair fails too: corpus/word_crash.docx)", log)
        self.assertIn("PROBEFAIL k2 (Word's own redline of the pair opens: corpus/word_opens.docx)", log)

    def test_a_bench_without_word_pdf_is_refused_before_anything_is_generated(self) -> None:
        (self.bench / "scripts" / "word_pdf.py").unlink()
        r = self.probe(self.row("k1", "a"))
        self.assertEqual(r.returncode, 2, r.stdout)
        self.assertIn("word_pdf.py", r.stderr)
        self.assertFalse((self.tmp / "argv.log").exists())

    def test_neurotic_docx_bench_names_the_bench_holding_the_scripts(self) -> None:
        other = self.tmp / "other_bench"
        shutil.copytree(self.bench / "scripts", other / "scripts")
        (self.bench / "scripts" / "word_pdf.py").unlink()
        r = self.probe(self.row("k1", "a"), NEUROTIC_DOCX_BENCH=str(other))
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertIn(f"(cwd {other})", self.calls.read_text())

    def test_the_sweep_drives_no_word_of_its_own(self) -> None:
        text = (HERE / "redline-sweep.sh").read_text()
        self.assertNotIn("osascript", text)
        self.assertNotIn("word-open-probe", text)
        self.assertFalse((HERE / "word-open-probe.sh").exists())
        self.assertFalse((HERE / "word-probe-sweep.sh").exists())


@unittest.skipUnless(shutil.which("zsh"), "redline-sweep.sh is a zsh script")
class ValidationSweep(unittest.TestCase):
    """Run the real shell pipeline with fake jubarte and dotnet executables."""

    setUp = Sweep.setUp

    def validate(self, rows, baseline="ran\tOldError\nmissing\tKnownError\n",
                 dotnet_body="exit 0\n"):
        scripts = self.tmp / "scripts"
        scripts.mkdir(exist_ok=True)
        script = scripts / "redline-sweep.sh"
        shutil.copy(HERE / "redline-sweep.sh", script)
        (self.tmp / "tools/validate-docx").mkdir(parents=True, exist_ok=True)
        dotnet = self.tmp / "dotnet"
        dotnet.write_text("#!/bin/sh\n" + dotnet_body)
        dotnet.chmod(0o755)
        baseline_path = self.tmp / "baseline.tsv"
        baseline_path.write_text(baseline)
        csv_path = self.tmp / "mapping.csv"
        csv_path.write_text("pair_stem,base,next,origin,src_base,src_next\n" + rows)
        return subprocess.run(
            ["zsh", str(script), str(csv_path), str(self.src), str(self.out), "--validate"],
            env=dict(os.environ, PATH=f"{self.tmp}:{os.environ['PATH']}",
                     VALIDITY_BASELINE=str(baseline_path), JUBARTE_BIN=str(self.bin),
                     ARGV_LOG=str(self.argv)), capture_output=True, text=True, timeout=10,
        )

    def test_skipped_baselined_stem_refuses_the_sweep(self):
        r = self.validate("ran,x,y,o,corpus/a.docx,corpus/b.docx\n"
                          "missing,x,y,o,corpus/a.docx,corpus/gone.docx\n")
        self.assertEqual(r.returncode, 1, "a skipped baseline stem must fail: " + r.stdout)
        self.assertIn("VALIDATOR NOT RUN", r.stdout)
        self.assertIn("missing\tKnownError", r.stdout)

    def test_skipped_keys_are_never_reported_fixed_even_with_stale_output(self):
        self.out.mkdir()
        (self.out / "missing.docx").write_text("left over from a previous sweep")
        r = self.validate("ran,x,y,o,corpus/a.docx,corpus/b.docx\n"
                          "missing,x,y,o,corpus/a.docx,corpus/gone.docx\n")
        fixed = r.stdout.split("VALIDATOR FIXED (re-bless baseline?):", 1)[1]
        self.assertIn("ran\tOldError", fixed)
        self.assertNotIn("missing\tKnownError", fixed)

    def test_a_crashed_validator_is_not_run_never_fixed(self):
        # validate-docx exits 0 or 1; any other code is a crash that validated nothing.
        crash = 'case "$*" in *missing.docx*) exit 134;; esac\nexit 0\n'
        r = self.validate("ran,x,y,o,corpus/a.docx,corpus/b.docx\n"
                          "missing,x,y,o,corpus/a.docx,corpus/b.docx\n", dotnet_body=crash)
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("VALIDATOR NOT RUN", r.stdout)
        self.assertNotIn("missing\tKnownError", r.stdout.split("VALIDATOR FIXED", 1)[1])

    def test_baselined_stem_absent_from_mapping_is_not_run(self):
        r = self.validate("ran,x,y,o,corpus/a.docx,corpus/b.docx\n")
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("VALIDATOR NOT RUN", r.stdout)

    def test_failed_generation_is_not_fixed(self):
        self.bin.write_text(STUB.replace('base=$1', 'case "$1" in *c.docx) exit 1;; esac\nbase=$1'))
        r = self.validate("ran,x,y,o,corpus/a.docx,corpus/b.docx\n"
                          "missing,x,y,o,corpus/c.docx,corpus/b.docx\n")
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("VALIDATOR NOT RUN", r.stdout)
        self.assertNotIn("missing\tKnownError", r.stdout.split("VALIDATOR FIXED", 1)[1])

    def test_missing_generated_output_is_not_fixed(self):
        self.bin.write_text(STUB.replace('base=$1', 'case "$1" in *c.docx) exit 0;; esac\nbase=$1'))
        r = self.validate("ran,x,y,o,corpus/a.docx,corpus/b.docx\n"
                          "missing,x,y,o,corpus/c.docx,corpus/b.docx\n")
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("VALIDATOR NOT RUN", r.stdout)

    def test_no_validated_outputs_means_no_fixed_keys(self):
        self.bin.write_text(STUB.replace('base=$1', 'exit 0\nbase=$1'))
        r = self.validate("ran,x,y,o,corpus/a.docx,corpus/b.docx\n")
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("VALIDATOR NOT RUN", r.stdout)
        self.assertNotIn("VALIDATOR FIXED", r.stdout)

    def test_completed_baseline_can_be_fixed_with_unbaselined_skips(self):
        r = self.validate("ran,x,y,o,corpus/a.docx,corpus/b.docx\n"
                          "other,x,y,o,corpus/a.docx,corpus/gone.docx\n", "ran\tOldError\n")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertIn("ran\tOldError", r.stdout)
        self.assertNotIn("VALIDATOR NOT RUN", r.stdout)


class StageSources(unittest.TestCase):
    """Filesystem integration: preserve bytes, resolve pool aliases, fail closed."""

    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp(prefix="stage-sources-"))
        self.addCleanup(shutil.rmtree, self.tmp)
        self.bench = self.tmp / "bench"
        self.word = self.bench / "corpus/word"
        (self.word / "pools").mkdir(parents=True)
        (self.word / "clean/docx").mkdir(parents=True)
        self.pool = self.word / "pools/word_based_pairs.csv"
        self.pool.write_text("key,base,next,base_name,next_name,docx,pdf,state\r\n"
                             "key,clean/docx/123_a,clean/docx/456_b,a,b,,,\r\n")
        (self.word / "clean/docx/123_a.docx").write_bytes(b"original a")
        (self.word / "clean/docx/456_b.docx").write_bytes(b"original b")
        self.local = self.tmp / "docx_source"
        self.local.mkdir()
        self.mapping = self.tmp / "mapping.csv"
        self.mapping.write_text("pair_stem,base,next,origin,docx_source_base,docx_source_next\n"
                                "old_pair,a,b,both,a.docx,b.docx\n")
        self.out = self.tmp / "staged"

    def stage(self):
        return subprocess.run(
            [os.sys.executable, str(HERE / "stage_ring2_sources.py"), str(self.out),
             "--mapping", str(self.mapping)],
            env=dict(os.environ, NEUROTIC_DOCX_BENCH=str(self.bench)),
            capture_output=True, text=True, timeout=10,
        )

    def test_stages_pool_bytes_under_legacy_names_and_repairs_dangling_output(self):
        self.out.mkdir()
        (self.out / "a.docx").symlink_to(self.tmp / "gone.docx")
        r = self.stage()
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual((self.out / "a.docx").read_bytes(), b"original a")
        self.assertEqual((self.out / "b.docx").read_bytes(), b"original b")
        self.assertFalse((self.out / "a.docx").is_symlink())
        self.assertEqual(self.stage().returncode, 0)

    def test_local_original_survives_a_missing_pool_file(self):
        (self.word / "clean/docx/123_a.docx").unlink()
        (self.local / "a.docx").write_bytes(b"local original")
        r = self.stage()
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual((self.out / "a.docx").read_bytes(), b"local original")

    def test_dangling_local_source_falls_back_to_pool(self):
        (self.local / "a.docx").symlink_to(self.tmp / "gone.docx")
        r = self.stage()
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual((self.out / "a.docx").read_bytes(), b"original a")

    def test_missing_sources_fail_and_name_documents_and_pairs(self):
        (self.word / "clean/docx/456_b.docx").unlink()
        (self.word / "clean/docx/456_b.docx").symlink_to(self.tmp / "missing-blob")
        r = self.stage()
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("b.docx", r.stderr)
        self.assertIn("old_pair", r.stderr)
        self.assertIn("uv run bench fixtures download", r.stderr)
        self.assertIn("maintainer", r.stderr)

    def test_ambiguous_pool_alias_is_refused(self):
        with self.pool.open("a") as f:
            f.write("other,clean/docx/different_a,clean/docx/456_b,a,b,,,\n")
        r = self.stage()
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("ambiguous", r.stderr)
        self.assertIn("a.docx", r.stderr)

    def test_local_and_pool_byte_mismatch_is_refused(self):
        (self.local / "a.docx").write_bytes(b"different a")
        r = self.stage()
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("different bytes", r.stderr)

    def test_existing_regular_output_is_never_overwritten_with_different_bytes(self):
        self.out.mkdir()
        (self.out / "a.docx").write_bytes(b"keep me")
        r = self.stage()
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertEqual((self.out / "a.docx").read_bytes(), b"keep me")

    def test_invalid_pool_columns_and_empty_values_are_refused(self):
        for contents in ("wrong,columns\n", "base,next,base_name,next_name\n",
                         "base,next,base_name,next_name\nclean/a,clean/b,,b\n"):
            with self.subTest(contents=contents):
                self.pool.write_text(contents)
                r = self.stage()
                self.assertEqual(r.returncode, 1, r.stderr)
                self.assertIn("word_based_pairs.csv", r.stderr)

    def test_pool_paths_and_mapping_names_cannot_escape_their_folders(self):
        self.pool.write_text(self.pool.read_text().replace("clean/docx/123_a", "../outside"))
        r = self.stage()
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("escapes corpus/word", r.stderr)
        self.pool.write_text(self.pool.read_text().replace("../outside", "clean/docx/123_a"))
        self.mapping.write_text(self.mapping.read_text().replace("a.docx", "../a.docx"))
        r = self.stage()
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("must be a .docx filename", r.stderr)

    def rename_ledger(self, digest=None, corpus_set="word_based", pool_path="clean/docx/123_a.docx"):
        notices = self.word / "notices"
        notices.mkdir(exist_ok=True)
        digest = digest or hashlib.sha256(b"original a").hexdigest()
        (notices / "RENAMED.csv").write_text(
            "original,new,id,sha256,set\n"
            f"grok_run/word_based/docx_source/legacy_a.docx,{pool_path},123,{digest},{corpus_set}\n"
        )
        self.mapping.write_text("pair_stem,base,next,origin,docx_source_base,docx_source_next\n"
                                "old_pair,a,b,both,legacy_a.docx,b.docx\n")

    def test_rename_ledger_recovers_legacy_alias_with_verified_pool_bytes(self):
        self.rename_ledger()
        r = self.stage()
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual((self.out / "legacy_a.docx").read_bytes(), b"original a")

    def test_rename_ledger_hash_mismatch_is_refused(self):
        self.rename_ledger(digest="0" * 64)
        r = self.stage()
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("sha256 mismatch", r.stderr)
        self.assertFalse((self.out / "legacy_a.docx").exists())

    def test_conflicting_rename_hashes_are_refused(self):
        self.rename_ledger()
        with (self.word / "notices/RENAMED.csv").open("a") as stream:
            stream.write("grok_run/word_based/docx_source/legacy_a.docx,"
                         f"clean/docx/123_a.docx,123,{'0' * 64},word_based\n")
        r = self.stage()
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("conflicting rename hashes", r.stderr)

    def test_rename_from_another_set_or_outside_the_pool_is_not_used(self):
        for kwargs in ({"corpus_set": "nocomments"}, {"pool_path": "other.docx"}):
            with self.subTest(kwargs=kwargs):
                self.rename_ledger(**kwargs)
                r = self.stage()
                self.assertEqual(r.returncode, 1, r.stderr)
                self.assertIn("MISSING", r.stderr)
                self.assertFalse((self.out / "legacy_a.docx").exists())

    def test_missing_pool_is_reported(self):
        self.pool.unlink()
        r = self.stage()
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("word_based_pairs.csv", r.stderr)


if __name__ == "__main__":
    unittest.main()

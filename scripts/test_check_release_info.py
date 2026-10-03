#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

# SPDX-License-Identifier: AGPL-3.0-only

"""scripts/check_release_info.py: the six evidence files a release needs.

Each case builds a throwaway release_info/ folder — a sound one, then one
broken at a time — and runs the checker as a subprocess, exactly as
scripts/release.sh does. Every rule the v2 checker adds has a test that
shows it fail on the broken shape and pass on the sound one. Nothing here
needs the network, git or the bench.
"""

from __future__ import annotations

import hashlib
import json
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
CHECK = HERE / "check_release_info.py"

VER = "0.11.2"
STAMP = "10-03-26_16-51"
ROWS = 3  # --rows keeps the fixture small; the release itself checks 600
A = "a" * 64
B = "b" * 64
C40 = "c" * 40  # the engine commit the candidate was built from
D64 = "d" * 64  # the candidate binary's sha256

REDLINE_HEADER = "key,base,base_sha256,next,next_sha256,docx,docx_sha256,pdf,pdf_sha256,state,id,sets,oracle,oracle_pdf,oracle_pdf_sha256,docxodus_pdf,docxodus_pdf_sha256,jubarte_docx,jubarte_docx_sha256,jubarte_pdf,jubarte_pdf_sha256"
CONVERSION_HEADER = "state,stem,docx,docx_sha256,word_pdf,word_pdf_sha256,jubarte_pdf,jubarte_pdf_sha256,soffice_pdf,soffice_pdf_sha256"
# the five keys scripts/check_release_facts.py reads to call a release done
FACTS_KEYS = ("engine.version", "engine.released", "release.history",
              "release.archives", "release.wheels")


def sha(text: str) -> str:
    return hashlib.sha256(text.encode()).hexdigest()


def redline_row(i: int) -> str:
    return (f"k{i},corpus/b.docx,{A},corpus/n.docx,{B},corpus/c.docx,{A},corpus/c.pdf,{B},"
            f"clean,id{i},sets,corpus,results/oracle/k{i}.pdf,{A},results/docxodus/k{i}.pdf,{B},"
            f"results/jubarte/docx/k{i}.docx,{A},results/jubarte/pdf/k{i}.pdf,{B}")


def conversion_row(i: int) -> str:
    return f"clean,s{i},corpus/d.docx,{A},corpus/w.pdf,{B},results/jubarte/s{i}.pdf,{A},results/soffice/s{i}.pdf,{B}"


class SixFiles(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp(prefix="release_info_"))
        self.info = self.tmp / "release_info"
        self.info.mkdir()
        self.write_six()

    def tearDown(self) -> None:
        shutil.rmtree(self.tmp, ignore_errors=True)

    # A sound set: every file present, shas right, aggregates sound, jubarte
    # carrying its identity, the JSONLs holding the required keys.
    def write_six(self, rows: int = ROWS) -> None:
        redline = "\n".join([REDLINE_HEADER] + [redline_row(i) for i in range(rows)]) + "\n"
        conversion = "\n".join([CONVERSION_HEADER] + [conversion_row(i) for i in range(rows)]) + "\n"
        (self.info / f"sample_redline_{VER}_{STAMP}.csv").write_text(redline)
        (self.info / f"sample_conversion_{VER}_{STAMP}.csv").write_text(conversion)
        for name, sample, tool2 in (
            ("results_redline", f"sample_redline_{VER}_{STAMP}.csv", "docxodus"),
            ("results_conversion", f"sample_conversion_{VER}_{STAMP}.csv", "soffice"),
        ):
            doc = {
                "sample": {"csv": sample, "sha256": sha((self.info / sample).read_text()),
                           "n": rows, "drawn": {"seed": 20261003}},
                "tools": {
                    "jubarte": {"n": rows, "failures": 1, "mean": 80.0, "median": 84.0,
                                "exact_100": 1, "at_least_90": 2, "version": f"jubarte {VER}",
                                "commit": C40, "binary_sha256": D64,
                                "candidate_reports": f"jubarte 0.11.1"},
                    tool2: {"n": rows, "failures": 2, "mean": 60.0, "median": 62.0,
                            "exact_100": 0, "at_least_90": 1, "version": tool2},
                },
                "comparison": {"comparator": tool2, "median_delta": 22.0, "ci95": [18.0, 26.0],
                               "bootstrap": {"reps": 2000, "seed": 42}},
            }
            (self.info / f"{name}_{VER}_{STAMP}.json").write_text(json.dumps(doc))
        website = [
            {"id": "018f" + "0" * 12, "ts": "2026-10-03T16:51:00.000Z", "key": "engine.version",
             "value": VER, "source": "release_info evidence"},
            {"key": "engine.released", "value": "<the CHANGELOG date>", "pending": True},
            {"key": "release.archives", "value": "<filled by sync-release.ts>", "pending": True},
            {"key": "release.wheels", "value": "<filled by sync-release.ts>", "pending": True},
            {"key": "release.history", "value": "<the release list>", "pending": True},
            {"key": "bench.tables", "value": [{"id": "conversion-sample"}]},
        ]
        app = [
            {"key": "app.version/package.json", "value": VER, "file": "package.json", "where": "line 4"},
            {"key": "app.changelog", "value": "## [0.11.2]", "file": "CHANGELOG.md", "where": "the new ## heading"},
        ]
        (self.info / f"website_data_{VER}_{STAMP}.jsonl").write_text(
            "".join(json.dumps(r) + "\n" for r in website))
        (self.info / f"app_data_{VER}_{STAMP}.jsonl").write_text(
            "".join(json.dumps(r) + "\n" for r in app))

    def check(self, *args: str, ver: str = VER, rows: int | None = ROWS) -> subprocess.CompletedProcess[str]:
        argv = ["python3", str(CHECK), ver, "--root", str(self.tmp)]
        if rows is not None:
            argv += ["--rows", str(rows)]
        return subprocess.run([*argv, *args], capture_output=True, text=True, timeout=120)

    def rewrite_sample(self, name: str, text: str) -> None:
        (self.info / f"{name}_{VER}_{STAMP}.csv").write_text(text)

    # ------------------------------------------------------------- sound set
    def test_a_sound_set_passes_and_prints_the_aggregates(self) -> None:
        r = self.check()
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("six files", r.stdout)
        self.assertIn("jubarte: n=3 failures=1 mean=80.0 median=84.0", r.stdout)
        self.assertIn("95% CI [18.0, 26.0] (2000 resamples, seed 42)", r.stdout)
        self.assertIn(f"binary sha256 {D64}", r.stdout)
        self.assertIn("format-checked only", r.stdout)  # no --bench-root: said out loud

    def test_the_default_row_count_is_600(self) -> None:
        for name in ("sample_redline", "sample_conversion"):
            self.rewrite_sample(name, (REDLINE_HEADER if name == "sample_redline" else CONVERSION_HEADER) + "\n")
        for name in ("results_redline", "results_conversion"):
            path = self.info / f"{name}_{VER}_{STAMP}.json"
            doc = json.loads(path.read_text())
            doc["sample"]["sha256"] = sha((self.info / doc["sample"]["csv"]).read_text())
            path.write_text(json.dumps(doc))
        r = self.check(rows=None)  # no --rows: the default applies
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("0 data rows, want 600", r.stderr)
        self.write_six(rows=600)
        r = self.check(rows=None)
        self.assertEqual(r.returncode, 0, r.stderr)

    # ------------------------------------------------------------- v1 rules
    def test_each_missing_file_is_named(self) -> None:
        for path in sorted(self.info.glob(f"*_{VER}_{STAMP}.*")):
            gone = path.with_suffix(".gone")
            path.rename(gone)
            r = self.check()
            self.assertEqual(r.returncode, 1, f"{path.name} removed: {r.stdout}")
            self.assertIn(path.name.split(f"_{VER}")[0], r.stderr)
            gone.rename(path)

    def test_two_stamps_of_one_file_is_ambiguous(self) -> None:
        shutil.copy(self.info / f"sample_redline_{VER}_{STAMP}.csv",
                    self.info / f"sample_redline_{VER}_10-04-26_09-00.csv")
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("2 files for 0.11.2", r.stderr)

    def test_six_files_with_different_stamps_are_refused(self) -> None:
        path = self.info / f"app_data_{VER}_{STAMP}.jsonl"
        path.rename(self.info / f"app_data_{VER}_10-04-26_09-00.jsonl")
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("stamps", r.stderr)

    def test_the_row_count_is_checked(self) -> None:
        path = self.info / f"sample_conversion_{VER}_{STAMP}.csv"
        lines = path.read_text().splitlines()
        path.write_text("\n".join(lines[:-1]) + "\n")
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn(f"{ROWS - 1} data rows, want {ROWS}", r.stderr)

    def test_the_header_is_checked(self) -> None:
        self.rewrite_sample("sample_redline", REDLINE_HEADER.replace("oracle_pdf_sha256", "oracle_pdf_hash") + "\n")
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("header is", r.stderr)

    def test_a_sha_that_is_not_64_lowercase_hex_fails(self) -> None:
        self.rewrite_sample("sample_redline",
                            REDLINE_HEADER + "\nk1,corpus/b.docx,ABC,corpus/n.docx," + ",".join([""] * 16) + "\n")
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("base_sha256 is not 64 lowercase hex", r.stderr)

    def test_a_path_without_its_sha_fails_and_a_sha_without_its_path_fails(self) -> None:
        row = (f"clean,s1,corpus/d.docx,,corpus/w.pdf,{B},results/j.pdf,{A},"
               f",{B}")  # docx without sha; soffice_pdf without path
        self.rewrite_sample("sample_conversion", CONVERSION_HEADER + "\n" + row + "\n" + "\n".join(
            [f"clean,s{i},corpus/d.docx,{A},corpus/w.pdf,{B},results/j.pdf,{A},results/s.pdf,{B}"
             for i in (2, 3)]) + "\n")
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("docx has no sha256", r.stderr)
        self.assertIn("soffice_pdf_sha256 without soffice_pdf", r.stderr)

    def test_a_results_json_must_name_its_sample_csv_with_its_real_sha(self) -> None:
        path = self.info / f"results_redline_{VER}_{STAMP}.json"
        doc = json.loads(path.read_text())
        doc["sample"]["sha256"] = "0" * 64
        path.write_text(json.dumps(doc))
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("hashes to", r.stderr)

    def test_a_results_json_needs_both_tools_aggregates(self) -> None:
        path = self.info / f"results_conversion_{VER}_{STAMP}.json"
        doc = json.loads(path.read_text())
        del doc["tools"]["jubarte"]["median"]
        path.write_text(json.dumps(doc))
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("tools.jubarte.median is not a number", r.stderr)
        doc["tools"] = {"jubarte": doc["tools"]["jubarte"]}
        path.write_text(json.dumps(doc))
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("no comparator beside jubarte", r.stderr)

    def test_each_jsonl_line_must_parse(self) -> None:
        path = self.info / f"website_data_{VER}_{STAMP}.jsonl"
        path.write_text('{"key": "a"}\nnot json\n')
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("line 2 is not JSON", r.stderr)

    # ------------------------------------------------- (a) rows, widths, keys
    def test_a_row_must_have_exactly_the_headers_cell_count(self) -> None:
        row = redline_row(1) + ",extra"
        self.rewrite_sample("sample_redline", REDLINE_HEADER + "\n" + redline_row(0) + "\n"
                            + row + "\n" + redline_row(2) + "\n")
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("22 cells, want the header's 21", r.stderr)

    def test_a_row_needs_a_non_empty_key_and_stem(self) -> None:
        row = redline_row(1).replace("k1,", ",", 1)
        self.rewrite_sample("sample_redline", REDLINE_HEADER + "\n" + row + "\n"
                            + redline_row(0) + "\n" + redline_row(2) + "\n")
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("key is empty", r.stderr)
        row = conversion_row(1).replace(",s1,", ",,", 1)
        self.rewrite_sample("sample_conversion", CONVERSION_HEADER + "\n" + row + "\n"
                            + conversion_row(0) + "\n" + conversion_row(2) + "\n")
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("stem is empty", r.stderr)

    def test_duplicate_keys_fail(self) -> None:
        self.rewrite_sample("sample_redline", REDLINE_HEADER + "\n" + redline_row(0) + "\n"
                            + redline_row(0) + "\n" + redline_row(2) + "\n")
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("key k0 is listed twice", r.stderr)
        self.rewrite_sample("sample_conversion", CONVERSION_HEADER + "\n" + conversion_row(1) + "\n"
                            + conversion_row(1) + "\n" + conversion_row(2) + "\n")
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("stem s1 is listed twice", r.stderr)

    def test_blank_lines_are_not_rows(self) -> None:
        self.rewrite_sample("sample_conversion", CONVERSION_HEADER + "\n" + conversion_row(0) + "\n\n\n"
                            + conversion_row(1) + "\n")
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn(f"2 data rows, want {ROWS}", r.stderr)  # the blank lines did not count
        self.assertNotIn("cells, want", r.stderr)  # nor were they checked as rows

    def test_every_row_needs_its_word_oracle(self) -> None:
        row = redline_row(1).replace("results/oracle/k1.pdf,{}".format(A), ",")
        self.rewrite_sample("sample_redline", REDLINE_HEADER + "\n" + redline_row(0) + "\n"
                            + row + "\n" + redline_row(2) + "\n")
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("oracle_pdf is empty — every sampled compare must carry its Word oracle PDF", r.stderr)
        row = conversion_row(1).replace("corpus/w.pdf,{}".format(B), ",")
        self.rewrite_sample("sample_conversion", CONVERSION_HEADER + "\n" + conversion_row(0) + "\n"
                            + row + "\n" + conversion_row(2) + "\n")
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("word_pdf is empty — every sampled fixture must carry Word's own PDF", r.stderr)

    # ----------------------------------------------------- (b) results rules
    def _edit_results(self, name: str, **change) -> None:
        path = self.info / f"{name}_{VER}_{STAMP}.json"
        doc = json.loads(path.read_text())
        tools = change.pop("tools", None)
        if tools:
            for tool, fields in tools.items():
                for field, value in fields.items():
                    if value is None:
                        (doc["tools"][tool] if tool in doc["tools"] else doc["tools"].setdefault(tool, {})).pop(field, None)
                    else:
                        doc["tools"].setdefault(tool, {})[field] = value
        doc.update(change)
        path.write_text(json.dumps(doc))

    def test_tools_n_must_equal_the_samples_row_count(self) -> None:
        self._edit_results("results_redline", tools={"jubarte": {"n": ROWS + 2},
                                                     "docxodus": {"n": ROWS + 2}})
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn(f"tools.jubarte.n is {ROWS + 2}, the sample holds {ROWS} rows", r.stderr)
        self.assertIn(f"tools.docxodus.n is {ROWS + 2}, the sample holds {ROWS} rows", r.stderr)
        self._edit_results("results_conversion", sample={"n": ROWS + 2})
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn(f"sample.n is {ROWS + 2}, the sample holds {ROWS} rows", r.stderr)

    def test_failures_must_stay_within_zero_and_n(self) -> None:
        self._edit_results("results_redline", tools={"jubarte": {"failures": ROWS + 1}})
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn(f"tools.jubarte.failures is {ROWS + 1}, want 0..{ROWS}", r.stderr)
        self._edit_results("results_redline", tools={"jubarte": {"failures": -1}})
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("tools.jubarte.failures is -1", r.stderr)

    def test_mean_and_median_must_stay_within_0_and_100(self) -> None:
        self._edit_results("results_conversion", tools={"jubarte": {"mean": 100.5}})
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("tools.jubarte.mean is 100.5, want 0..100", r.stderr)
        self._edit_results("results_conversion", tools={"soffice": {"median": -0.5}})
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("tools.soffice.median is -0.5, want 0..100", r.stderr)

    def test_jubarte_must_carry_its_identity(self) -> None:
        path = self.info / f"results_redline_{VER}_{STAMP}.json"
        sound = path.read_text()
        for field, bad, message in (
            ("commit", None, "tools.jubarte.commit is not 40 lowercase hex"),
            ("commit", "c" * 41, "tools.jubarte.commit is not 40 lowercase hex"),
            ("binary_sha256", "d" * 63, "tools.jubarte.binary_sha256 is not 64 lowercase hex"),
            ("version", None, "tools.jubarte.version is not a non-empty string"),
            ("version", "jubarte 0.11.1", "which does not name 0.11.2"),
        ):
            doc = json.loads(sound)
            if bad is None:
                doc["tools"]["jubarte"].pop(field, None)
            else:
                doc["tools"]["jubarte"][field] = bad
            path.write_text(json.dumps(doc))
            r = self.check()
            self.assertEqual(r.returncode, 1, f"{field}={bad!r}: {r.stdout}")
            self.assertIn(message, r.stderr)
            path.write_text(sound)  # restored: the sound set passes again
            self.assertEqual(self.check().returncode, 0, r.stderr)

    def test_commit_and_binary_sha_arguments_bind_the_evidence(self) -> None:
        self.assertEqual(self.check("--commit", C40, "--binary-sha256", D64).returncode, 0)
        r = self.check("--commit", "e" * 40)
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn(f"tools.jubarte.commit is {C40}, not the given {'e' * 40}", r.stderr)
        r = self.check("--binary-sha256", "e" * 64)
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn(f"tools.jubarte.binary_sha256 is {D64}, not the given {'e' * 64}", r.stderr)
        for bad in (("--commit", "nothex"), ("--binary-sha256", "short")):
            r = self.check(*bad)
            self.assertEqual(r.returncode, 2, r.stderr)

    # ------------------------------------------------------- (c) real stamps
    def test_the_stamp_must_be_a_real_date_and_time(self) -> None:
        for stamp in ("99-99-99_99-99", "02-30-26_10-00", "10-03-26_24-00"):
            path = self.info / f"sample_redline_{VER}_{STAMP}.csv"
            other = self.info / f"sample_redline_{VER}_{stamp}.csv"
            path.rename(other)
            r = self.check()
            self.assertEqual(r.returncode, 1, stamp)
            self.assertIn(f"stamp {stamp} is not a real date and time", r.stderr)
            other.rename(path)
        self.assertEqual(self.check().returncode, 0)

    # --------------------------------------------- (d) no seventh/mistyped file
    def test_a_file_that_carries_the_version_but_is_not_one_of_the_six_fails(self) -> None:
        (self.info / f"sample_redline_{VER}_16-51.csv").write_text("x\n")  # mistyped: no date
        (self.info / f"results_extra_{VER}_{STAMP}.json").write_text("{}")  # unknown definition
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn(f"sample_redline_{VER}_16-51.csv", r.stderr)
        self.assertIn("not one of the six names", r.stderr)
        self.assertIn(f"results_extra_{VER}_{STAMP}.json", r.stderr)
        self.assertIn("not one of the six definitions", r.stderr)
        # another version's files are none of this release's business
        (self.info / f"sample_redline_{VER}_16-51.csv").unlink()
        (self.info / f"results_extra_{VER}_{STAMP}.json").unlink()
        (self.info / "sample_redline_0.11.1_10-03-26_09-00.csv").write_text("x\n")
        self.assertEqual(self.check().returncode, 0)

    # -------------------------------------------------------- (e) JSONL keys
    def test_each_record_needs_a_non_empty_string_key(self) -> None:
        path = self.info / f"app_data_{VER}_{STAMP}.jsonl"
        path.write_text('{"key": "app.changelog", "value": "x"}\n{"value": 1}\n{"key": ""}\n')
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("line 2: key is not a non-empty string", r.stderr)
        self.assertIn("line 3: key is not a non-empty string", r.stderr)

    def test_website_data_must_hold_the_keys_check_release_facts_reads(self) -> None:
        path = self.info / f"website_data_{VER}_{STAMP}.jsonl"
        records = [json.loads(line) for line in path.read_text().splitlines()]
        kept = [r for r in records if r["key"] != "release.wheels"]
        path.write_text("".join(json.dumps(r) + "\n" for r in kept))
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("no record keyed release.wheels — scripts/check_release_facts.py reads it", r.stderr)
        # the five keys it reads, pending placeholders included, are enough
        path.write_text("".join(json.dumps({"key": k, "pending": True}) + "\n" for k in FACTS_KEYS))
        r = self.check()
        self.assertEqual(r.returncode, 0, r.stderr)

    # ------------------------------------------------- (f) unreadable bytes
    def test_unreadable_bytes_give_a_clean_error_not_a_traceback(self) -> None:
        for name, ext in (("sample_redline", "csv"), ("results_redline", "json"), ("app_data", "jsonl")):
            path = self.info / f"{name}_{VER}_{STAMP}.{ext}"
            keep = path.read_bytes()
            path.write_bytes(b"\xff\xfe not utf-8 \x00\x81\x82binary\n")
            r = self.check()
            self.assertEqual(r.returncode, 1, name)
            self.assertIn("not UTF-8 text", r.stderr)
            self.assertNotIn("Traceback", r.stderr)
            path.write_bytes(keep)
        self.assertEqual(self.check().returncode, 0)

    # --------------------------------------------------- (g) --bench-root
    def test_bench_root_verifies_every_path_against_the_real_files(self) -> None:
        bench = self.tmp / "bench"
        rows = []
        for i in range(ROWS):
            cells = {}
            rels = [f"corpus/b{i}.docx", f"corpus/n{i}.docx", f"corpus/c{i}.docx", f"corpus/c{i}.pdf",
                    f"results/oracle/k{i}.pdf", f"results/docxodus/k{i}.pdf",
                    f"results/jubarte/docx/k{i}.docx", f"results/jubarte/pdf/k{i}.pdf"]
            for rel in rels:
                target = bench / rel
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text(f"content of {rel}\n")
                cells[rel] = sha(target.read_text())
            b, n = f"corpus/b{i}.docx", f"corpus/n{i}.docx"
            docx, pdf = f"corpus/c{i}.docx", f"corpus/c{i}.pdf"
            oracle, dx = f"results/oracle/k{i}.pdf", f"results/docxodus/k{i}.pdf"
            jdocx, jpdf = f"results/jubarte/docx/k{i}.docx", f"results/jubarte/pdf/k{i}.pdf"
            rows.append(
                f"k{i},{b},{cells[b]},{n},{cells[n]},{docx},{cells[docx]},{pdf},{cells[pdf]},"
                f"clean,id{i},sets,corpus,{oracle},{cells[oracle]},{dx},{cells[dx]},"
                f"{jdocx},{cells[jdocx]},{jpdf},{cells[jpdf]}")
        text = REDLINE_HEADER + "\n" + "\n".join(rows) + "\n"
        self.rewrite_sample("sample_redline", text)
        conv_rows = []
        for i in range(ROWS):
            cells = {}
            rels = [f"corpus/d{i}.docx", f"corpus/w{i}.pdf",
                    f"results/jubarte/c{i}.pdf", f"results/soffice/c{i}.pdf"]
            for rel in rels:
                target = bench / rel
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text(f"content of {rel}\n")
                cells[rel] = sha(target.read_text())
            conv_rows.append(
                f"clean,s{i},{rels[0]},{cells[rels[0]]},{rels[1]},{cells[rels[1]]},"
                f"{rels[2]},{cells[rels[2]]},{rels[3]},{cells[rels[3]]}")
        conv_text = CONVERSION_HEADER + "\n" + "\n".join(conv_rows) + "\n"
        self.rewrite_sample("sample_conversion", conv_text)
        for name, sample_text in (("results_redline", text), ("results_conversion", conv_text)):
            path = self.info / f"{name}_{VER}_{STAMP}.json"
            doc = json.loads(path.read_text())
            doc["sample"]["sha256"] = sha(sample_text)
            path.write_text(json.dumps(doc))
        r = self.check("--bench-root", str(bench))
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn(f"verified against {bench}", r.stdout)
        # a rewritten file under the bench root no longer matches its sha
        (bench / "corpus/b1.docx").write_text("tampered\n")
        r = self.check("--bench-root", str(bench))
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("corpus/b1.docx hashes to", r.stderr)
        # a deleted file is named, an absolute path is refused
        (bench / "corpus/b1.docx").unlink()
        r = self.check("--bench-root", str(bench))
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("corpus/b1.docx does not exist under", r.stderr)
        tampered = text.replace("corpus/b0.docx", "/abs/b0.docx")
        self.rewrite_sample("sample_redline", tampered)
        redline_doc = json.loads((self.info / f"results_redline_{VER}_{STAMP}.json").read_text())
        redline_doc["sample"]["sha256"] = sha(tampered)
        (self.info / f"results_redline_{VER}_{STAMP}.json").write_text(json.dumps(redline_doc))
        r = self.check("--bench-root", str(bench))
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("/abs/b0.docx is not relative to the bench root", r.stderr)
        r = self.check("--bench-root", str(self.tmp / "nowhere"))
        self.assertEqual(r.returncode, 2, r.stderr)


class Version(unittest.TestCase):
    def test_only_a_release_version_is_accepted(self) -> None:
        tmp = Path(tempfile.mkdtemp(prefix="release_info_ver_"))
        try:
            (tmp / "release_info").mkdir()
            for bad in ("0.11", "v0.11.2"):
                r = subprocess.run(
                    ["python3", str(CHECK), bad, "--root", str(tmp)],
                    capture_output=True, text=True, timeout=60,
                )
                self.assertEqual(r.returncode, 2, r.stderr)
            r = subprocess.run(
                ["python3", str(CHECK), "0.11.2", "--root", str(tmp)],
                capture_output=True, text=True, timeout=60,
            )
            self.assertEqual(r.returncode, 1, r.stderr)
            self.assertIn("no such file", r.stderr)
        finally:
            shutil.rmtree(tmp, ignore_errors=True)

    def test_a_missing_folder_is_explained(self) -> None:
        tmp = Path(tempfile.mkdtemp(prefix="release_info_empty_"))
        try:
            r = subprocess.run(
                ["python3", str(CHECK), "0.11.2", "--root", str(tmp)],
                capture_output=True, text=True, timeout=60,
            )
            self.assertEqual(r.returncode, 1, r.stderr)
            self.assertIn("needs its six files", r.stderr)
        finally:
            shutil.rmtree(tmp, ignore_errors=True)


class ReleaseSh(unittest.TestCase):
    """scripts/release.sh runs the checker as its step 3 and dies without it.

    Skipped where this file runs outside the engine checkout (release.sh
    sits beside this test only once the patch has installed it)."""

    @unittest.skipUnless((HERE / "release.sh").is_file(), "release.sh sits beside this test in the engine checkout")
    def step3(self) -> str:
        text = (HERE / "release.sh").read_text()
        start = text.index('say "3. release_info')
        return text[start:text.index('say "4.', start)]

    @unittest.skipUnless((HERE / "release.sh").is_file(), "release.sh sits beside this test in the engine checkout")
    def test_step_3_runs_the_check_and_dies_on_failure(self) -> None:
        step = self.step3()
        self.assertIn('python3 scripts/check_release_info.py "$VER"', step)
        self.assertIn("|| die", step)

    @unittest.skipUnless((HERE / "release.sh").is_file(), "release.sh sits beside this test in the engine checkout")
    def test_step_3_verifies_against_the_bench_when_it_exists(self) -> None:
        step = self.step3()
        self.assertIn('--bench-root "$BENCH_ROOT"', step)
        self.assertIn("NEUROTIC_DOCX_BENCH", step)
        self.assertIn("format-checked", step)  # the no-bench warning says what is NOT proven

    @unittest.skipUnless((HERE / "release.sh").is_file(), "release.sh sits beside this test in the engine checkout")
    def test_the_release_commit_carries_the_evidence(self) -> None:
        text = (HERE / "release.sh").read_text()
        start = text.index("git add Cargo.toml")
        add = text[start:text.index("git commit", start)]
        self.assertIn("release_info", add)


if __name__ == "__main__":
    unittest.main()

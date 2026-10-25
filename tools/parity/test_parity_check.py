# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only
"""Unit tests for parity_check.py v2's pure comparison logic (audit F10-F12
repairs). No binaries, no PyMuPDF, no corpus — compare_pdfs /
compare_painted_characters / parse_bindings / pdf_coverage_failure are pure.

    python3 -m pytest test_parity_v2.py -q
"""
import importlib.util
import json
import os
import shutil
import subprocess
from collections import Counter
from pathlib import Path

import pytest

HERE = Path(__file__).resolve().parent
_spec = importlib.util.spec_from_file_location("parity_check", HERE / "parity_check.py")
pc = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(pc)


def facts(pages, page_count=None):
    return {"pdf": b"", "sha": "0" * 64,
            "page_count": page_count if page_count is not None else len(pages),
            "pages_text": pages}


# --- F12: the WASM text tolerance ---------------------------------------------


def test_a_lost_bullet_fails():
    out = pc.compare_pdfs({"cli": facts(["• one", "• two"]), "wasm": facts(["one", "two"])})
    assert out["mismatches"], "a bullet painted only by the native build must fail"
    assert not out["same_characters"]


def test_a_bullet_drawn_by_another_bullet_character_passes():
    # • (U+2022) vs ▪ (U+25AA): different characters, same fold class
    # (Wingdings 0x9F vs 0xA0, src/convert/font.rs:184-185) — tolerated, and
    # the fold is what the waived column reports.
    out = pc.compare_pdfs({"cli": facts(["• one"]), "wasm": facts(["▪ one"])})
    assert out["same_characters"]["cli_wasm"]
    assert not out["mismatches"]
    assert out["waived"]["cli_wasm"]["symbols_folded"]["bullet-geo"] == 2


def test_an_arrow_replaced_by_a_bullet_fails():
    # v1 folded every symbol to one token; v2 keeps the classes apart.
    out = pc.compare_pdfs({"cli": facts(["→ next"]), "wasm": facts(["• next"])})
    assert out["mismatches"], "an arrow must not fold into the bullet class"
    assert not out["same_characters"]


def test_u037e_equals_semicolon():
    assert pc.painted_characters([";"]) == pc.painted_characters(["\u037e"]) == Counter({";": 1})
    out = pc.compare_pdfs({"cli": facts(["a;"]), "wasm": facts(["a\u037e"])})
    assert out["same_characters"]["cli_wasm"]


def test_a_moved_page_digit_passes_and_is_reported():
    out = pc.compare_pdfs({"cli": facts(["Page 1", "body"]), "wasm": facts(["Page 2", "body"])})
    assert out["same_characters"]["cli_wasm"]
    assert not out["mismatches"]
    assert out["page_dependent_digits"]["cli_wasm"] == 1
    assert out["waived"]["cli_wasm"]["digits_moved"] == 1


def test_a_lost_digit_fails():
    out = pc.compare_pdfs({"cli": facts(["Page 12"]), "wasm": facts(["Page 1"])})
    assert out["mismatches"], "a digit count drop must not pass as a moved digit"


# --- F12: the PUA class and the emoji count ------------------------------------


def test_a_pua_stand_in_passes_as_a_waiver():
    # The native side paints the Wingdings bullet code U+F0A7 (the real face
    # has the glyph), the wasm side its stand-in ▪ (symbol_stand_in,
    # src/convert/font.rs:1120-1141): one PUA class, paired one-for-one.
    out = pc.compare_pdfs({"cli": facts(["\uf0a7 one"]), "wasm": facts(["▪ one"])})
    assert out["same_characters"]["cli_wasm"]
    assert not out["mismatches"]
    assert out["waived"]["cli_wasm"]["pua_standins"] == 1


def test_a_lost_pua_mark_fails():
    # The 03398c02 bug class: wasm painted nothing where native painted the
    # bullet — a count drop, not a substitution.
    out = pc.compare_pdfs({"cli": facts(["\uf0b7 one"]), "wasm": facts(["one"])})
    assert out["mismatches"]


def test_a_pua_may_pair_with_an_emoji_stand_in():
    # Symbol's card-suit codes stand in as emoji (symbol_unicode,
    # src/convert/font.rs:35) — the emoji exclusion must not turn that into
    # a false mismatch.
    out = pc.compare_pdfs({"cli": facts(["\uf0a7 two"]), "wasm": facts(["\u2663 two"])})
    assert out["same_characters"]["cli_wasm"]
    assert not out["mismatches"]


def test_an_arrow_stand_in_greater_than_sign_passes():
    # Wingdings 0xD8's third stand-in is '>' (src/convert/font.rs:186).
    out = pc.compare_pdfs({"cli": facts(["\uf0d8 go"]), "wasm": facts(["> go"])})
    assert out["same_characters"]["cli_wasm"]


def test_unpainted_emoji_counted_and_waived():
    out = pc.compare_pdfs({"cli": facts(["ok \U0001f600"]), "wasm": facts(["ok"])})
    assert out["same_characters"]["cli_wasm"]
    assert out["waived"]["cli_wasm"]["emoji_unpainted"] == 1
    assert out["wasm_unpainted_emoji"]["cli_wasm"] == 1


def test_missing_emoji_totals():
    rows = [{"row": 1, "pdf": {"wasm_unpainted_emoji": {"cli_wasm": 2, "python_wasm": 2}}},
            {"row": 2, "pdf": {"wasm_unpainted_emoji": {"cli_wasm": 1}}},
            {"row": 3, "pdf": {}}]
    assert pc.missing_emoji_per_row(rows) == {1: 2, 2: 1}  # max per row, not the sum
    assert pc.missing_emoji_total(rows) == 3


def test_native_pairs_stay_strict_text():
    # No wasm in the pair → no tolerance at all (v1 behaviour, kept).
    out = pc.compare_pdfs({"cli": facts(["• one"]), "python": facts(["▪ one"])})
    assert out["mismatches"]
    assert not out["same_characters"]


def test_page_count_is_strict_for_wasm_pairs():
    out = pc.compare_pdfs({"cli": facts(["a"], page_count=1), "wasm": facts(["a"], page_count=2)})
    assert out["mismatches"]


# --- F11: the binding rules -----------------------------------------------------


def test_duplicate_bindings_rejected():
    with pytest.raises(ValueError):
        pc.parse_bindings("cli,cli")
    with pytest.raises(ValueError):
        pc.parse_bindings("cli,python,cli")
    with pytest.raises(ValueError):
        pc.parse_bindings("cli")          # fewer than two
    with pytest.raises(ValueError):
        pc.parse_bindings("cli,wasmx")    # unknown binding
    assert pc.parse_bindings("cli, python ,wasm") == ["cli", "python", "wasm"]


def test_single_pdf_producer_fails_when_two_bindings_have_the_operation():
    slim = {"wasm": "unavailable: slim build exports no docxToPdf"}
    assert pc.pdf_coverage_failure(["cli", "python", "wasm"], slim, ["cli"])
    assert pc.pdf_coverage_failure(["cli", "python"], {}, ["python"])
    assert pc.pdf_coverage_failure(["cli", "python", "wasm"], slim, []) is not None
    # two producers: no failure
    assert pc.pdf_coverage_failure(["cli", "python", "wasm"], slim, ["cli", "python"]) is None
    # only one binding has the operation at all: a single producer is honest
    assert pc.pdf_coverage_failure(["cli", "wasm"], slim, ["cli"]) is None


# --- the inputs (#1757) ---------------------------------------------------------


def test_a_release_sample_names_its_sources_relative_to_the_bench_root(tmp_path):
    bench = tmp_path / "bench"
    for rel in ("corpus/a.docx", "corpus/b.docx", "corpus/c.docx"):
        (bench / rel).parent.mkdir(parents=True, exist_ok=True)
        (bench / rel).write_text(rel)
    sample = tmp_path / "sample_redline_9.9.9_x.csv"
    sample.write_text("key,base,base_sha256,next,next_sha256\r\n"
                      "k1,corpus/a.docx,0,corpus/b.docx,0\r\nk2,corpus/b.docx,0,corpus/c.docx,0\r\n")
    pairs = pc.load_pairs(sample, bench / "unused", 1, bench)
    assert [(p["key"], p["base"], p["next"]) for p in pairs] == [
        ("k1", bench / "corpus/a.docx", bench / "corpus/b.docx"),
        ("k2", bench / "corpus/b.docx", bench / "corpus/c.docx")]


def test_a_bench_pairs_csv_names_stems_under_the_corpus(tmp_path):
    corpus = tmp_path / "corpus"
    corpus.mkdir()
    for stem in ("a", "b"):
        (corpus / f"{stem}.docx").write_text(stem)
    pairs_csv = tmp_path / "pairs.csv"
    pairs_csv.write_text("key,base,next\nk1,a,b\n")
    assert pc.load_pairs(pairs_csv, corpus, 1, tmp_path)[0]["base"] == corpus / "a.docx"


def test_a_missing_source_stops_the_run(tmp_path):
    pairs_csv = tmp_path / "pairs.csv"
    pairs_csv.write_text("key,base,next\nk1,corpus/a.docx,corpus/b.docx\n")
    with pytest.raises(SystemExit):
        pc.load_pairs(pairs_csv, tmp_path, 1, tmp_path)


def test_the_cli_is_given_the_timestamp_flag_it_has():
    # 0.13.0 removed --date: the recovered tool passed it to every redline
    text = (HERE / "parity_check.py").read_text()
    assert '"-d", args.date' in text
    assert '"--date", args.date' not in text


# --- tools/parity/build.sh (#1757) ------------------------------------------------


BUILD = HERE / "build.sh"
# stub uv and wasm-pack: log their argv, and wasm-pack writes a package to --out-dir
STUBS = {
    "uv": '#!/bin/sh\necho "uv $*" >> "$STUB_LOG"\n',
    "wasm-pack": '#!/bin/sh\necho "wasm-pack $*" >> "$STUB_LOG"\nwhile [ $# -gt 0 ]; do\n'
                 '  [ "$1" = --out-dir ] && { mkdir -p "$2"; echo x > "$2/jubarte_wasm_bg.wasm"; }\n'
                 '  shift\ndone\n',
}


def engine_copy(tmp_path):
    """A git checkout holding build.sh and the three source folders, committed."""
    root = tmp_path / "engine"
    for rel in ("src/lib.rs", "jubarte-python/src/lib.rs", "jubarte-wasm/src/lib.rs"):
        (root / rel).parent.mkdir(parents=True, exist_ok=True)
        (root / rel).write_text("// x\n")
    (root / "tools/parity").mkdir(parents=True)
    shutil.copy(BUILD, root / "tools/parity/build.sh")
    git = ["git", "-C", str(root), "-c", "user.name=t", "-c", "user.email=t@t", "-c", "commit.gpgsign=false"]
    subprocess.run(["git", "init", "-q", str(root)], check=True)
    subprocess.run([*git, "add", "-A"], check=True)
    subprocess.run([*git, "commit", "-qm", "x"], check=True)
    head = subprocess.run([*git, "rev-parse", "HEAD"], check=True, capture_output=True, text=True).stdout.strip()
    bin_dir = tmp_path / "bin"
    bin_dir.mkdir()
    for name, body in STUBS.items():
        (bin_dir / name).write_text(body)
        (bin_dir / name).chmod(0o755)
    return root, head, bin_dir


def run_build(root, bin_dir, tmp_path, *args):
    env = {**os.environ, "PATH": f"{bin_dir}:{os.environ['PATH']}", "STUB_LOG": str(tmp_path / "stub.log")}
    return subprocess.run(["bash", str(root / "tools/parity/build.sh"), *args],
                          capture_output=True, text=True, env=env, timeout=60)


def test_build_stamps_the_wasm_package_with_head(tmp_path):
    root, head, bin_dir = engine_copy(tmp_path)
    out = tmp_path / "out"
    r = run_build(root, bin_dir, tmp_path, str(out))
    assert r.returncode == 0, r.stdout + r.stderr
    assert (out / "wasm-pkg/ENGINE_COMMIT.txt").read_text().strip() == head
    log = (tmp_path / "stub.log").read_text()
    assert "maturin develop --release" in log
    assert "--target nodejs" in log and f"--out-dir {out / 'wasm-pkg'}" in log


def test_build_refuses_uncommitted_engine_sources(tmp_path):
    # the stamp would name a commit the build does not embody
    root, _, bin_dir = engine_copy(tmp_path)
    (root / "jubarte-wasm/src/lib.rs").write_text("// changed\n")
    r = run_build(root, bin_dir, tmp_path, str(tmp_path / "out"))
    assert r.returncode == 1, r.stdout
    assert "jubarte-wasm/src/lib.rs" in r.stderr
    assert not (tmp_path / "stub.log").exists(), "nothing builds before the tree is checked"


def test_build_never_leaves_a_previous_stamp(tmp_path):
    root, head, bin_dir = engine_copy(tmp_path)
    old = tmp_path / "out/wasm-pkg"
    old.mkdir(parents=True)
    (old / "ENGINE_COMMIT.txt").write_text("0" * 40 + "\n")
    (old / "stale.js").write_text("old")
    r = run_build(root, bin_dir, tmp_path, str(tmp_path / "out"))
    assert r.returncode == 0, r.stderr
    assert not (old / "stale.js").exists()
    assert (old / "ENGINE_COMMIT.txt").read_text().strip() == head


def test_build_wants_its_out_dir():
    r = subprocess.run(["bash", str(BUILD)], capture_output=True, text=True, timeout=30)
    assert r.returncode == 2
    assert "usage" in r.stderr


def test_the_wasm_driver_loads_a_module_named_relative_to_its_cwd(tmp_path):
    # release.sh names the package relative to the engine root (#1799); a bare
    # relative path reached require() as a package name and failed
    pkg = tmp_path / "parity-work/wasm-pkg"
    pkg.mkdir(parents=True)
    (pkg / "jubarte_wasm.js").write_text("module.exports = { compareDocuments() {} };\n")
    (pkg / "package.json").write_text('{"name": "jubarte-wasm", "version": "0.0.0"}\n')
    (tmp_path / "jobs.json").write_text('{"jobs": []}')
    r = subprocess.run(["node", str(HERE / "parity_wasm.mjs"), "--module", "parity-work/wasm-pkg",
                        "--jobs", "jobs.json", "--out", "out.json"],
                       capture_output=True, text=True, cwd=tmp_path, timeout=60)
    assert r.returncode == 0, r.stderr
    out = json.loads((tmp_path / "out.json").read_text())
    assert out["module"] == str(pkg / "jubarte_wasm.js")
    assert out["packageVersion"] == "0.0.0"


def test_release_sh_runs_these_tests_and_names_the_tools():
    text = (HERE.parent.parent / "scripts/release.sh").read_text()
    assert "pytest -q -p no:cacheprovider tools/parity" in text, "step 5 runs these tests"
    items = text[text.index("checklist_items() {"):]
    items = items[:items.index("\n}\n")]
    assert "tools/parity/build.sh" in items and "tools/parity/parity_check.py" in items
    assert "parity_build_v2" not in items

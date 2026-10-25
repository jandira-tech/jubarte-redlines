#!/usr/bin/env -S uv run --script
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only
"""Parity check v2: the Rust CLI, the Python extension and the WASM build of
jubarte must produce the SAME redline (and the same PDF of it) for the same
inputs — from builds whose identity is recorded and, when --expect-commit /
--engine-dir are given, proven fresh. Repairs the 0.11.2 release audit's
findings F10, F11, F12; the v1 phases below are unchanged except where
noted.

    uv run tools/parity/parity_check.py --cli <jubarte-binary> \
        [--py-python <venv-python-with-jubarte_redlines>] \
        [--wasm <jubarte-wasm dir or jubarte_wasm.js>] \
        [--expect-commit <engine-HEAD-sha>] [--engine-dir <engine-checkout>] \
        [--max-wasm-missing-emoji N] \
        --pairs release_info/sample_redline_<version>_<stamp>.csv \
        [--corpus <dir>] [--step 40] [--out-dir DIR]

Committed under tools/parity/ for 0.14.0 (#1757): before that it lived only
in a release work folder, which was deleted once and rebuilt from session
logs. tools/parity/build.sh builds the Python and WASM bindings it compares.

Build identity (F10). The report records, and the stdout block prints, a
build identity for every binding: the CLI binary's path, sha256 and mtime;
the Python extension module's path, sha256 and mtime; the WASM .wasm file's
path, sha256 and mtime plus, when an ENGINE_COMMIT.txt sits beside the
package, its content. With --expect-commit SHA the run FAILS (exit 2) unless
the identities agree: the wasm package's ENGINE_COMMIT.txt must exist and
equal SHA — a bare wasm-pack pkg/ has no identity and no longer passes (the
0.11.2 report rode one: package 0.10.0, engine_commit null). Build the wasm
side with tools/parity/build.sh (stamps ENGINE_COMMIT.txt) or jubarte-wasm's
own build-npm.sh (jubarte-wasm/npm carries it). With --engine-dir DIR a
binding whose artifact is older than the newest commit time of the engine
sources (git -C DIR log -1 --format=%ct -- src jubarte-python/src
jubarte-wasm/src) fails as STALE — a rebuilt-nothing rerun can no longer
compare yesterday's artifacts against today's engine.

Then the v1 phases, with the vacuous-pass holes closed (F11): duplicate
bindings are rejected up front; a pair where fewer than two bindings
produced a PDF, when at least two bindings have the operation, FAILS instead
of passing as "nothing to compare".

1. Redline with all three bindings, options pinned identically:
   - CLI:   jubarte BASE NEXT -o OUT --force --quiet --author Redline \
            -d 1970-01-01T00:00:00Z  (0.13.0 removed --date)
     (the bench's invocation plus the author/date the CLI already defaults
     to: author "Redline", src/bin/jubarte.rs:87; date DEFAULT_DATE,
     src/bin/jubarte.rs:92-96).
   - Python: jubarte_redlines.compare_documents(base, nxt, author="Redline",
            date="1970-01-01T00:00:00Z")  (jubarte-python/src/lib.rs:66-81;
            its own defaults differ — author "jubarte" — hence the pin).
   - WASM:  compareDocuments(base, nxt, "Redline")  (jubarte-wasm/src/lib.rs:86-99).
            The WASM binding takes no date: it uses WmlComparerSettings::default(),
            whose date_time_for_revisions is DEFAULT_DATE (src/comparer/mod.rs:1381-1384),
            i.e. exactly the epoch pinned on the other two bindings.
2. Compare the .docx packages: identical zip entry list (same order) and
   every part byte-identical. NO normalisation is applied — none is legitimate
   (src/opc/mod.rs:297-306, asserted by entries_carry_no_wall_clock_time,
   src/opc/mod.rs:624-640). Any byte difference is a real parity failure.
3. Convert EACH binding's own redline to PDF with that binding, Word revision
   style. Compare page count and per-page extracted text with PyMuPDF,
   strictly. PDF BYTES are compared informationally only (wasm embeds
   Carlito/Liberation and ignores system font overrides —
   jubarte-wasm/src/lib.rs:184-191; the PDF carries no wall clock,
   src/convert/pdf.rs:1038, no /ID, src/convert/pdf.rs:1994).

WASM text tolerance (F12) — kept, tightened and exposed. The native side
paints a Symbol/Wingdings character as its private-use code (the real face
has the glyph), the WASM side as a stand-in Unicode character
(symbol_stand_in, src/convert/font.rs:1120-1141; the stand-in tables are
symbol_unicode, src/convert/font.rs:35, and wingdings_unicode,
src/convert/font.rs:175-201). Symbols therefore fold PER CLASS, grounded in
those tables:
  - bullet-geo  bullets and geometric shapes: · • ▪ ■ □ ● ○ ◆ ❖
                (Wingdings 0x6C..0x77, 0x9E..0xA8 → font.rs:176-185)
  - arrow       ← → ↑ ↓ ↔ plus ➢ and the '>' of Wingdings 0xD8 (font.rs:186)
  - tick-cross  ✓ ✗ and their U+00D7 / √ fallbacks (Wingdings 0xFB/0xFC,
                font.rs:191-192)
  - pua         U+E000..U+F8FF — ONE class: which stand-in class a
                private-use code belongs to is NOT knowable (the face reads
                it by whichever table it honours, font.rs:1131-1141), so a
                PUA difference may pair one-for-one with a folded symbol or
                an emoji-only stand-in on the other side — never with text.
  An arrow replaced by a bullet, or a bullet by a tick, no longer folds away.
Emoji stay excluded from the multiset (no embedded face paints them) but the
unpainted count is printed in the waived column, and --max-wasm-missing-emoji
N (default: no limit) fails the run when the total exceeds N. The digit
tolerance stays whole-document (a PAGE field reflowing between pages) but is
reported per pair. A 'waived' column makes everything tolerated legible.

A binding that lacks an operation is reported as such (e.g. a slim WASM build
has no docxToPdf — jubarte-wasm/npm/README.md "Slim builds" table) and the
remaining bindings are compared — but never fewer than two producers when two
bindings have the operation (F11).

Output: a per-pair table on stdout and a JSON report (--report); exit 1 on
any mismatch (including a vacuous single-producer pair and an over-limit
emoji count), 2 on setup problems — including a build identity that cannot
be proven (missing ENGINE_COMMIT.txt, mismatched commit, STALE artifact) —
0 when everything agrees.
"""

# /// script
# requires-python = ">=3.10"
# dependencies = ["pymupdf>=1.24"]
# ///
from __future__ import annotations

import argparse
from collections import Counter
import csv
import hashlib
import io
import json
import os
import re
import subprocess
import sys
import time
import unicodedata
import zipfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
# the bench checkout beside this engine checkout ($NEUROTIC_DOCX_BENCH wins)
DEFAULT_BENCH_ROOT = HERE.parent.parent.parent / "neurotic_docx_bench"
DEFAULT_CORPUS_REL = Path("corpus/word")
# The CLI's own defaults (src/bin/jubarte.rs:87, :92-96); pinned explicitly so
# the Python/WASM calls provably match. DEFAULT_DATE: src/document_comparer.rs:261.
DEFAULT_AUTHOR = "Redline"
DEFAULT_DATE = "1970-01-01T00:00:00Z"

# Runs under the interpreter that has jubarte_redlines importable (either
# in-process, or the --py-python venv built with maturin). Uses only the
# library surface re-exported by jubarte-python/python/jubarte_redlines/__init__.py.
# v2 (F10): also reports the native extension module's file path so the
# report can record the Python binding's build identity (path/sha256/mtime).
PY_DRIVER = r"""
import json, sys
from pathlib import Path

job = json.loads(Path(sys.argv[1]).read_text())
from jubarte_redlines import __version__, compare_documents, docx_to_pdf

results = {"version": __version__, "jobs": []}
try:
    from jubarte_redlines import _native
    results["native_module"] = getattr(_native, "__file__", None)
except Exception:
    results["native_module"] = None
for j in job["jobs"]:
    r = {"id": j["id"], "ok": True}
    try:
        if j["kind"] == "compare":
            out = compare_documents(
                Path(j["base"]).read_bytes(),
                Path(j["next"]).read_bytes(),
                author=job["author"],
                date=job["date"],
            )
        elif j["kind"] == "topdf":
            out = docx_to_pdf(
                Path(j["docx"]).read_bytes(),
                compress=j.get("compress", False),
                revisions=j.get("revisions", "conventional"),
            )
        else:
            raise ValueError(f"unknown kind: {j['kind']}")
        Path(j["out"]).write_bytes(out)
        r["bytes"] = len(out)
    except Exception as e:  # JubarteError and anything else: data, not a crash
        r["ok"] = False
        r["error"] = f"{type(e).__name__}: {e}".splitlines()[0]
    results["jobs"].append(r)
Path(sys.argv[3]).write_text(json.dumps(results, indent=1))
"""


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def first_line(text: str) -> str:
    return text.strip().splitlines()[0] if text.strip() else ""


# Each binding wraps engine refusals differently (CLI "error: …",
# Python "JubarteError: …", wasm "jubarte-wasm: …"), and the Python layer
# fronts the admission code via admission::code_first (jubarte-python/src/lib.rs:34-36)
# while wasm prefixes the engine message (jubarte-wasm/src/lib.rs:57-59). For the
# "every binding refused this pair" case we compare the refusal CODE
# (e.g. LEGACY_DOC, INPUT_LIMIT), not the envelope — behaviour parity, not
# string parity. Output bytes themselves are never normalised.
_ERR_ENVELOPE = re.compile(
    r"(?i)^(?:error|jubarte(?:-wasm|_redlines)?|jubarteerror)\s*:\s*")
_ERR_CODE = re.compile(r"^[A-Z][A-Z0-9]*(?:_[A-Z0-9]+)+\b")


def err_key(msg: str) -> str:
    m = _ERR_ENVELOPE.sub("", msg.strip()).strip()
    code = _ERR_CODE.match(m)
    return code.group(0) if code else m


# --- symbol fold classes (F12) --------------------------------------------------
#
# Fold tokens are sentinels from Supplementary Private Use Area-B (U+F0000..):
# NFC-normalised engine text is BMP-only, so a token can never collide with a
# real painted character. Classes are grounded in the engine's stand-in
# tables — wingdings_unicode (src/convert/font.rs:175-201) and symbol_unicode
# (src/convert/font.rs:35) — not invented here.

_TOK_BULLET_GEO = "\U000F0000"   # bullets and geometric shapes
_TOK_ARROW = "\U000F0001"        # arrows
_TOK_TICK_CROSS = "\U000F0002"   # ticks and crosses
_TOK_PUA = "\U000F0003"          # private use, one class (see docstring)

SYMBOL_CLASSES: dict[str, str] = {
    "bullet-geo": _TOK_BULLET_GEO,
    "arrow": _TOK_ARROW,
    "tick-cross": _TOK_TICK_CROSS,
    "pua": _TOK_PUA,
}
_CLASS_OF_TOKEN = {v: k for k, v in SYMBOL_CLASSES.items()}
_CLASS_SHORT = {"bullet-geo": "geo", "arrow": "arrow", "tick-cross": "tick",
                "pua": "pua"}


def _symbol_class(ch: str) -> str | None:
    """The fold class of `ch`, or None when `ch` is a real character that
    must not fold (v1 folded every symbol to one token; v2 keeps classes
    apart so • ↔ → no longer passes)."""
    o = ord(ch)
    if 0xE000 <= o <= 0xF8FF:
        return "pua"
    if o in (0xB7, 0x2022, 0x2023, 0x2043) or 0x25A0 <= o <= 0x25FF or o == 0x2756:
        return "bullet-geo"     # · • ▪ ■ □ ● ○ ◆ ❖ (font.rs:176-185)
    if 0x2190 <= o <= 0x21FF or o in (0x3E, 0x27A2):
        return "arrow"          # ← → ↑ ↓ ↔ ➢ and Wingdings 0xD8's '>' (font.rs:186)
    if 0x2713 <= o <= 0x2718 or o in (0xD7, 0x221A):
        return "tick-cross"     # ✓ ✗ and the U+00D7 / √ fallbacks (font.rs:191-192)
    return None


# Dingbats (U+2700..U+27BF) that carry Unicode's Emoji property (emoji-data.txt).
_DINGBAT_EMOJI = frozenset(
    [0x2702, 0x2705, 0x270F, 0x2712, 0x2714, 0x2716, 0x271D, 0x2721, 0x2728, 0x2733, 0x2734,
     0x2744, 0x2747, 0x274C, 0x274E, 0x2757, 0x2763, 0x2764, 0x27A1, 0x27B0, 0x27BF,
     *range(0x2708, 0x270E), *range(0x2753, 0x2756), *range(0x2795, 0x2798)])


def _is_emoji(ch: str) -> bool:
    o = ord(ch)
    return (0x1F000 <= o <= 0x1FAFF or 0x2600 <= o <= 0x26FF or o in (0xFE0F, 0x200D)
            or o in _DINGBAT_EMOJI)


def painted_characters(pages_text: list[str], drop_emoji: bool = True,
                       fold: bool = True) -> Counter:
    """The characters a PDF paints, whitespace aside. With fold=True every
    symbol-class character counts as its class token (F12: per class, not the
    one token of v1); with fold=False characters stay themselves (used to
    report what a fold excused). Canonically equal characters are one
    character (NFC: a face may draw U+037E with its semicolon glyph, which
    reads back as U+003B)."""
    out: Counter = Counter()
    for text in pages_text:
        for ch in unicodedata.normalize("NFC", text):
            if ch.isspace() or (drop_emoji and _is_emoji(ch)):
                continue
            if fold:
                cls = _symbol_class(ch)
                if cls:
                    out[SYMBOL_CLASSES[cls]] += 1
                else:
                    out[ch] += 1
            else:
                out[ch] += 1
    return out


def emoji_counter(pages_text: list[str]) -> Counter:
    """The emoji each side paints (they are excluded from the multiset; the
    WASM side paints none — no embedded face has them — and the gap is
    reported, optionally gated by --max-wasm-missing-emoji)."""
    out: Counter = Counter()
    for text in pages_text:
        for ch in unicodedata.normalize("NFC", text):
            if _is_emoji(ch):
                out[ch] += 1
    return out


def _fold_counter(unfolded: Counter) -> Counter:
    out: Counter = Counter()
    for ch, n in unfolded.items():
        cls = _symbol_class(ch)
        out[SYMBOL_CLASSES[cls] if cls else ch] += n
    return out


def _folded_diff_counts(ua: Counter, ub: Counter) -> dict[str, int]:
    """Per class, how many folded characters actually differed between the
    sides — the honest content of the 'waived' column."""
    diff = (ua - ub) + (ub - ua)
    out: dict[str, int] = {}
    for ch, n in diff.items():
        cls = _symbol_class(ch)
        if cls:
            out[cls] = out.get(cls, 0) + n
    return out


def _show_tokens(c: Counter) -> str:
    names = {v: f"<{k}>" for k, v in SYMBOL_CLASSES.items()}
    return " ".join(f"{names.get(ch, repr(ch))}x{n}" for ch, n in c.most_common(8))


def compare_painted_characters(native_text: list[str], wasm_text: list[str]) -> dict:
    """The F12 wasm tolerance, in one pure function.

    The two multisets must agree after per-class symbol folding, with three
    explicit, reported waivers:
      - digits_moved:   digit VALUES ignored when the total digit count agrees
                        (a PAGE field the other fonts wrap onto the next page;
                        whole-document, reported per pair);
      - pua_standins:   a private-use difference (the native side paints the
                        Symbol/Wingdings code, the wasm side its stand-in,
                        font.rs:1120-1141) may pair one-for-one with a folded
                        symbol or an emoji-only stand-in on the other side —
                        which class a PUA stands for is not knowable;
      - emoji_unpainted: emoji are excluded from the multiset; the count the
                        wasm side leaves unpainted is reported (and can be
                        failed by --max-wasm-missing-emoji in main()).
    Everything else — a lost bullet, a lost digit, an arrow replaced by a
    bullet — is a mismatch.
    """
    ua = painted_characters(native_text, drop_emoji=True, fold=False)
    ub = painted_characters(wasm_text, drop_emoji=True, fold=False)
    ea, eb = emoji_counter(native_text), emoji_counter(wasm_text)
    ca, cb = _fold_counter(ua), _fold_counter(ub)
    only_a, only_b = ca - cb, cb - ca
    waived: dict = {}
    net_emoji = sum(ea.values()) - sum(eb.values())
    if net_emoji > 0:
        waived["emoji_unpainted"] = net_emoji
    folded = _folded_diff_counts(ua, ub)
    if folded:
        waived["symbols_folded"] = folded
    if not only_a and not only_b:
        return {"ok": True, "waived": waived, "only_native": Counter(), "only_wasm": Counter()}
    moved_digits = (all(ch.isdigit() for ch in (*only_a, *only_b))
                    and sum(only_a.values()) == sum(only_b.values()))
    if moved_digits:
        waived["digits_moved"] = sum(only_a.values())
        return {"ok": True, "waived": waived, "only_native": only_a, "only_wasm": only_b}

    def sym_pua(diff: Counter) -> tuple[int, int]:
        sym = sum(n for t, n in diff.items() if t != _TOK_PUA and t in SYMBOL_CLASSES.values())
        return sym, diff.get(_TOK_PUA, 0)

    sym_a, pua_a = sym_pua(only_a)
    sym_b, pua_b = sym_pua(only_b)
    extra_emoji_b = sum(max(0, eb[c] - ea.get(c, 0)) for c in eb)
    extra_emoji_a = sum(max(0, ea[c] - eb.get(c, 0)) for c in ea)
    all_symbolic = all(t in SYMBOL_CLASSES.values() for t in (*only_a, *only_b))
    pua_explains = (all_symbolic and (pua_a or pua_b)
                    and sym_a <= pua_b and sym_b <= pua_a
                    and pua_a <= sym_b + extra_emoji_b
                    and pua_b <= sym_a + extra_emoji_a)
    if pua_explains:
        waived["pua_standins"] = pua_a + pua_b
        return {"ok": True, "waived": waived, "only_native": only_a, "only_wasm": only_b}
    return {"ok": False, "waived": waived, "only_native": only_a, "only_wasm": only_b}


# --- bindings (F11) ------------------------------------------------------------


def parse_bindings(spec: str) -> list[str]:
    """Split --bindings, rejecting duplicates (v1 accepted `cli,cli`, compared
    cli.docx with itself and PASSed the PDF phase vacuously — F11)."""
    bindings = [b.strip() for b in spec.split(",") if b.strip()]
    valid = ("cli", "python", "wasm")
    unknown = sorted({b for b in bindings if b not in valid})
    if unknown:
        raise ValueError(f"unknown binding(s) {','.join(unknown)} — valid: cli,python,wasm")
    if len(bindings) < 2:
        raise ValueError("at least two bindings are required (cli,python,wasm)")
    dupes = sorted({b for b in bindings if bindings.count(b) > 1})
    if dupes:
        raise ValueError(f"duplicate binding(s) {','.join(dupes)} — every binding runs once")
    return bindings


def pdf_coverage_failure(bindings: list[str], pdf_errors: dict, producers: list[str]) -> str | None:
    """F11: a pair where fewer than two bindings produced a PDF, when at
    least two bindings have the operation, is a FAILURE — not a vacuous pass.
    'unavailable:' marks a build that lacks the operation at all (a slim wasm
    without docxToPdf); every other binding has it."""
    havers = [b for b in bindings if not str(pdf_errors.get(b, "")).startswith("unavailable:")]
    if len(havers) >= 2 and len(producers) < 2:
        produced = ", ".join(sorted(producers)) or "none"
        return (f"only {len(producers)} of {len(havers)} PDF-capable bindings produced a PDF "
                f"({produced}) — vacuous pass is a failure (F11)")
    return None


def missing_emoji_per_row(pairs: list[dict]) -> dict[int, int]:
    """Per pair, the wasm-unpainted emoji count (max over its wasm pairs —
    cli_wasm and python_wasm measure the same wasm gap)."""
    out: dict[int, int] = {}
    for p in pairs:
        counts = (p.get("pdf") or {}).get("wasm_unpainted_emoji") or {}
        if counts:
            out[p["row"]] = max(counts.values())
    return out


def missing_emoji_total(pairs: list[dict]) -> int:
    return sum(missing_emoji_per_row(pairs).values())


# --- inputs -------------------------------------------------------------------


def source(value: str, corpus: Path, bench_root: Path) -> Path:
    """A release_info sample names a .docx relative to the bench root; a bench
    pairs CSV names a stem under the corpus."""
    return bench_root / value if value.endswith(".docx") else corpus / f"{value}.docx"


def load_pairs(pairs_csv: Path, corpus: Path, step: int, bench_root: Path = DEFAULT_BENCH_ROOT) -> list[dict]:
    """Every `step`-th data row (1-based) of the pairs CSV: a release_info
    redline sample or a bench pairs CSV (key, base, next either way)."""
    with pairs_csv.open(newline="") as fh:
        rows = [r for r in csv.DictReader(fh) if r.get("base") and r.get("next")]
    pairs, missing = [], []
    for i, r in enumerate(rows):
        row_no = i + 1
        if row_no % step != 0:
            continue
        base, nxt = source(r["base"], corpus, bench_root), source(r["next"], corpus, bench_root)
        if not base.is_file():
            missing.append((row_no, base))
        elif not nxt.is_file():
            missing.append((row_no, nxt))
        else:
            pairs.append({"row": row_no, "key": r["key"], "base": base, "next": nxt})
    for row_no, path in missing:
        print(f"error: corpus file for CSV row {row_no} missing: {path}", file=sys.stderr)
    if missing:
        raise SystemExit(2)
    return pairs


# --- binding drivers ----------------------------------------------------------


def run_cli(argv: list[str], timeout: float) -> dict:
    try:
        proc = subprocess.run(argv, capture_output=True, text=True, timeout=timeout)
    except subprocess.TimeoutExpired:
        return {"ok": False, "error": f"timeout after {timeout:.0f}s"}
    if proc.returncode != 0:
        return {"ok": False, "error": first_line(proc.stderr) or f"exit {proc.returncode}",
                "exit": proc.returncode}
    return {"ok": True, "stdout": proc.stdout}


def cli_version(cli: Path, timeout: float) -> str:
    r = run_cli([str(cli), "--version"], timeout)
    return first_line(r.get("stdout", "")) if r["ok"] else r.get("error", "?")


def run_python_jobs(jobs: dict, jobs_path: Path, results_path: Path,
                    py_python: Path | None, timeout: float) -> dict:
    """Run the compare/topdf jobs under jubarte_redlines. Returns the driver's
    results JSON, or {"error": ...} when the binding cannot run at all."""
    jobs_path.write_text(json.dumps(jobs))
    budget = max(timeout, timeout * len(jobs["jobs"]))
    if py_python is not None:
        try:
            proc = subprocess.run(
                [str(py_python), "-c", PY_DRIVER, str(jobs_path), "", str(results_path)],
                capture_output=True, text=True, timeout=budget,
            )
        except subprocess.TimeoutExpired:
            return {"error": f"python driver timeout after {budget:.0f}s"}
        if proc.returncode != 0:
            return {"error": first_line(proc.stderr) or f"python driver exit {proc.returncode}"}
    else:  # in-process: the invoking interpreter must already import it
        old_argv = sys.argv
        try:
            sys.argv = ["py-driver", str(jobs_path), "", str(results_path)]
            exec(PY_DRIVER, {"__name__": "__main__"})  # our own source
        except SystemExit:
            raise
        except BaseException as e:  # import failure included
            return {"error": f"{type(e).__name__}: {e}".splitlines()[0]}
        finally:
            sys.argv = old_argv
    return json.loads(results_path.read_text())


def run_wasm_jobs(jobs: dict, jobs_path: Path, results_path: Path,
                  wasm_module: str, timeout: float) -> dict:
    jobs_path.write_text(json.dumps(jobs))
    argv = ["node", str(HERE / "parity_wasm.mjs"),
            "--module", wasm_module, "--jobs", str(jobs_path), "--out", str(results_path)]
    budget = max(timeout, timeout * len(jobs["jobs"]))
    try:
        proc = subprocess.run(argv, capture_output=True, text=True, timeout=budget)
    except subprocess.TimeoutExpired:
        return {"error": f"wasm driver timeout after {budget:.0f}s"}
    if proc.returncode != 0:
        return {"error": first_line(proc.stderr) or f"wasm driver exit {proc.returncode}"}
    return json.loads(results_path.read_text())


def job_result(results: dict, job_id: str) -> dict | None:
    for j in results.get("jobs", []):
        if j["id"] == job_id:
            return j
    return None


# --- build identity (F10) -------------------------------------------------------


def file_identity(path: Path) -> dict:
    st = path.stat()
    return {"path": str(path), "sha256": sha256(path.read_bytes()),
            "mtime": int(st.st_mtime),
            "mtime_iso": time.strftime("%Y-%m-%dT%H:%M:%S", time.localtime(st.st_mtime))}


def engine_src_commit_ts(engine_dir: Path) -> dict:
    """The newest commit time of the engine sources — the STALE rule's clock
    (exactly the audit's command: git log -1 --format=%ct -- src
    jubarte-python/src jubarte-wasm/src)."""
    proc = subprocess.run(
        ["git", "-C", str(engine_dir), "log", "-1", "--format=%ct", "--",
         "src", "jubarte-python/src", "jubarte-wasm/src"],
        capture_output=True, text=True, timeout=60)
    if proc.returncode != 0 or not proc.stdout.strip().isdigit():
        raise RuntimeError(f"git log failed in {engine_dir}: "
                           f"{first_line(proc.stderr) or proc.returncode}")
    ts = int(proc.stdout.strip())
    return {"ts": ts, "iso": time.strftime("%Y-%m-%dT%H:%M:%S", time.localtime(ts))}


def print_identity(identity: dict) -> None:
    exp = identity.get("expect_commit")
    print("build identity (F10):")
    print(f"  expect-commit: {exp or '(not pinned)'}")
    src = identity.get("engine_src_newest_commit")
    if src:
        print(f"  engine src newest commit: {src['iso']} ({src['ts']})")
    for b in ("cli", "python", "wasm"):
        ident = identity.get(b)
        if not ident:
            continue
        stale = " STALE" if ident.get("stale") else ""
        print(f"  {b:<8} {ident['path']}")
        line = f"          sha256={ident['sha256'][:16]}… mtime={ident['mtime_iso']}{stale}"
        if b == "wasm":
            ec = ident.get("engine_commit")
            note = ""
            if exp:
                note = " (matches expect)" if ident.get("engine_commit_matches_expect") \
                    else " (MISMATCH)"
            line += f" engine_commit={ec or 'none'}{note}"
        print(line)


# --- comparisons --------------------------------------------------------------


def zip_parts(data: bytes) -> tuple[list[str], dict[str, bytes]]:
    with zipfile.ZipFile(io.BytesIO(data)) as zf:
        names = [i.filename for i in zf.infolist()]
        return names, {n: zf.read(n) for n in names}


def compare_docx(a: bytes, b: bytes) -> dict:
    """Ordered entry list + every part byte-identical. No normalisation."""
    names_a, parts_a = zip_parts(a)
    names_b, parts_b = zip_parts(b)
    problems: list = []
    if names_a != names_b:
        problems.append({
            "entry_list": {
                "only_in_first": [n for n in names_a if n not in names_b][:10],
                "only_in_second": [n for n in names_b if n not in names_a][:10],
                "order_equal": sorted(names_a) == sorted(names_b),
            }
        })
        return {"ok": False, "entries": len(names_a), "problems": problems}
    for name in names_a:
        pa, pb = parts_a[name], parts_b[name]
        if pa != pb:
            first = next((i for i in range(min(len(pa), len(pb))) if pa[i] != pb[i]),
                         min(len(pa), len(pb)))
            problems.append({"part": name, "first_diff_offset": first, "sizes": [len(pa), len(pb)]})
    return {"ok": not problems, "entries": len(names_a), "problems": problems}


def pdf_facts(data: bytes) -> dict:
    import fitz  # PyMuPDF (PEP 723 dependency)

    doc = fitz.open(stream=data, filetype="pdf")
    try:
        return {"page_count": doc.page_count,
                "pages_text": [page.get_text("text") for page in doc]}
    finally:
        doc.close()


def compare_pdfs(facts: dict[str, dict]) -> dict:
    """Page count is strict for every pair.

    Two native bindings (the CLI and the Python extension run the same code
    with the same fonts) must also agree on every page's text, line for line.
    A pair with the WASM build cannot: it has only its embedded Carlito and
    Liberation faces, so lines wrap elsewhere and a symbol is drawn by a
    stand-in. Those pairs must paint the same characters the same number of
    times under the F12 rules (see compare_painted_characters): symbols fold
    per class — bullets/geometric, arrows, ticks/crosses — private-use codes
    keep one class of their own, emoji stay out but are counted and can be
    gated, digits are tolerated whole-document when the counts agree. Every
    tolerance is recorded under 'waived' so a human reads what was excused.
    PDF bytes are informational.

    facts: binding -> {pdf: bytes, sha: hex, page_count: int, pages_text: [str]}
    """
    names = sorted(facts)
    out: dict = {"page_count": {}, "sha256": {}, "byte_identical": {},
                 "text_identical": {}, "same_characters": {}, "waived": {},
                 "wasm_unpainted_emoji": {}, "page_dependent_digits": {},
                 "mismatches": []}
    for n in names:
        out["page_count"][n] = facts[n]["page_count"]
        out["sha256"][n] = facts[n]["sha"]
    for i in range(len(names)):
        for j in range(i + 1, len(names)):
            a, b, pair = names[i], names[j], f"{names[i]}_{names[j]}"
            out["byte_identical"][pair] = facts[a]["sha"] == facts[b]["sha"]
            if facts[a]["page_count"] != facts[b]["page_count"]:
                out["mismatches"].append({"pair": pair,
                                          "page_count": [facts[a]["page_count"], facts[b]["page_count"]]})
                continue
            if facts[a]["pages_text"] == facts[b]["pages_text"]:
                out["text_identical"][pair] = True
                continue
            if "wasm" not in (a, b):
                pages = [k + 1 for k, (x, y) in enumerate(zip(facts[a]["pages_text"], facts[b]["pages_text"]))
                         if x != y]
                out["mismatches"].append({"pair": pair, "text_pages": pages[:10]})
                continue
            native = a if b == "wasm" else b
            verdict = compare_painted_characters(facts[native]["pages_text"],
                                                 facts["wasm"]["pages_text"])
            if verdict["ok"]:
                out["same_characters"][pair] = True
                if verdict["waived"]:
                    out["waived"][pair] = verdict["waived"]
                    if verdict["waived"].get("emoji_unpainted"):
                        out["wasm_unpainted_emoji"][pair] = verdict["waived"]["emoji_unpainted"]
                    if verdict["waived"].get("digits_moved"):
                        out["page_dependent_digits"][pair] = verdict["waived"]["digits_moved"]
            else:
                out["mismatches"].append({
                    "pair": pair,
                    f"only_{native}": _show_tokens(verdict["only_native"]),
                    "only_wasm": _show_tokens(verdict["only_wasm"])})
    return out


def waived_cell(pdf: dict) -> str:
    """The 'waived' column (F12): what the comparison tolerated on this pair.
    When a row has two wasm pairs (cli_wasm, python_wasm) the same wasm gap is
    measured twice — take the max per waiver, do not double count."""
    best: dict[str, int] = {}
    for w in (pdf.get("waived") or {}).values():
        counts: dict[str, int] = {}
        for cls, n in (w.get("symbols_folded") or {}).items():
            k = f"fold:{_CLASS_SHORT.get(cls, cls)}"
            counts[k] = max(counts.get(k, 0), n)
        if w.get("pua_standins"):
            counts["pua"] = max(counts.get("pua", 0), w["pua_standins"])
        if w.get("emoji_unpainted"):
            counts["emoji"] = max(counts.get("emoji", 0), w["emoji_unpainted"])
        if w.get("digits_moved"):
            counts["digits"] = max(counts.get("digits", 0), w["digits_moved"])
        for k, n in counts.items():
            best[k] = max(best.get(k, 0), n)
    if not best:
        return "-"
    return " ".join(f"{k}x{n}" for k, n in sorted(best.items()))


# --- main ---------------------------------------------------------------------


def main() -> int:
    ap = argparse.ArgumentParser(description="jubarte CLI/Python/WASM parity check (v2)")
    ap.add_argument("--cli", required=True, type=Path, help="jubarte Rust CLI binary")
    ap.add_argument("--py-python", type=Path, default=None,
                    help="python interpreter with jubarte_redlines importable (default: the "
                         "invoking interpreter; the maturin venv is jubarte-python/.venv/bin/python)")
    ap.add_argument("--wasm", default=None,
                    help="jubarte-wasm: jubarte_wasm.js file, or a directory holding it "
                         "(wasm-pack pkg/, jubarte-wasm/npm/, node_modules/jubarte-wasm)")
    ap.add_argument("--bindings", default="cli,python,wasm",
                    help="subset to check (default cli,python,wasm; at least two; duplicates rejected)")
    ap.add_argument("--expect-commit", default=None, metavar="SHA",
                    help="engine commit the artifacts must embody (F10): the wasm package's "
                         "ENGINE_COMMIT.txt must exist and equal SHA, else the run fails (exit 2)")
    ap.add_argument("--engine-dir", type=Path, default=None, metavar="DIR",
                    help="engine checkout for the STALE rule (F10): a binding artifact older than "
                         "the newest commit touching src/, jubarte-python/src or jubarte-wasm/src "
                         "fails (exit 2)")
    ap.add_argument("--max-wasm-missing-emoji", type=int, default=None, metavar="N",
                    help="fail (exit 1) when the TOTAL wasm-unpainted emoji count across pairs "
                         "exceeds N (default: no limit; the count is always printed in the "
                         "waived column and the report)")
    ap.add_argument("--pairs", type=Path, required=True,
                    help="the pairs: release_info/sample_redline_<version>_<stamp>.csv (paths relative to "
                         "the bench root), or a bench pairs CSV (stems under --corpus)")
    ap.add_argument("--corpus", type=Path, default=None,
                    help=f"corpus root the CSV paths are relative to "
                         f"(default $NEUROTIC_DOCX_BENCH/{DEFAULT_CORPUS_REL})")
    ap.add_argument("--step", type=int, default=40,
                    help="sample every Nth data row (default 40 -> 15 pairs of a 600-pair release sample)")
    ap.add_argument("--author", default=DEFAULT_AUTHOR, help=f"default {DEFAULT_AUTHOR!r}, the CLI default")
    ap.add_argument("--date", default=DEFAULT_DATE,
                    help=f"default {DEFAULT_DATE!r} (engine DEFAULT_DATE, src/document_comparer.rs:261)")
    ap.add_argument("--out-dir", type=Path, default=HERE / "parity-work")
    ap.add_argument("--report", type=Path, default=None, help="default <out-dir>/parity_report.json")
    ap.add_argument("--timeout", type=float, default=900.0, help="seconds per binding call")
    args = ap.parse_args()

    try:
        bindings = parse_bindings(args.bindings)
    except ValueError as e:
        print(f"error: {e}", file=sys.stderr)
        return 2
    if "wasm" in bindings and not args.wasm:
        print("error: --wasm is required with the wasm binding "
              "(e.g. jubarte-wasm/npm, or a dir with node_modules/jubarte-wasm)", file=sys.stderr)
        return 2
    if args.expect_commit and not re.fullmatch(r"[0-9a-fA-F]{40}", args.expect_commit):
        print(f"error: --expect-commit must be a 40-hex sha (got: {args.expect_commit})",
              file=sys.stderr)
        return 2
    bench_root = Path(os.environ.get("NEUROTIC_DOCX_BENCH", DEFAULT_BENCH_ROOT))
    pairs_csv = args.pairs
    corpus = args.corpus or bench_root / DEFAULT_CORPUS_REL
    if not args.cli.is_file():
        print(f"error: --cli {args.cli} is not a file", file=sys.stderr)
        return 2
    if not pairs_csv.is_file():
        print(f"error: pairs CSV not found: {pairs_csv}", file=sys.stderr)
        return 2
    if args.engine_dir is not None and not args.engine_dir.is_dir():
        print(f"error: --engine-dir {args.engine_dir} is not a directory", file=sys.stderr)
        return 2

    pairs = load_pairs(pairs_csv, corpus, args.step, bench_root)
    if not pairs:
        print("error: no pairs sampled", file=sys.stderr)
        return 2
    out_dir = args.out_dir
    out_dir.mkdir(parents=True, exist_ok=True)
    report_path = args.report or out_dir / "parity_report.json"
    tmp = out_dir / "_jobs"
    tmp.mkdir(exist_ok=True)

    meta = {
        "generated": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
        "pairs_csv": str(pairs_csv),
        "corpus": str(corpus),
        "step": args.step,
        "author": args.author,
        "date": args.date,
        "bindings": bindings,
        "cli": str(args.cli),
        "py_python": str(args.py_python) if args.py_python else "in-process",
        "wasm_module": args.wasm,
        "normalisation": "docx: none — the engine pins zip entry dates to 1980-01-01 "
            "(src/opc/mod.rs:297-306) and the revision date is pinned through settings, "
            "so any byte difference is a real mismatch. PDFs: page counts strict; native "
            "bindings agree on every page's text; a pair with the WASM build (embedded "
            "Carlito/Liberation only) paints the same characters the same number of times "
            "with symbols folded PER CLASS (bullet-geo / arrow / tick-cross; private-use "
            "codes one class — which stand-in class a PUA belongs to is not knowable, "
            "src/convert/font.rs:1120-1141), emoji excluded but counted (gated by "
            "--max-wasm-missing-emoji), digits tolerated whole-document when counts agree; "
            "every tolerance is listed under 'waived'; bytes informational",
        "symbol_classes": {
            "bullet-geo": "U+00B7 U+2022 U+2023 U+2043 U+25A0..U+25FF U+2756 (bullets and "
                          "geometric shapes — wingdings_unicode, src/convert/font.rs:176-185)",
            "arrow": "U+2190..U+21FF U+003E U+27A2 (the '>' is Wingdings 0xD8's third "
                     "stand-in, src/convert/font.rs:186)",
            "tick-cross": "U+2713..U+2718 U+00D7 U+221A (the U+00D7 / √ fallbacks of Wingdings "
                          "0xFB/0xFC, src/convert/font.rs:191-192)",
            "pua": "U+E000..U+F8FF, ONE class: which stand-in class a private-use code "
                   "belongs to is not knowable; a PUA difference may pair one-for-one with "
                   "a folded symbol or an emoji-only stand-in, never with text",
        },
    }

    # ---- phase 0: build identity (F10) --------------------------------------
    versions: dict = {"cli": cli_version(args.cli, args.timeout)}
    identity: dict = {"expect_commit": args.expect_commit,
                      "engine_dir": str(args.engine_dir) if args.engine_dir else None}
    identity_failures: list[str] = []
    if args.engine_dir:
        try:
            identity["engine_src_newest_commit"] = engine_src_commit_ts(args.engine_dir)
        except (RuntimeError, subprocess.TimeoutExpired) as e:
            print(f"error: cannot read engine history: {e}", file=sys.stderr)
            return 2
    identity["cli"] = file_identity(args.cli)

    py_results: dict = {"jobs": []}
    if "python" in bindings:
        py_results = run_python_jobs({"author": args.author, "date": args.date, "jobs": []},
                                     tmp / "py_probe.json", tmp / "py_probe_results.json",
                                     args.py_python, args.timeout)
        if "error" in py_results:
            print(f"error: python binding unavailable: {py_results['error']}", file=sys.stderr)
            print("  build it: (cd jubarte-python && uv run --with maturin maturin develop --release), "
                  "then pass --py-python jubarte-python/.venv/bin/python", file=sys.stderr)
            return 2
        versions["python"] = py_results.get("version", "?")
        module = py_results.get("native_module")
        if not module:
            identity_failures.append(
                "python: the binding reports no native extension module path — "
                "build identity cannot be recorded")
        else:
            identity["python"] = file_identity(Path(module))

    wasm_results: dict = {"jobs": []}
    if "wasm" in bindings:
        wasm_results = run_wasm_jobs({"author": args.author, "date": args.date, "jobs": []},
                                     tmp / "wasm_probe.json", tmp / "wasm_probe_results.json",
                                     args.wasm, args.timeout)
        if "error" in wasm_results:
            print(f"error: wasm binding unavailable: {wasm_results['error']}", file=sys.stderr)
            print("  build it: (cd jubarte-wasm && ./build-npm.sh), then pass --wasm jubarte-wasm/npm", file=sys.stderr)
            return 2
        versions["wasm"] = {"package": wasm_results.get("packageVersion", "?"),
                            "engine_commit": wasm_results.get("engineCommit"),
                            "has_docx_to_pdf": wasm_results.get("hasDocxToPdf")}
        wasm_file = wasm_results.get("wasmFile")
        if not wasm_file:
            identity_failures.append(
                "wasm: no .wasm file beside the module — build identity cannot be recorded")
        else:
            identity["wasm"] = {**file_identity(Path(wasm_file)),
                                "package_dir": wasm_results.get("packageDir"),
                                "engine_commit_file": wasm_results.get("engineCommitFile"),
                                "engine_commit": wasm_results.get("engineCommit")}

    if args.expect_commit and "wasm" in bindings and "wasm" in identity:
        commit = identity["wasm"].get("engine_commit")
        commit_file = identity["wasm"].get("engine_commit_file")
        if not commit:
            where = commit_file or "next to the package's package.json"
            identity_failures.append(
                f"wasm: no ENGINE_COMMIT.txt beside the package ({where}) — the build "
                f"cannot be bound to {args.expect_commit}; build with tools/parity/build.sh "
                "or jubarte-wasm/build-npm.sh")
        elif commit.lower() != args.expect_commit.lower():
            identity_failures.append(
                f"wasm: ENGINE_COMMIT.txt says {commit}, expected {args.expect_commit}")
        else:
            identity["wasm"]["engine_commit_matches_expect"] = True

    src = identity.get("engine_src_newest_commit")
    if src:
        for b in ("cli", "python", "wasm"):
            ident = identity.get(b)
            if ident and ident["mtime"] < src["ts"]:
                ident["stale"] = True
                identity_failures.append(
                    f"{b}: STALE — artifact mtime {ident['mtime_iso']} predates the newest "
                    f"engine source commit {src['iso']} ({ident['path']}); rebuild it")

    if identity_failures:
        print_identity(identity)
        report = {"parity_check": meta, "versions": versions,
                  "build_identity": identity,
                  "identity_failures": identity_failures,
                  "summary": {"ok": False, "failures": identity_failures}}
        report_path.write_text(json.dumps(report, indent=1))
        for f in identity_failures:
            print(f"FAIL: {f}", file=sys.stderr)
        print(f"FAIL: build identity not provable (F10); report: {report_path}", file=sys.stderr)
        return 2

    # ---- phase 1: redline every pair with every binding --------------------
    py_jobs: dict = {"author": args.author, "date": args.date, "jobs": []}
    wasm_jobs: dict = {"author": args.author, "date": args.date, "jobs": []}
    cli_calls: list[tuple[dict, Path]] = []
    for p in pairs:
        p["dir"] = out_dir / f"row{p['row']}"
        p["dir"].mkdir(exist_ok=True)
        p["compare_errors"] = {}
        if "cli" in bindings:
            cli_calls.append((p, p["dir"] / "cli.docx"))
        if "python" in bindings:
            py_jobs["jobs"].append({"id": f"row{p['row']}-compare", "kind": "compare",
                                    "base": str(p["base"]), "next": str(p["next"]),
                                    "out": str(p["dir"] / "python.docx")})
        if "wasm" in bindings:
            wasm_jobs["jobs"].append({"id": f"row{p['row']}-compare", "kind": "compare",
                                      "base": str(p["base"]), "next": str(p["next"]),
                                      "out": str(p["dir"] / "wasm.docx")})

    if "python" in bindings:
        py_results = run_python_jobs(py_jobs, tmp / "py_compare.json",
                                     tmp / "py_compare_results.json", args.py_python, args.timeout)
        if "error" in py_results:
            print(f"error: python binding failed: {py_results['error']}", file=sys.stderr)
            return 2
    if "wasm" in bindings:
        wasm_results = run_wasm_jobs(wasm_jobs, tmp / "wasm_compare.json",
                                     tmp / "wasm_compare_results.json", args.wasm, args.timeout)
        if "error" in wasm_results:
            print(f"error: wasm binding failed: {wasm_results['error']}", file=sys.stderr)
            return 2

    for p, out in cli_calls:
        r = run_cli([str(args.cli), str(p["base"]), str(p["next"]), "-o", str(out),
                     "--force", "--quiet", "--author", args.author, "-d", args.date],
                    args.timeout)
        p["compare_errors"]["cli"] = None if r["ok"] else r["error"]
    for p in pairs:
        if "python" in bindings:
            j = job_result(py_results, f"row{p['row']}-compare")
            p["compare_errors"]["python"] = None if j and j.get("ok") else (j or {}).get("error", "missing job result")
        if "wasm" in bindings:
            j = job_result(wasm_results, f"row{p['row']}-compare")
            p["compare_errors"]["wasm"] = None if j and j.get("ok") else (j or {}).get("error", "missing job result")

    # ---- docx parity --------------------------------------------------------
    for p in pairs:
        outputs: dict[str, bytes] = {}
        for b in bindings:
            f = p["dir"] / f"{b}.docx"
            if p["compare_errors"][b] is None and f.is_file():
                outputs[b] = f.read_bytes()
        p["docx"] = {"sha256": {b: sha256(v) for b, v in outputs.items()}}
        errs = {b: e for b, e in p["compare_errors"].items() if e}
        if not errs and len(outputs) == len(bindings):
            detail = {}
            for i in range(len(bindings)):
                for j in range(i + 1, len(bindings)):
                    a, b = bindings[i], bindings[j]
                    cmp = compare_docx(outputs[a], outputs[b])
                    detail[f"{a}_{b}"] = cmp
            p["docx"]["entries"] = detail[f"{bindings[0]}_{bindings[1]}"]["entries"]
            p["docx"]["detail"] = {
                k: {"ok": v["ok"], "problems": v["problems"]} for k, v in detail.items()}
            p["docx"]["ok"] = all(v["ok"] for v in detail.values())
        elif errs and len(errs) == len(bindings) and len({err_key(v) for v in errs.values()}) == 1:
            # every binding refused the pair the same way (same admission code
            # after stripping each binding's error envelope): same behaviour,
            # nothing to compare (recorded, not a mismatch)
            p["docx"]["ok"] = True
            p["docx"]["identical_refusal"] = errs
        else:
            # at least one binding failed where another succeeded (or did not run)
            p["docx"]["ok"] = False
            p["docx"]["errors"] = errs

    # ---- phase 2: each binding converts ITS OWN redline to PDF -------------
    wasm_has_pdf = bool(wasm_results.get("hasDocxToPdf", False))
    py_pdf_jobs: dict = {"jobs": []}
    wasm_pdf_jobs: dict = {"jobs": []}
    cli_pdf_calls: list[tuple[dict, Path]] = []
    for p in pairs:
        p["pdf_errors"] = {}
        if p["docx"].get("identical_refusal"):
            continue  # nothing to convert
        if "cli" in bindings and p["compare_errors"]["cli"] is None:
            cli_pdf_calls.append((p, p["dir"] / "cli.pdf"))
        if "python" in bindings and p["compare_errors"]["python"] is None:
            py_pdf_jobs["jobs"].append(
                {"id": f"row{p['row']}-pdf", "kind": "topdf",
                 "docx": str(p["dir"] / "python.docx"), "out": str(p["dir"] / "python.pdf"),
                 "compress": False, "revisions": "word"})
        if "wasm" in bindings and p["compare_errors"]["wasm"] is None:
            if wasm_has_pdf:
                wasm_pdf_jobs["jobs"].append(
                    {"id": f"row{p['row']}-pdf", "kind": "topdf",
                     "docx": str(p["dir"] / "wasm.docx"), "out": str(p["dir"] / "wasm.pdf"),
                     "compress": False, "revisions": "word"})
            else:
                p["pdf_errors"]["wasm"] = "unavailable: slim build exports no docxToPdf"
    py_pdf_results = run_python_jobs(py_pdf_jobs, tmp / "py_pdf.json", tmp / "py_pdf_results.json",
                                     args.py_python, args.timeout) if py_pdf_jobs["jobs"] else {"jobs": []}
    wasm_pdf_results = run_wasm_jobs(wasm_pdf_jobs, tmp / "wasm_pdf.json", tmp / "wasm_pdf_results.json",
                                     args.wasm, args.timeout) if wasm_pdf_jobs["jobs"] else {"jobs": []}
    if "error" in py_pdf_results or "error" in wasm_pdf_results:
        print(f"error: pdf driver failed: "
              f"{py_pdf_results.get('error', '')}{wasm_pdf_results.get('error', '')}", file=sys.stderr)
        return 2
    for p, out in cli_pdf_calls:
        r = run_cli([str(args.cli), "convert", str(p["dir"] / "cli.docx"), "-o", str(out),
                     "--force", "--revisions", "word"], args.timeout)
        p["pdf_errors"]["cli"] = None if r["ok"] else r["error"]
    for p in pairs:
        if "python" in bindings:
            j = job_result(py_pdf_results, f"row{p['row']}-pdf")
            if j is not None:
                p["pdf_errors"]["python"] = None if j.get("ok") else j.get("error", "?")
        if "wasm" in bindings and wasm_has_pdf:
            j = job_result(wasm_pdf_results, f"row{p['row']}-pdf")
            if j is not None:
                p["pdf_errors"]["wasm"] = None if j.get("ok") else j.get("error", "?")

    # ---- pdf parity ---------------------------------------------------------
    for p in pairs:
        if p["docx"].get("identical_refusal"):
            p["pdf"] = {"ok": True, "note": "identical refusal on compare; no PDF to convert"}
            continue
        if p["docx"].get("ok") is not True:
            p["pdf"] = {"ok": True, "note": "docx parity failed; PDF comparison skipped (docx mismatch dominates)"}
            continue
        facts: dict[str, dict] = {}
        pdf_errors = {b: e for b, e in p["pdf_errors"].items() if e}
        for b in bindings:
            if b in pdf_errors:
                continue
            f = p["dir"] / f"{b}.pdf"
            if not f.is_file():
                pdf_errors[b] = "no output"
                continue
            data = f.read_bytes()
            facts[b] = {"pdf": data, "sha": sha256(data), **pdf_facts(data)}
        # A binding that HAS the operation but errored (or wrote nothing) where
        # another succeeded is a parity failure; "unavailable:" marks a build
        # that lacks the operation at all (e.g. slim wasm: no docxToPdf).
        hard = {b: e for b, e in pdf_errors.items() if not str(e).startswith("unavailable:")}
        p["pdf"] = {"errors": pdf_errors} if pdf_errors else {}
        compared = sorted(facts)
        if hard:
            p["pdf"]["ok"] = False
            continue
        # F11: fewer than two producers although at least two bindings have the
        # operation is a FAILURE, not a vacuous pass.
        coverage = pdf_coverage_failure(bindings, pdf_errors, compared)
        if coverage:
            p["pdf"]["ok"] = False
            p["pdf"]["note"] = coverage
            continue
        if not compared:
            p["pdf"]["ok"] = True
            p["pdf"]["note"] = "no PDF produced for this pair (fewer than two bindings have the operation)"
            continue
        if len(compared) == 1:
            p["pdf"]["ok"] = True
            p["pdf"]["page_count"] = {compared[0]: facts[compared[0]]["page_count"]}
            p["pdf"]["note"] = f"only one binding produced a PDF ({compared[0]}); no other binding has the operation"
            continue
        p["pdf"].update(compare_pdfs(facts))
        p["pdf"]["ok"] = not p["pdf"]["mismatches"]

    # ---- table + report -----------------------------------------------------
    docx_bad = [p["row"] for p in pairs if p["docx"].get("ok") is not True]
    pdf_bad = [p["row"] for p in pairs if p.get("pdf", {}).get("ok") is not True]
    refusals = [p["row"] for p in pairs if p["docx"].get("identical_refusal")]
    per_row_missing = missing_emoji_per_row(pairs)
    emoji_total = sum(per_row_missing.values())
    emoji_over = (args.max_wasm_missing_emoji is not None
                  and emoji_total > args.max_wasm_missing_emoji)

    print()
    print(f"parity over {len(pairs)} pairs (every {args.step}th data row of {pairs_csv.name}), "
          f"author={args.author!r} date={args.date!r}")
    print(f"versions: cli={versions.get('cli')} python={versions.get('python')} wasm={versions.get('wasm')}")
    print_identity(identity)
    hdr = f"{'row':>4}  {'key':<40}  {'docx':<8} {'pdf pages':<14} {'pdf text':<8} {'waived':<26} note"
    print(hdr)
    print("-" * len(hdr))
    for p in pairs:
        docx_cell = "refusal" if p["docx"].get("identical_refusal") else ("PASS" if p["docx"].get("ok") else "FAIL")
        pdf = p.get("pdf", {})
        names = sorted(pdf.get("page_count", {}))
        pages = "/".join(str(pdf["page_count"][n]) for n in names) if names else "-"
        text_pairs = sorted(pdf.get("text_identical", {}))
        hard_errs = {k: v for k, v in pdf.get("errors", {}).items()
                     if not str(v).startswith("unavailable:")}
        if not pdf.get("ok"):
            text_cell = "FAIL"
        elif (text_pairs or pdf.get("same_characters")) and not pdf.get("mismatches") and not hard_errs:
            text_cell = "PASS"
        else:
            text_cell = "-"
        note = ""
        if pdf.get("ok") is not True and pdf.get("note"):
            note = pdf["note"]
        refusal = p["docx"].get("identical_refusal")
        if refusal:
            note = (f"refused: {err_key(next(iter(refusal.values())))} " + note).strip()
        if p["docx"].get("errors"):
            note += " docx-errors: " + json.dumps(p["docx"]["errors"])[:100]
        if p["docx"].get("detail"):
            bad = {k: v["problems"] for k, v in p["docx"]["detail"].items() if not v["ok"]}
            if bad:
                note += " docx-diff: " + json.dumps(bad)[:140]
        if pdf.get("errors"):
            note += " pdf-errors: " + json.dumps(pdf["errors"])[:100]
        if pdf.get("mismatches"):
            note += " pdf-diff: " + json.dumps(pdf["mismatches"], ensure_ascii=False)[:200]
        key = p["key"][:38] + "…" if len(p["key"]) > 39 else p["key"]
        print(f"{p['row']:>4}  {key:<40}  {docx_cell:<8} {pages:<14} {text_cell:<8} "
              f"{waived_cell(pdf):<26} {note}")
    print()

    report = {
        "parity_check": meta,
        "versions": versions,
        "build_identity": identity,
        "pairs": [
            {"row": p["row"], "key": p["key"], "base": str(p["base"]), "next": str(p["next"]),
             "docx": p["docx"], "pdf": p.get("pdf", {}),
             "compare_errors": {k: v for k, v in p["compare_errors"].items() if v},
             "pdf_errors": {k: v for k, v in p["pdf_errors"].items() if v}}
            for p in pairs
        ],
        "summary": {
            "pairs": len(pairs),
            "docx_mismatch_rows": docx_bad,
            "pdf_mismatch_rows": pdf_bad,
            "identical_refusal_rows": refusals,
            "wasm_missing_emoji": {"per_row": per_row_missing, "total": emoji_total,
                                   "limit": args.max_wasm_missing_emoji,
                                   "over_limit": emoji_over},
            "build_identity_ok": True,
            "ok": not docx_bad and not pdf_bad and not emoji_over,
        },
    }
    report_path.write_text(json.dumps(report, indent=1))
    print(f"report: {report_path}")
    if docx_bad or pdf_bad or emoji_over:
        parts = []
        if docx_bad:
            parts.append(f"docx mismatches on rows {docx_bad}")
        if pdf_bad:
            parts.append(f"pdf mismatches on rows {pdf_bad}")
        if emoji_over:
            parts.append(f"wasm-unpainted emoji total {emoji_total} exceeds "
                         f"--max-wasm-missing-emoji {args.max_wasm_missing_emoji} "
                         f"(rows {sorted(per_row_missing)})")
        print("FAIL: " + "; ".join(parts))
        return 1
    if refusals:
        print(f"note: {len(refusals)} pair(s) refused identically by every binding: rows {refusals}")
    if per_row_missing:
        print(f"note: wasm left emoji unpainted on {len(per_row_missing)} pair(s), "
              f"total {emoji_total} (waived; gate with --max-wasm-missing-emoji)")
    print("PASS: all bindings agree on every sampled pair, from builds whose identity is recorded")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

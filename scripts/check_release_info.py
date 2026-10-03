#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

# SPDX-License-Identifier: AGPL-3.0-only

"""Refuse a release whose release_info/ evidence is missing or unsound.

`python3 scripts/check_release_info.py 0.11.2` exits 1 and lists each
problem; scripts/release.sh runs it as its step 3, right after the changelog
check. The benchmark writes the six files (release_info/README.md names
them): two 600-item sample CSVs, two results JSONs scored on exactly those
samples, and two JSONL lists of the website facts and app items the release
moves. Plain Python 3, stdlib only; it reads the folder and never writes it.

What is proven outright: the six exact names under one shared stamp that is
a real date and time; the exact CSV headers and 600 data rows of exactly the
header's width, each with a unique non-empty key/stem and the Word oracle
path with its sha; results JSONs bound to the sample CSV beside them by that
file's real sha256, with every tool's n equal to the sample's row count,
failures within 0..n, mean/median within 0..100, and jubarte's version,
40-hex commit and 64-hex binary sha256; JSONL records that are objects with
a non-empty string key, website_data holding the five keys
scripts/check_release_facts.py reads for a release.

The sha256 columns name files that live in the BENCH repository, so without
--bench-root they are format-checked only. With --bench-root DIR (release.sh
passes it when a bench checkout is at hand) every non-empty path cell is
resolved against DIR and its sha256 column is verified against the real
file — the actual proof the evidence names real files. --commit SHA and
--binary-sha256 HEX additionally bind tools.jubarte's commit and
binary_sha256 to the release's own binary; release.sh passes neither, since
the scored binary is a candidate built before the release commit exists
(release_info/README.md, "Which binary the evidence names").
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import re
import sys
from datetime import datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
INFO = Path("release_info")

STAMP = r"\d{2}-\d{2}-\d{2}_\d{2}-\d{2}"
SHA256 = re.compile(r"^[0-9a-f]{64}$")
COMMIT = re.compile(r"^[0-9a-f]{40}$")
VERSION = re.compile(r"^\d+\.\d+\.\d+$")
DEFAULT_ROWS = 600  # each release sample is 600 items (release_info/README.md)

# definition → extension; the six files of a release, in a stable order.
SIX = (
    ("sample_redline", "csv"),
    ("sample_conversion", "csv"),
    ("results_redline", "json"),
    ("results_conversion", "json"),
    ("website_data", "jsonl"),
    ("app_data", "jsonl"),
)

# The CSV headers, exactly as the bench writer emits them
# (neurotic_docx_bench jubarte_release_info.py REDLINE_HEADER /
# CONVERSION_HEADER); every path column is immediately followed by its
# sha256 sibling (release_info/README.md).
REDLINE_HEADER = [
    "key", "base", "base_sha256", "next", "next_sha256",
    "docx", "docx_sha256", "pdf", "pdf_sha256",
    "state", "id", "sets", "oracle",
    "oracle_pdf", "oracle_pdf_sha256",
    "docxodus_pdf", "docxodus_pdf_sha256",
    "jubarte_docx", "jubarte_docx_sha256",
    "jubarte_pdf", "jubarte_pdf_sha256",
]
CONVERSION_HEADER = [
    "state", "stem",
    "docx", "docx_sha256", "word_pdf", "word_pdf_sha256",
    "jubarte_pdf", "jubarte_pdf_sha256",
    "soffice_pdf", "soffice_pdf_sha256",
]
PATH_COLUMNS = {
    "sample_redline": ["base", "next", "docx", "pdf", "oracle_pdf",
                       "docxodus_pdf", "jubarte_docx", "jubarte_pdf"],
    "sample_conversion": ["docx", "word_pdf", "jubarte_pdf", "soffice_pdf"],
}
# the one path column each CSV cannot do without: the Word-made reference
# (the writer's sampling guarantees it exists for every sampled item).
MUST_HAVE = {
    "sample_redline": ("oracle_pdf", "every sampled compare must carry its Word oracle PDF"),
    "sample_conversion": ("word_pdf", "every sampled fixture must carry Word's own PDF"),
}
# key (redline) / stem (conversion): the column a row is identified by.
KEY_COLUMN = {"sample_redline": "key", "sample_conversion": "stem"}
# tool name → the results JSONs' tools block must carry these, numeric.
AGGREGATES = ("n", "failures", "mean", "median")
JUBARTE = "jubarte"
# The keys scripts/check_release_facts.py reads to call a release done (its
# problems() refuses the release while any of these is absent from
# jubarte-app/data/facts.jsonl). website_data drafts them; a record marked
# "pending": true (a placeholder the site step fills) counts as present.
FACTS_KEYS = ("engine.version", "engine.released", "release.history",
              "release.archives", "release.wheels")


def _valid_stamp(stamp: str) -> bool:
    """mm-dd-yy_hh-mm that names a real date and time (the writer's stamp_now)."""
    try:
        datetime.strptime(stamp, "%m-%d-%y_%H-%M")
    except ValueError:
        return False
    return True


def find_six(folder: Path, version: str) -> tuple[dict[str, Path], list[str]]:
    """The six files of `version`, and everything wrong with what is there."""
    known = dict(SIX)
    found: dict[str, list[Path]] = {}
    problems: list[str] = []
    for path in sorted(folder.glob(f"*_{version}_*")):
        match = re.fullmatch(rf"(\w+?)_{re.escape(version)}_({STAMP})\.(\w+)", path.name)
        if match is None:
            problems.append(
                f"{path.name}: carries {version} but is not one of the six names "
                f"(<definition>_{version}_<mm-dd-yy_hh-mm>.<csv|json|jsonl>) — a mistyped file")
            continue
        name, stamp, ext = match.groups()
        if name not in known:
            problems.append(f"{path.name}: not one of the six definitions ({', '.join(known)})")
            continue
        if ext != known[name]:
            problems.append(f"{path.name}: a {name} file is .{known[name]}, not .{ext}")
            continue
        if not _valid_stamp(stamp):
            problems.append(f"{path.name}: stamp {stamp} is not a real date and time (mm-dd-yy_hh-mm)")
            continue
        found.setdefault(name, []).append(path)
    out: dict[str, Path] = {}
    for name, _ in SIX:
        got = found.get(name, [])
        if len(got) == 0:
            problems.append(f"{name}_{version}_<mm-dd-yy_hh-mm>: no such file in {folder}")
        elif len(got) > 1:
            problems.append(f"{name}: {len(got)} files for {version} ({', '.join(p.name for p in got)})")
        else:
            out[name] = got[0]
    stamps = {p.name.split(f"_{version}_")[1].rsplit(".", 1)[0] for p in out.values()}
    if len(out) == len(SIX) and len(stamps) > 1:
        problems.append(f"the six files carry {len(stamps)} stamps ({', '.join(sorted(stamps))})")
    return out, problems


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def check_sample_csv(path: Path, header: list[str], paths: list[str], rows: int,
                     bench_root: Path | None) -> list[str]:
    """Header, row count, row width, keys, the sha sibling of every path
    column, the must-have oracle column — and, with a bench root, the real
    file behind every non-empty path cell."""
    problems: list[str] = []
    try:
        with path.open(newline="", encoding="utf-8") as fh:
            data = list(csv.reader(fh))
    except UnicodeDecodeError as exc:
        return [f"{path.name}: not UTF-8 text ({exc.reason} at byte {exc.start}) — a corrupted or binary file"]
    if not data:
        return [f"{path.name}: holds no header row"]
    got, data = data[0], data[1:]
    if got != header:
        return [f"{path.name}: header is {got}, want {header}"]
    real = [row for row in data if row]  # a blank line is not a row
    if len(real) != rows:
        problems.append(f"{path.name}: {len(real)} data rows, want {rows}")
    kind = "sample_redline" if "key" in header else "sample_conversion"
    key_col, must_col, must_why = KEY_COLUMN[kind], *MUST_HAVE[kind]
    ki, mi = header.index(key_col), header.index(must_col)
    ms = header.index(f"{must_col}_sha256")
    seen: set[str] = set()
    for n, row in enumerate(data, 2):
        if not row:
            continue  # a blank line is not a row (it is not checked either)
        key = row[ki] if ki < len(row) else ""
        if not key:
            problems.append(f"{path.name} row {n}: {key_col} is empty")
        elif key in seen:
            problems.append(f"{path.name} row {n}: {key_col} {key} is listed twice")
        else:
            seen.add(key)
        if len(row) != len(header):
            problems.append(f"{path.name} row {n}: {len(row)} cells, want the header's {len(header)}")
        if not (row[mi] if mi < len(row) else ""):
            problems.append(f"{path.name} row {n}: {must_col} is empty — {must_why}")
        elif not SHA256.fullmatch(row[ms] if ms < len(row) else ""):
            problems.append(f"{path.name} row {n}: {must_col} without its sha256")
        for col in paths:
            i, j = header.index(col), header.index(f"{col}_sha256")
            value = row[i] if i < len(row) else ""
            sha = row[j] if j < len(row) else ""
            if SHA256.fullmatch(sha):
                if not value:
                    problems.append(f"{path.name} row {n}: {col}_sha256 without {col}")
            elif sha == "":
                if value:
                    problems.append(f"{path.name} row {n}: {col} has no sha256")
            else:
                problems.append(f"{path.name} row {n}: {col}_sha256 is not 64 lowercase hex")
            if bench_root is not None and value:
                if Path(value).is_absolute():
                    problems.append(f"{path.name} row {n}: {col} {value} is not relative to the bench root")
                    continue
                real_file = bench_root / value
                if not real_file.is_file():
                    problems.append(f"{path.name} row {n}: {col} {value} does not exist under {bench_root}")
                elif SHA256.fullmatch(sha) and _sha256(real_file) != sha:
                    problems.append(f"{path.name} row {n}: {col} {value} hashes to "
                                    f"{_sha256(real_file)}, not {sha}")
    return problems


def sample_row_count(sample: Path) -> int | None:
    """Data rows of a sample CSV (blank lines are not rows); None when unreadable."""
    try:
        with sample.open(newline="", encoding="utf-8") as fh:
            data = list(csv.reader(fh))
    except (OSError, UnicodeDecodeError, csv.Error):
        return None
    return len([row for row in data[1:] if row]) if data else None


def check_results_json(path: Path, name: str, six: dict[str, Path], version: str,
                       want_commit: str | None, want_sha: str | None) -> list[str]:
    """The sample it scores is the CSV beside it, both tools answer with sound
    aggregates, and jubarte's identity is the writer's (version naming the
    release, 40-hex commit, 64-hex binary sha256)."""
    problems: list[str] = []
    try:
        doc = json.loads(path.read_text(encoding="utf-8"))
    except UnicodeDecodeError as exc:
        return [f"{path.name}: not UTF-8 text ({exc.reason} at byte {exc.start})"]
    except ValueError as exc:
        return [f"{path.name} is not JSON: {exc}"]
    sample = doc.get("sample") or {}
    csv_name, csv_sha = sample.get("csv"), sample.get("sha256")
    sample_csv = six.get(f"sample_{name[len('results_'):]}")
    rows = sample_row_count(sample_csv) if sample_csv is not None else None
    if not csv_name:
        problems.append(f"{path.name}: sample.csv is missing")
    elif sample_csv is None or sample_csv.name != csv_name:
        problems.append(f"{path.name}: scores sample {csv_name}, not {sample_csv}")
    elif not csv_sha or not SHA256.fullmatch(str(csv_sha)):
        problems.append(f"{path.name}: sample.sha256 is not 64 lowercase hex")
    else:
        digest = _sha256(sample_csv)
        if digest != csv_sha:
            problems.append(f"{path.name}: sample.sha256 is {csv_sha}; {csv_name} hashes to {digest}")
    if rows is not None and sample.get("n") != rows:
        problems.append(f"{path.name}: sample.n is {sample.get('n')}, the sample holds {rows} rows")
    tools = doc.get("tools") or {}
    if JUBARTE not in tools:
        problems.append(f"{path.name}: no {JUBARTE} in tools")
    comparators = [t for t in tools if t != JUBARTE]
    if not comparators:
        problems.append(f"{path.name}: no comparator beside {JUBARTE}")
    for tool in [JUBARTE, *comparators]:
        block = tools.get(tool) or {}
        for field in AGGREGATES:
            value = block.get(field)
            if not isinstance(value, (int, float)) or isinstance(value, bool):
                problems.append(f"{path.name}: tools.{tool}.{field} is not a number")
        n = block.get("n")
        if isinstance(n, bool) or not isinstance(n, (int, float)):
            continue
        if rows is not None and n != rows:
            problems.append(f"{path.name}: tools.{tool}.n is {n}, the sample holds {rows} rows")
        for field in ("failures",):
            value = block.get(field)
            if isinstance(value, (int, float)) and not isinstance(value, bool) and not 0 <= value <= n:
                problems.append(f"{path.name}: tools.{tool}.{field} is {value}, want 0..{n:g}")
        for field in ("mean", "median"):
            value = block.get(field)
            if isinstance(value, (int, float)) and not isinstance(value, bool) and not 0 <= value <= 100:
                problems.append(f"{path.name}: tools.{tool}.{field} is {value}, want 0..100")
    jub = tools.get(JUBARTE) or {}
    ver = jub.get("version")
    if not isinstance(ver, str) or not ver.strip():
        problems.append(f"{path.name}: tools.jubarte.version is not a non-empty string")
    elif not re.search(rf"(?<![\w.]){re.escape(version)}(?![\w.])", ver):
        problems.append(f"{path.name}: tools.jubarte.version is {ver!r}, which does not name {version}")
    commit = jub.get("commit")
    if not isinstance(commit, str) or not COMMIT.fullmatch(commit):
        problems.append(f"{path.name}: tools.jubarte.commit is not 40 lowercase hex "
                        "(the bench writes the engine checkout's HEAD)")
    binary = jub.get("binary_sha256")
    if not isinstance(binary, str) or not SHA256.fullmatch(binary):
        problems.append(f"{path.name}: tools.jubarte.binary_sha256 is not 64 lowercase hex")
    if want_commit and isinstance(commit, str) and COMMIT.fullmatch(commit) and commit != want_commit:
        problems.append(f"{path.name}: tools.jubarte.commit is {commit}, not the given {want_commit}")
    if want_sha and isinstance(binary, str) and SHA256.fullmatch(binary) and binary != want_sha:
        problems.append(f"{path.name}: tools.jubarte.binary_sha256 is {binary}, not the given {want_sha}")
    return problems


def check_jsonl(path: Path, required: tuple[str, ...] = ()) -> list[str]:
    problems: list[str] = []
    try:
        text = path.read_text(encoding="utf-8")
    except UnicodeDecodeError as exc:
        return [f"{path.name}: not UTF-8 text ({exc.reason} at byte {exc.start}) — a corrupted or binary file"]
    keys: set[str] = set()
    for n, raw in enumerate(text.splitlines(), 1):
        if not raw.strip():
            continue
        try:
            rec = json.loads(raw)
        except ValueError as exc:
            problems.append(f"{path.name} line {n} is not JSON: {exc}")
            continue
        if not isinstance(rec, dict):
            problems.append(f"{path.name} line {n} is not an object")
            continue
        key = rec.get("key")
        if not isinstance(key, str) or not key.strip():
            problems.append(f"{path.name} line {n}: key is not a non-empty string")
        else:
            keys.add(key)  # a "pending": true record counts as present
    if not text.strip():
        problems.append(f"{path.name} holds no records")
    for key in required:
        if key not in keys:
            problems.append(f"{path.name}: no record keyed {key} — scripts/check_release_facts.py reads it")
    return problems


def print_aggregates(six: dict[str, Path]) -> None:
    for name in ("results_redline", "results_conversion"):
        path = six.get(name)
        if path is None:
            continue
        doc = json.loads(path.read_text(encoding="utf-8"))
        print(f"{name}:")
        jub = (doc.get("tools") or {}).get(JUBARTE) or {}
        if isinstance(jub.get("commit"), str):
            print(f"  jubarte binary sha256 {jub.get('binary_sha256')}, commit {jub.get('commit')}, "
                  f"reports {jub.get('candidate_reports')!r}")
        for tool, data in sorted((doc.get("tools") or {}).items()):
            print(f"  {tool}: n={data.get('n')} failures={data.get('failures')} "
                  f"mean={data.get('mean')} median={data.get('median')} "
                  f"=100:{data.get('exact_100')} >=90:{data.get('at_least_90')}")
        if comparison := doc.get("comparison"):
            print(f"  jubarte - {comparison.get('comparator')}: median delta "
                  f"{comparison.get('median_delta')}, 95% CI {comparison.get('ci95')} "
                  f"({comparison.get('bootstrap', {}).get('reps')} resamples, "
                  f"seed {comparison.get('bootstrap', {}).get('seed')})")


def problems(version: str, folder: Path, rows: int, bench_root: Path | None = None,
             want_commit: str | None = None, want_sha: str | None = None) -> list[str]:
    six, found = find_six(folder, version)
    out = list(found)
    if "sample_redline" in six:
        out += check_sample_csv(six["sample_redline"], REDLINE_HEADER, PATH_COLUMNS["sample_redline"], rows, bench_root)
    if "sample_conversion" in six:
        out += check_sample_csv(six["sample_conversion"], CONVERSION_HEADER, PATH_COLUMNS["sample_conversion"], rows, bench_root)
    for name in ("results_redline", "results_conversion"):
        if name in six:
            out += check_results_json(six[name], name, six, version, want_commit, want_sha)
    if "website_data" in six:
        out += check_jsonl(six["website_data"], FACTS_KEYS)
    if "app_data" in six:
        out += check_jsonl(six["app_data"])
    if not out:
        print_aggregates(six)
    return out


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("version", help="the release, x.y.z")
    ap.add_argument("--root", type=Path, default=ROOT, help="the engine checkout (default: this script's)")
    ap.add_argument("--dir", type=Path, default=INFO, help="the evidence folder inside it (default: release_info)")
    ap.add_argument("--rows", type=int, default=DEFAULT_ROWS,
                    help=f"data rows each sample CSV must have (default: {DEFAULT_ROWS})")
    ap.add_argument("--bench-root", type=Path, default=None,
                    help="the neurotic_docx_bench checkout: every non-empty path cell is resolved against it "
                         "and its sha256 column is verified against the real file")
    ap.add_argument("--commit", help="the engine commit the scored binary was built from (40 hex); "
                                     "tools.jubarte.commit must equal it")
    ap.add_argument("--binary-sha256", help="the scored binary's sha256 (64 hex); "
                                            "tools.jubarte.binary_sha256 must equal it")
    args = ap.parse_args(argv)
    if not VERSION.fullmatch(args.version):
        print(f"not a release version: {args.version}", file=sys.stderr)
        return 2
    if args.commit is not None and not COMMIT.fullmatch(args.commit):
        print(f"--commit is not 40 lowercase hex: {args.commit}", file=sys.stderr)
        return 2
    if args.binary_sha256 is not None and not SHA256.fullmatch(args.binary_sha256):
        print(f"--binary-sha256 is not 64 lowercase hex: {args.binary_sha256}", file=sys.stderr)
        return 2
    if args.bench_root is not None and not args.bench_root.is_dir():
        print(f"no such bench root: {args.bench_root}", file=sys.stderr)
        return 2
    folder = args.dir if args.dir.is_absolute() else args.root / args.dir
    if not folder.is_dir():
        print(f"no {folder}: a release needs its six files there (release_info/README.md)", file=sys.stderr)
        return 1
    found = problems(args.version, folder, args.rows, args.bench_root, args.commit, args.binary_sha256)
    for p in found:
        print(f"  ✗ {p}", file=sys.stderr)
    if found:
        print(f"release_info/ is not on {args.version}: run the bench flow that writes the six files "
              f"(neurotic_docx_bench jubarte_release_info {args.version} --engine-dir <this checkout>), "
              "commit them, then rerun this check.", file=sys.stderr)
        return 1
    print(f"release_info/ carries {args.version}'s six files")
    if args.bench_root is not None:
        print(f"  every non-empty sample path was verified against {args.bench_root}")
    else:
        print("  (sha256 columns format-checked only — pass --bench-root <bench checkout> "
              "to verify them against the real files)")
    return 0


if __name__ == "__main__":
    sys.exit(main())

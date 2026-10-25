#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

# SPDX-License-Identifier: AGPL-3.0-only

"""Prove two jubarte binaries write the same bytes on the release samples.

`python3 scripts/identity.py BIN_A BIN_B OUT_DIR CSV... --root BENCH_ROOT`
runs both binaries on every row of each release sample CSV (release_info/'s
`sample_redline_*` and `sample_conversion_*`, paths relative to the bench
root) with the bench's own flags: a redline is `BIN BASE NEXT -o DEST
--force --quiet -d 1970-01-01T00:00:00Z`, a conversion `BIN convert DOCX -o
DEST --force --revisions word --compress`. It compares the two outputs byte
for byte and the two exit codes.

The checklist's before#6 (the release candidate against the scored one) and
after#8 (each published binary against the candidate) name this tool, so an
identity claim is a logged run rather than a person's word (#1752). Each
output is named by its CSV and row index, never by a stem: 0.13.0's ad-hoc
check named outputs by stem[:60], and parallel jobs on stems sharing that
prefix overwrote each other into 25 false differences.

OUT_DIR/identity.log records both binaries (path, sha256, --version) and
every difference; the outputs of a differing row stay beside it, identical
ones are deleted. A row where neither binary writes an output and both exit
alike is identical (a jubarte failure is the same failure), but a run where
no row writes anything proves nothing and fails.

Exit 0 when every row agrees; 1 on a difference, a one-sided output, a
timeout on either side, or a run with no output at all; 2 when a binary is
no executable file, or a CSV cannot be read, is of neither sample kind, has
a row short of a path, or names an input outside the bench root or missing
from it. Plain Python 3, stdlib only.
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import os
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass
from pathlib import Path

EPOCH = "1970-01-01T00:00:00Z"


@dataclass(frozen=True)
class Job:
    label: str  # CSV and row index, then the row's key or stem
    name: str  # the output's file name stem: CSV index and row index only
    kind: str  # "redline" or "conversion"
    inputs: tuple[Path, ...]

    def command(self, binary: Path, dest: Path) -> list[str]:
        if self.kind == "redline":
            base, nxt = self.inputs
            return [str(binary), str(base), str(nxt), "-o", str(dest), "--force", "--quiet", "-d", EPOCH]
        (docx,) = self.inputs
        return [str(binary), "convert", str(docx), "-o", str(dest), "--force", "--revisions", "word", "--compress"]

    @property
    def suffix(self) -> str:
        return ".docx" if self.kind == "redline" else ".pdf"


@dataclass(frozen=True)
class Run:
    code: int | None  # None: timed out
    data: bytes | None  # None: no output


def jobs_of(csvs: list[Path], root: Path) -> list[Job] | str:
    """Every row of every CSV as a job, or the reason a CSV is refused."""
    out, absent = [], []
    for c, path in enumerate(csvs):
        try:
            with path.open(newline="", encoding="utf-8") as f:
                rows = list(csv.DictReader(f))
        except (OSError, UnicodeDecodeError) as e:
            return f"cannot read {path} ({getattr(e, 'strerror', None) or e})"
        fields = set(rows[0]) if rows else set()
        if {"key", "base", "next"} <= fields:
            kind, columns, ident = "redline", ("base", "next"), "key"
        elif {"stem", "docx"} <= fields:
            kind, columns, ident = "conversion", ("docx",), "stem"
        else:
            return f"{path} is neither a redline sample (key, base, next) nor a conversion sample (stem, docx)"
        for r, row in enumerate(rows):
            short = [col for col in columns if not row.get(col)]
            if short:
                return f"{path.name} row {r} names no {', '.join(short)}"
            # every input relative to the root, as check_release_info.py --bench-root holds them
            outside = [row[col] for col in columns
                       if Path(row[col]).is_absolute() or ".." in Path(row[col]).parts]
            if outside:
                return f"{path.name} row {r} names a path outside --root: {outside[0]}"
            inputs = tuple(root / row[col] for col in columns)
            absent += [str(p.relative_to(root)) for p in inputs if not p.is_file()]
            out.append(Job(f"{path.name}#{r} {row[ident]}", f"{c}-{r}", kind, inputs))
    if absent:
        return f"{len(absent)} input(s) missing under {root}: " + ", ".join(absent[:5])
    if not out:
        return "the CSVs hold no rows"
    return out


def run(binary: Path, job: Job, dest: Path, timeout: float) -> Run:
    dest.unlink(missing_ok=True)
    try:
        code = subprocess.run(job.command(binary, dest), capture_output=True, timeout=timeout).returncode
    except subprocess.TimeoutExpired:
        code = None
    return Run(code, dest.read_bytes() if dest.is_file() else None)


def described(binary: Path) -> str:
    sha = hashlib.sha256(binary.read_bytes()).hexdigest()
    shown = subprocess.run([str(binary), "--version"], capture_output=True, text=True, timeout=60)
    return f"{binary} sha256 {sha} ({(shown.stdout or shown.stderr).strip()})"


def compare(job: Job, a: Run, b: Run, paths: tuple[Path, Path]) -> str | None:
    """None when the two runs agree; otherwise the line naming the difference."""
    timed_out = [side for side, r in (("A", a), ("B", b)) if r.code is None]
    if timed_out:  # a hang on both sides proves nothing either (#1774)
        return f"TIMEOUT {job.label}: {' and '.join(timed_out)}"
    if (a.data is None) != (b.data is None):
        side = "B" if b.data is None else "A"
        return f"MISSING {job.label}: no output from {side} (exits {a.code} / {b.code})"
    if a.data != b.data:
        return f"DIFF {job.label}: {paths[0].name} {paths[1].name}"
    if a.code != b.code:
        return f"EXIT {job.label}: exits {a.code} / {b.code}"
    return None


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("bin_a", type=Path)
    ap.add_argument("bin_b", type=Path)
    ap.add_argument("out_dir", type=Path)
    ap.add_argument("csvs", type=Path, nargs="+")
    ap.add_argument("--root", type=Path, required=True, help="the bench root the CSV paths are relative to")
    ap.add_argument("--jobs", type=int, default=os.cpu_count() or 4)
    ap.add_argument("--timeout", type=float, default=300, help="seconds per binary per row (default 300)")
    args = ap.parse_args()

    for binary in (args.bin_a, args.bin_b):
        if not (binary.is_file() and os.access(binary, os.X_OK)):
            print(f"identity: {binary} is no executable file", file=sys.stderr)
            return 2
    jobs = jobs_of(args.csvs, args.root.resolve())
    if isinstance(jobs, str):
        print(f"identity: {jobs}", file=sys.stderr)
        return 2
    args.out_dir.mkdir(parents=True, exist_ok=True)
    log = (args.out_dir / "identity.log").open("w")

    def say(line: str) -> None:
        print(line, flush=True)
        log.write(line + "\n")

    say(f"A: {described(args.bin_a)}")
    say(f"B: {described(args.bin_b)}")

    def one(job: Job) -> tuple[Job, str | None, bool]:
        paths = (args.out_dir / f"{job.name}-a{job.suffix}", args.out_dir / f"{job.name}-b{job.suffix}")
        a = run(args.bin_a, job, paths[0], args.timeout)
        b = run(args.bin_b, job, paths[1], args.timeout)
        problem = compare(job, a, b, paths)
        if problem is None:
            for p in paths:
                p.unlink(missing_ok=True)
        return job, problem, a.data is None and b.data is None

    with ThreadPoolExecutor(max_workers=max(1, args.jobs)) as pool:
        results = list(pool.map(one, jobs))
    problems = [p for _, p, _ in results if p is not None]
    empty = sum(1 for _, p, none in results if p is None and none)
    for line in problems:
        say(line)
    same = len(results) - len(problems)
    summary = f"{len(results)} jobs: {same} identical"
    if problems:
        summary += f", {len(problems)} different"
    if empty:
        summary += f" ({empty} of them no output from either)"
    say(summary)
    if empty == len(results):
        say("identity: no job wrote an output, so the run proves nothing (a flag both binaries reject?)")
        return 1
    log.close()
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())

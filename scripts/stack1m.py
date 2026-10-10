#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

# SPDX-License-Identifier: AGPL-3.0-only

"""Run a release binary over the release samples under a small main-thread stack.

`python3 scripts/stack1m.py BINARY BENCH_ROOT OUT_DIR [--kib 1024] CSV...`
runs every row of each release_info sample CSV (paths relative to the bench
root) with RLIMIT_STACK set to --kib before exec, which sizes the main
thread's stack: a redline CSV row runs `compare base next`, a conversion CSV
row runs `convert docx`. 1024 KiB is Windows' default main-thread stack, so a
deep recursion that the 8 MiB Unix default hides fails here. Exits 1 and
lists each job that did not exit zero (a stack overflow dies by signal).
"""

import argparse
import concurrent.futures
import csv
import subprocess
import sys
from pathlib import Path


def jobs(bench, out, csv_path):
    """(label, argv tail) for each row of one release sample CSV."""
    with open(csv_path, newline="") as handle:
        rows = list(csv.DictReader(handle))
    for index, row in enumerate(rows):
        label = "{}:{}".format(Path(csv_path).name, index)
        target = out / "{}_{}".format(Path(csv_path).stem, index)
        if "next" in row:
            yield label, ["compare", str(bench / row["base"]), str(bench / row["next"]),
                          "-o", str(target.with_suffix(".docx")), "--force"]
        else:
            yield label, ["convert", str(bench / row["docx"]), "-o", str(target.with_suffix(".pdf")),
                          "--force"]


def run(binary, kib, timeout, label, tail):
    # `ulimit -s` sets the soft limit that sizes the main thread at exec;
    # Python's setrlimit is refused on macOS for any value.
    argv = ["/bin/sh", "-c", 'ulimit -s "$0" && exec "$@"', str(kib), binary] + tail
    try:
        done = subprocess.run(argv, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, timeout=timeout)
    except subprocess.TimeoutExpired:
        return label, "timed out after {} s".format(timeout)
    if done.returncode == 0:
        return label, None
    reason = "signal {}".format(-done.returncode) if done.returncode < 0 else "exit {}".format(done.returncode)
    return label, "{}: {}".format(reason, done.stderr.decode(errors="replace").strip()[-200:])


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("binary")
    parser.add_argument("bench_root", type=Path)
    parser.add_argument("out_dir", type=Path)
    parser.add_argument("csv", nargs="+")
    parser.add_argument("--kib", type=int, default=1024)
    parser.add_argument("--jobs", type=int, default=4)
    parser.add_argument("--timeout", type=int, default=300)
    args = parser.parse_args(argv)
    args.out_dir.mkdir(parents=True, exist_ok=True)
    work = [job for path in args.csv for job in jobs(args.bench_root, args.out_dir, path)]
    failed = []
    with concurrent.futures.ThreadPoolExecutor(args.jobs) as pool:
        results = [pool.submit(run, args.binary, args.kib, args.timeout, label, tail) for label, tail in work]
        for future in concurrent.futures.as_completed(results):
            label, problem = future.result()
            if problem:
                failed.append((label, problem))
    for label, problem in sorted(failed):
        print("FAIL {}: {}".format(label, problem))
    print("{} jobs at a {} KiB main-thread stack: {} failed".format(len(work), args.kib, len(failed)))
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())

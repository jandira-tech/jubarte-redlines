#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Re-score the 50-fixture regression sample after every converter change.

Lives in <jubarte-redlines>/planning; expects ../../docxide-pdf and ../../neurotic_docx_bench
(fixture paths in sample50.tsv are relative to this directory; the scorer binary path is
derived from T). Run from the repository root:

    python3 planning/sample50_check.py --bless   # record the current binary's scores as the baseline
    python3 planning/sample50_check.py           # compare with the baseline; exit 1 on regression

Rules (plan.md, ground rules): a row that drops by more than --max-drop Jaccard points,
or a sample mean that drops by more than --max-mean-drop, is a regression. A convert
failure is a regression and is never blessed. Missing binaries exit 2 with the path.
Rasters are deleted by the scorer after each document; only the JSON survives.
"""
import argparse, json, os, shutil, statistics as st, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))          # <jubarte-redlines>/planning
JUBARTE = os.path.dirname(HERE)                             # <jubarte-redlines>
T = os.path.dirname(JUBARTE)                                # ~/temp/T: docxide-pdf, neurotic_docx_bench, jubarte-redlines


def load_rows(sample):
    """Parse sample50.tsv; relative paths resolve against this directory.

    Ids name the scratch PDF, so one that is empty or carries a path
    separator / `..` (it would write outside the work dir) is rejected.
    """
    rows = []
    for line in open(sample, encoding="utf-8"):
        if line.startswith("#") or not line.strip():
            continue
        s, id_, docx, ref, _j, stratum = line.rstrip("\n").split("\t")
        if not id_ or id_ in (".", "..") or os.path.basename(id_) != id_ or os.sep in id_ or "/" in id_:
            raise ValueError(f"sample id must be a plain file stem: {id_!r}")
        rows.append(dict(set=s, id=id_, stratum=stratum,
                         docx=os.path.normpath(os.path.join(HERE, docx)),
                         ref=os.path.normpath(os.path.join(HERE, ref))))
    return rows


def convert_and_score(rows, jubarte, scorer, workers):
    work = tempfile.mkdtemp(prefix="sample50_")
    try:
        jobs, failed = [], []
        for r in rows:
            out = os.path.join(work, r["id"] + ".pdf")
            p = subprocess.run([jubarte, "convert", r["docx"], "-o", out, "--force"], capture_output=True, text=True)
            if p.returncode != 0 or not os.path.exists(out):
                failed.append(r["id"])
                continue
            jobs.append(dict(stem=r["id"], oracle=r["ref"], candidate=out))
        if failed:
            return {}, failed
        jobs_path, scores_path, scratch = (os.path.join(work, n) for n in ("jobs.json", "scores.json", "scratch"))
        os.makedirs(scratch, exist_ok=True)
        json.dump(jobs, open(jobs_path, "w"))
        subprocess.run([scorer, "--jobs", jobs_path, "--scratch", scratch, "--out", scores_path, "--workers", str(workers)], check=True)
        raw = json.load(open(scores_path))
        raw = raw if isinstance(raw, list) else list(raw.values())
        return {s["stem"]: s for s in raw}, failed
    finally:
        shutil.rmtree(work, ignore_errors=True)


def main(argv=None):
    ap = argparse.ArgumentParser()
    ap.add_argument("--bless", action="store_true")
    ap.add_argument("--jubarte", default=os.path.join(JUBARTE, "target", "release", "jubarte"))
    ap.add_argument("--scorer", default=os.path.join(T, "neurotic_docx_bench", "src", "neurotic_docx_bench", "utils", "docxide-metrics", "target", "release", "docxide-metrics"))
    ap.add_argument("--sample", default=os.path.join(HERE, "sample50.tsv"))
    ap.add_argument("--baseline", default=os.path.join(HERE, "sample50_baseline.json"))
    ap.add_argument("--workers", type=int, default=4)
    ap.add_argument("--max-drop", type=float, default=1.0)
    ap.add_argument("--max-mean-drop", type=float, default=0.2)
    a = ap.parse_args(argv)

    for label, path in (("jubarte binary", a.jubarte), ("scorer", a.scorer)):
        if not os.path.isfile(path):
            print(f"missing {label}: {path}", file=sys.stderr)
            return 2
    rows = load_rows(a.sample)
    missing = [p for r in rows for p in (r["docx"], r["ref"]) if not os.path.isfile(p)]
    if missing:
        print(f"missing fixtures (clone the siblings next to jubarte-redlines): {missing[0]}", file=sys.stderr)
        return 2

    scores, failed = convert_and_score(rows, a.jubarte, a.scorer, a.workers)
    if failed:
        # A failed conversion scores 0 and would bless a hole into the baseline.
        print(f"RESULT: REGRESSION — convert failures: {failed}")
        return 1

    def pct(v):
        return 0.0 if v is None else (v * 100.0 if v <= 1.0 else v)
    cur = {r["id"]: {k: pct(scores.get(r["id"], {}).get(k)) for k in ("jaccard", "ssim", "text_boundary")} for r in rows}
    mean = st.mean(v["jaccard"] for v in cur.values())

    if a.bless or not os.path.exists(a.baseline):
        json.dump(dict(jubarte=os.path.relpath(a.jubarte, JUBARTE), mean=mean, rows=cur), open(a.baseline, "w"), indent=1)
        print(f"blessed {len(cur)} rows, mean J {mean:.2f} -> {a.baseline}")
        return 0

    base = json.load(open(a.baseline))
    print(f"{'id':60s} {'set':7s} {'stratum':9s} {'base':>6s} {'now':>6s} {'delta':>6s}")
    worst, regress = 0.0, []
    for r in rows:
        b = base["rows"].get(r["id"], {}).get("jaccard", 0.0); n = cur[r["id"]]["jaccard"]; d = n - b
        flag = "  <-- REGRESSION" if d < -a.max_drop else ""
        if d < -a.max_drop:
            regress.append(r["id"])
        worst = min(worst, d)
        print(f"{r['id'][:60]:60s} {r['set']:7s} {r['stratum']:9s} {b:6.1f} {n:6.1f} {d:+6.1f}{flag}")
    dm = mean - base["mean"]
    print(f"\nmean J: baseline {base['mean']:.2f} -> now {mean:.2f} ({dm:+.2f}); worst row {worst:+.1f}; regressions {len(regress)}")
    if regress or dm < -a.max_mean_drop:
        print("RESULT: REGRESSION — do not keep this change without naming every row above in the commit message.")
        return 1
    print("RESULT: OK")
    return 0

if __name__ == "__main__":
    sys.exit(main())

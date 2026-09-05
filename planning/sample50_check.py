#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Re-score the 50-fixture regression sample after every converter change.

Lives in <jubarte-redlines>/planning; expects ../../docxide-pdf and ../../neurotic_docx_bench
(fixture paths in sample50.tsv are absolute; the scorer binary path is derived from T).

    python3 sample50_check.py --bless      # record the current binary's scores as the baseline
    python3 sample50_check.py              # compare with the baseline; exit 1 on regression

Rules (plan.md, ground rules): a row that drops by more than --max-drop Jaccard points,
or a sample mean that drops by more than --max-mean-drop, is a regression. Rasters are
deleted by the scorer after each document; only the JSON survives.
"""
import argparse, json, os, shutil, statistics as st, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))          # <jubarte-redlines>/planning
JUBARTE = os.path.dirname(HERE)                             # <jubarte-redlines>
T = os.path.dirname(JUBARTE)                                # ~/temp/T: docxide-pdf, neurotic_docx_bench, jubarte-redlines

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--bless", action="store_true")
    ap.add_argument("--jubarte", default=os.path.join(JUBARTE, "target", "release", "jubarte"))
    ap.add_argument("--scorer", default=os.path.join(T, "neurotic_docx_bench", "src", "neurotic_docx_bench", "utils", "docxide-metrics", "target", "release", "docxide-metrics"))
    ap.add_argument("--sample", default=os.path.join(HERE, "sample50.tsv"))
    ap.add_argument("--baseline", default=os.path.join(HERE, "sample50_baseline.json"))
    ap.add_argument("--workers", type=int, default=4)
    ap.add_argument("--max-drop", type=float, default=1.0)
    ap.add_argument("--max-mean-drop", type=float, default=0.2)
    a = ap.parse_args()

    rows = []
    for line in open(a.sample):
        if line.startswith("#") or not line.strip():
            continue
        s, id_, docx, ref, _j, stratum = line.rstrip("\n").split("\t")
        rows.append(dict(set=s, id=id_, stratum=stratum,
                         docx=docx if os.path.isabs(docx) else os.path.join(HERE, docx),
                         ref=ref if os.path.isabs(ref) else os.path.join(HERE, ref)))

    work = tempfile.mkdtemp(prefix="sample50_")
    jobs, failed = [], []
    for r in rows:
        out = os.path.join(work, r["id"] + ".pdf")
        p = subprocess.run([a.jubarte, "convert", r["docx"], "-o", out, "--force"], capture_output=True, text=True)
        if p.returncode != 0 or not os.path.exists(out):
            failed.append(r["id"])
        jobs.append(dict(stem=r["id"], oracle=r["ref"], candidate=out))
    jobs_path, scores_path, scratch = (os.path.join(work, n) for n in ("jobs.json", "scores.json", "scratch"))
    os.makedirs(scratch, exist_ok=True)
    json.dump(jobs, open(jobs_path, "w"))
    subprocess.run([a.scorer, "--jobs", jobs_path, "--scratch", scratch, "--out", scores_path, "--workers", str(a.workers)], check=True)
    raw = json.load(open(scores_path))
    raw = raw if isinstance(raw, list) else list(raw.values())
    scores = {s["stem"]: s for s in raw}
    shutil.rmtree(work, ignore_errors=True)

    def pct(v):
        return 0.0 if v is None else (v * 100.0 if v <= 1.0 else v)
    cur = {r["id"]: {k: pct(scores.get(r["id"], {}).get(k)) for k in ("jaccard", "ssim", "text_boundary")} for r in rows}
    mean = st.mean(v["jaccard"] for v in cur.values())

    if a.bless or not os.path.exists(a.baseline):
        json.dump(dict(jubarte=a.jubarte, mean=mean, rows=cur), open(a.baseline, "w"), indent=1)
        print(f"blessed {len(cur)} rows, mean J {mean:.2f} -> {a.baseline}" + (f"  (convert failures: {failed})" if failed else ""))
        return 0

    base = json.load(open(a.baseline))
    print(f"{'id':60s} {'set':7s} {'stratum':9s} {'base':>6s} {'now':>6s} {'delta':>6s}")
    worst, regress = 0.0, []
    for r in rows:
        b = base["rows"].get(r["id"], {}).get("jaccard", 0.0); n = cur[r["id"]]["jaccard"]; d = n - b
        flag = "  <-- REGRESSION" if d < -a.max_drop else ("  (fail)" if r["id"] in failed else "")
        if d < -a.max_drop:
            regress.append(r["id"])
        worst = min(worst, d)
        print(f"{r['id'][:60]:60s} {r['set']:7s} {r['stratum']:9s} {b:6.1f} {n:6.1f} {d:+6.1f}{flag}")
    dm = mean - base["mean"]
    print(f"\nmean J: baseline {base['mean']:.2f} -> now {mean:.2f} ({dm:+.2f}); worst row {worst:+.1f}; regressions {len(regress)}; convert failures {len(failed)}")
    if regress or dm < -a.max_mean_drop or failed:
        print("RESULT: REGRESSION — do not keep this change without naming every row above in the commit message.")
        return 1
    print("RESULT: OK")
    return 0

if __name__ == "__main__":
    sys.exit(main())

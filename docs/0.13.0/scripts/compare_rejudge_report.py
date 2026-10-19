"""Score table for the compare experiment: original rejudge, control rerun, compare view.

Usage: python3 -I compare_rejudge_report.py CASES.json
"""
import json, sys, collections
ROOT = "/Users/arthrod/temp/T/redline-bench"
JOB = {"old": "muse-1-3-contributor-total", "new": "muse-1-3-contributor-total-jubarte-oct-18"}
M = "muse-spark-1.3-contributor"
cases = [c for c in json.load(open(sys.argv[1])) if c["category"] == "C-invisible-settle"]
load = lambda p: json.load(open(p))
tot = collections.Counter()
flips = []
print(f"{'build':5s} {'task':22s} {'orig':>6s} {'ctrl':>6s} {'cmp':>6s}  settle-items PASS (orig->cmp)")
for c in cases:
    t, b = c["task"], c["build"]
    o = load(f"{ROOT}/reproduce_out/rejudge/{JOB[b]}/{M}/{t}.json")
    k = load(f"{ROOT}/reproduce_out/rejudge/compare-experiment/control-{b}/{M}/{t}.json")
    f = load(f"{ROOT}/reproduce_out/rejudge/compare-experiment/compare-{b}/{M}/{t}.json")
    v = {n: {p["rubric_id"]: p for p in g["score"]["per_rubric"]} for n, g in (("o", o), ("k", k), ("f", f))}
    ids = {i["rubric_id"] for i in c["items"]}
    passed = lambda n: sum(v[n][i]["verdict"] == "PASS" for i in ids if i in v[n])
    print(f"{b:5s} {t:22s} {o['score']['weighted']:6.3f} {k['score']['weighted']:6.3f} {f['score']['weighted']:6.3f}"
          f"  {passed('o')}/{len(ids)} -> {passed('f')}/{len(ids)} (ctrl {passed('k')}/{len(ids)})")
    for n in ("o", "k", "f"):
        tot[n] += {"o": o, "k": k, "f": f}[n]["score"]["weighted"]
    for rid, p in v["f"].items():
        a, kk = v["o"].get(rid), v["k"].get(rid)
        if a and kk and a["verdict"] == kk["verdict"] and a["verdict"] != p["verdict"]:
            flips.append((b, t, rid, a["verdict"], p["verdict"], p["weight"], rid in ids, p["criteria"][:110], p["justification"][:200]))
n = len(cases)
print(f"\nmean over {n}: orig {tot['o']/n:.3f}  control {tot['k']/n:.3f}  compare {tot['f']/n:.3f}")
print("\nverdict flips where orig and control agree but compare differs:")
for x in sorted(flips, key=lambda x: (x[3], x[0], x[1])):
    print(f"  {x[0]} {x[1]} {x[2]} {x[3]}->{x[4]} w={x[5]} settle-item={x[6]} :: {x[7]}\n      judge: {x[8]}")

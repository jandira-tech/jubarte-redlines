"""For every FAILed rubric (Kilo judge) that quotes contract text, compare where the
quoted text sits in the task input vs the agent's deliverable. Flags losses where
the outcome text is right but the change is invisible (jubarte reject/accept leave
no trace).

Usage: python3 -I silent_settle_scan.py CASES.json OUT.json
"""
import json, os, re, sys
sys.path.insert(0, os.path.dirname(__file__))
from docx_phrase import locate

ROOT = "/Users/arthrod/temp/T/redline-bench"
QUOTE_RE = re.compile(r"[“\"]([^”\"]{18,})[”\"]")


def where(path, phrase):
    hits = locate(path, phrase)
    if not hits:
        return "absent"
    kinds = set()
    for view, states, _ in hits:
        if view == "current":
            kinds.add("current:" + ",".join(states))
        else:
            kinds.add("original:" + ",".join(states))
    return " | ".join(sorted(kinds))


rows = json.load(open(sys.argv[1]))
out = []
for r in rows:
    if r["empty"] or r["score"] is None:
        continue
    g = json.load(open(f"{ROOT}/reproduce_out/rejudge/{'muse-1-3-contributor-total' if r['build']=='old' else 'muse-1-3-contributor-total-jubarte-oct-18'}/muse-spark-1.3-contributor/{r['task']}.json"))
    inp = f"{ROOT}/.tasks/{r['task']}/environment/app/contract.docx"
    deliv = r["dir"] + "/artifacts/app/contract.docx"
    if not os.path.exists(deliv):
        continue
    for p in g["score"]["per_rubric"]:
        if p["verdict"] != "FAIL" or p["weight"] <= 0:
            continue
        crit = p["criteria"]
        verb = (re.match(r"\s*(\w+)", crit) or [None, ""])[1].lower()
        for ph in QUOTE_RE.findall(crit)[:2]:
            ph = ph.strip(" .,;:")
            if len(ph) < 18:
                continue
            wi, wo = where(inp, ph), where(deliv, ph)
            kind = None
            if wi.startswith("current:ins:") and "|" not in wi and wo == "absent":
                kind = "silent reject of their insertion"
            elif wi.startswith("original:del:") and "|" not in wi and wo == "absent":
                kind = "silent accept of their deletion"
            elif wi.startswith("original:del:") and "|" not in wi and wo.startswith("current:plain"):
                kind = "silent restore (reject of their deletion)"
            out.append({"build": r["build"], "task": r["task"], "trial": r["trial"], "score": r["score"],
                        "rubric_id": p["rubric_id"], "weight": p["weight"], "verb": verb,
                        "phrase": ph[:120], "input": wi, "output": wo, "kind": kind,
                        "criteria": crit, "justification": p["justification"],
                        "used_reject": bool(r["reject_calls"])})
json.dump(out, open(sys.argv[2], "w"), indent=1)
k = [x for x in out if x["kind"]]
print(len(out), "quoted FAIL phrases checked;", len(k), "invisible-settle losses")
for b in ("old", "new"):
    kb = [x for x in k if x["build"] == b]
    print(f"  {b}: {len(kb)} items in {len({x['task'] for x in kb})} trials, weight lost {sum(x['weight'] for x in kb)}")
    for x in kb:
        print(f"    {x['task']} w={x['weight']} {x['verb']:8s} {x['kind']}: {x['phrase'][:70]!r}")

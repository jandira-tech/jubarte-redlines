"""Per-trial table for both jubarte builds under the Kilo judge, with the
evidence needed to decide whether jubarte caused a lost score.

Usage: python3 -I jubarte_cause_cases.py OUT.json
"""
import glob, json, os, re, sys
sys.path.insert(0, os.path.dirname(__file__))
from jubarte_usage import load_calls

ROOT = "/Users/arthrod/temp/T/redline-bench"
JOBS = {"old": "muse-1-3-contributor-total", "new": "muse-1-3-contributor-total-jubarte-oct-18"}
MODEL = "muse-spark-1.3-contributor"
TASK_RE = re.compile(r"redline-s\d+-t\d+-g\d+[a-z]")
EDIT_RE = re.compile(r"jubarte (edit|add)\b")
ERR_RE = re.compile(r"^error: .*$", re.M)
CODE_RE = re.compile(r"^error: (?:plan refused: )?([A-Z][A-Z_]{4,})\b", re.M)
WRITE_RE = re.compile(r"-o\s+/app/contract\.docx")
SETTLE_RE = re.compile(r"^(accepted|rejected) r\d+", re.M)
# A silent reject: the judge finds the counterparty text gone, but no tracked change shows the walk-back.
ABSENT_RE = re.compile(r"\b(absent|does not appear|doesn't appear|not present|no longer (appears|present)|"
                       r"missing|removed|gone|vanish\w*|disappear\w*|nowhere)\b|appears anywhere", re.I)
NOTRACK_RE = re.compile(r"no (tracked|rejecting|visible|redline)|nothing (is |was )?rejected|untracked|"
                        r"not tracked|without (a |any )?(tracked|redline|visible)|silent\w*|no trace|"
                        r"rather than (a )?tracked|not shown as", re.I)
REJECT_USE_RE = re.compile(r'--reject\b|"reject"\s*:')


def mins(c, t0):
    return round((c["t_use"] - t0).total_seconds() / 60, 1) if c.get("t_use") and t0 else None


def trial_row(build, d):
    name = os.path.basename(d.rstrip("/"))
    task = TASK_RE.search(name).group(0)
    r = json.load(open(d + "result.json")) if os.path.exists(d + "result.json") else {}
    exc = (r.get("exception_info") or {}).get("exception_type")
    if exc == "CancelledError" or not r.get("finished_at"):
        return None
    rj = f"{ROOT}/reproduce_out/rejudge/{JOBS[build]}/{MODEL}/{task}.json"
    g = json.load(open(rj)) if os.path.exists(rj) else None
    calls, t0, t1 = load_calls(d)
    events, writes, rejects_written, accepts_written = [], [], 0, 0
    reject_calls = []
    for c in calls:
        blob = c["cmd"] or json.dumps(c["input"])
        if REJECT_USE_RE.search(blob) or REJECT_USE_RE.search(json.dumps(c["input"])):
            reject_calls.append(mins(c, t0))
    for c in calls:
        cmd, out = c["cmd"], c.get("out", "")
        if not EDIT_RE.search(cmd) or "--help" in cmd:
            continue
        errs = ERR_RE.findall(out)
        failed = bool(errs) or bool(re.search(r"Exit code [1-9]", out[:200]))
        dry = "--dry-run" in cmd
        is_write = bool(WRITE_RE.search(cmd)) and not dry
        ev = {"min": mins(c, t0), "dry": dry, "write": is_write, "failed": failed,
              "codes": CODE_RE.findall(out)[:3], "err": errs[0][:300] if errs else None}
        events.append(ev)
        if is_write and not failed:
            writes.append(ev["min"])
            s = SETTLE_RE.findall(out)
            rejects_written += s.count("rejected")
            accepts_written += s.count("accepted")
    silent = []
    if g and not g.get("gate_failure"):
        for p in g["score"]["per_rubric"]:
            j = p.get("justification") or ""
            if p["verdict"] == "FAIL" and p["weight"] > 0 and ABSENT_RE.search(j) and NOTRACK_RE.search(j):
                silent.append({"rubric_id": p["rubric_id"], "weight": p["weight"],
                               "criteria": p["criteria"], "justification": p["justification"]})
    gate_reason = None
    gp = d + "verifier/grade.json"
    if os.path.exists(gp):
        gg = json.load(open(gp))
        if not gg.get("_synthetic_for_rejudge"):
            gate_reason = (gg.get("gate") or {}).get("reason")
    return {
        "build": build, "task": task, "trial": name, "dir": d.rstrip("/"),
        "score": g["score"]["weighted"] if g else None,
        "total_weight": (g["score"].get("total_weight") if g else None),
        "empty": bool(g and g.get("gate_failure")), "gate_reason": gate_reason,
        "exception": exc, "agent_minutes": round((t1 - t0).total_seconds() / 60, 1) if t0 and t1 else None,
        "edit_events": events, "first_edit_min": events[0]["min"] if events else None,
        "writes_ok": writes, "rejects_written": rejects_written, "accepts_written": accepts_written,
        "refusals": [e for e in events if e["failed"]],
        "reject_calls": reject_calls,
        "silent_reject_fails": silent if reject_calls and writes else [],
        "silent_like_fails_no_reject": silent if not (reject_calls and writes) else [],
    }


rows = []
for build, job in JOBS.items():
    for d in sorted(glob.glob(f"{ROOT}/reproduce_out/jobs/{job}/redline-*/")):
        row = trial_row(build, d)
        if row:
            rows.append(row)
json.dump(rows, open(sys.argv[1], "w"), indent=1)
print(len(rows), "rows")

"""Side-by-side statistics for the two jubarte runs (old build vs new build).

Usage: python3 -I run_stats.py
"""
import glob, json, os, re, statistics as st, sys
sys.path.insert(0, os.path.dirname(__file__))
from jubarte_usage import analyze_trial

ROOT = "/Users/arthrod/temp/T/redline-bench"
JOBS = {"old": "muse-1-3-contributor-total", "new": "muse-1-3-contributor-total-jubarte-oct-18"}
TASK_RE = re.compile(r"redline-s\d+-t\d+-g\d+[a-z]")


def ts(s):
    import datetime as dt
    return dt.datetime.fromisoformat(s.replace("Z", "+00:00"))


def session_stats(d):
    """Assistant turns, turns that hit the 32k output cap, and thinking-only capped turns."""
    sess = sorted(glob.glob(f"{d}/agent/sessions/projects/*/*.jsonl"), key=os.path.getsize, reverse=True)
    turns = capped = thinking_only = 0
    seen = set()
    if not sess:
        return None
    for line in open(sess[0], errors="replace"):
        if '"assistant"' not in line:
            continue
        try:
            o = json.loads(line)
        except json.JSONDecodeError:
            continue
        m = o.get("message") or {}
        if m.get("role") != "assistant" or m.get("id") in seen:
            continue
        seen.add(m.get("id"))
        turns += 1
        out = (m.get("usage") or {}).get("output_tokens") or 0
        if out >= 32000:
            capped += 1
            kinds = {p.get("type") for p in m.get("content") or [] if isinstance(p, dict)}
            if kinds <= {"thinking", "redacted_thinking"}:
                thinking_only += 1
    return turns, capped, thinking_only


def rows(build):
    out = []
    for d in sorted(glob.glob(f"{ROOT}/reproduce_out/jobs/{JOBS[build]}/redline-*/")):
        r = json.load(open(d + "result.json")) if os.path.exists(d + "result.json") else {}
        exc = (r.get("exception_info") or {}).get("exception_type")
        if not r.get("finished_at") or exc == "CancelledError":
            continue
        task = TASK_RE.search(d).group(0)
        a = r.get("agent_result") or {}
        ae = r.get("agent_execution") or {}
        g = f"{ROOT}/reproduce_out/rejudge/{JOBS[build]}/muse-spark-1.3-contributor/{task}.json"
        gj = json.load(open(g)) if os.path.exists(g) else None
        t = analyze_trial(d)
        ss = session_stats(d.rstrip("/"))
        out.append({
            "task": task, "score": gj["score"]["weighted"] if gj else None,
            "empty": bool(gj and gj.get("gate_failure")),
            "timeout": exc == "AgentTimeoutError",
            "in": a.get("n_input_tokens"), "cache": a.get("n_cache_tokens"), "out": a.get("n_output_tokens"),
            "cost": a.get("cost_usd"),
            "agent_min": (ts(ae["finished_at"]) - ts(ae["started_at"])).total_seconds() / 60 if ae.get("finished_at") else None,
            "tool_calls": t["tool_calls"], "jub_calls": t["jub_calls"], "edit_attempts": t["edit_attempts"],
            "edit_fail": t["edit_fail"], "refusals": len(t["refusals"]), "help_reads": t["help_reads"],
            "jub_min": t["jub_secs"] / 60, "first_edit_min": (t["first_edit_secs"] / 60) if t["first_edit_secs"] is not None else None,
            "first_write_min": (t["first_write_secs"] / 60) if t["first_write_secs"] is not None else None,
            "turns": ss[0] if ss else None, "capped": ss[1] if ss else None, "thinking_only": ss[2] if ss else None,
        })
    return out


def agg(R, k, f=st.mean):
    v = [x[k] for x in R if x[k] is not None]
    return f(v) if v else float("nan")


R = {b: rows(b) for b in JOBS}
paired = sorted(set(x["task"] for x in R["old"]) & set(x["task"] for x in R["new"]))
P = {b: [x for x in R[b] if x["task"] in paired] for b in JOBS}

lines = [
    ("trials finished (excl. cancelled)", lambda R: len(R), "{:.0f}"),
    ("Kilo Haiku score, mean", lambda R: agg(R, "score"), "{:.4f}"),
    ("empty deliverable (gate 0)", lambda R: sum(x["empty"] for x in R), "{:.0f}"),
    ("agent timeouts (60 min)", lambda R: sum(x["timeout"] for x in R), "{:.0f}"),
    ("agent minutes, mean", lambda R: agg(R, "agent_min"), "{:.1f}"),
    ("input tokens/trial, mean (M)", lambda R: agg(R, "in") / 1e6, "{:.2f}"),
    ("  of which cache reads (M)", lambda R: agg(R, "cache") / 1e6, "{:.2f}"),
    ("output tokens/trial, mean (k)", lambda R: agg(R, "out") / 1e3, "{:.0f}"),
    ("output tokens/trial, median (k)", lambda R: agg(R, "out", st.median) / 1e3, "{:.0f}"),
    ("output tokens, total (M)", lambda R: sum(x["out"] or 0 for x in R) / 1e6, "{:.1f}"),
    ("cost/trial, mean (USD, Claude Code est.)", lambda R: agg(R, "cost"), "{:.2f}"),
    ("cost, total (USD, Claude Code est.)", lambda R: sum(x["cost"] or 0 for x in R), "{:.0f}"),
    ("assistant turns/trial, mean", lambda R: agg(R, "turns"), "{:.1f}"),
    ("turns at the 32k output cap/trial", lambda R: agg(R, "capped"), "{:.2f}"),
    ("  trials with >=1 capped turn", lambda R: sum((x["capped"] or 0) > 0 for x in R), "{:.0f}"),
    ("  thinking-only capped turns/trial", lambda R: agg(R, "thinking_only"), "{:.2f}"),
    ("tool calls/trial, mean", lambda R: agg(R, "tool_calls"), "{:.1f}"),
    ("jubarte calls/trial, mean", lambda R: agg(R, "jub_calls"), "{:.1f}"),
    ("jubarte runtime/trial, mean (min)", lambda R: agg(R, "jub_min"), "{:.2f}"),
    ("help/docs reads/trial, mean", lambda R: agg(R, "help_reads"), "{:.1f}"),
    ("edit/add attempts/trial, mean", lambda R: agg(R, "edit_attempts"), "{:.1f}"),
    ("edit/add refused or failed, %", lambda R: 100 * sum(x["edit_fail"] for x in R) / max(1, sum(x["edit_attempts"] for x in R)), "{:.1f}"),
    ("trials with >=1 refusal code", lambda R: sum(x["refusals"] > 0 for x in R), "{:.0f}"),
    ("first edit attempt, median (min)", lambda R: agg(R, "first_edit_min", st.median), "{:.1f}"),
    ("first write to contract.docx, median (min)", lambda R: agg(R, "first_write_min", st.median), "{:.1f}"),
    ("never attempted an edit", lambda R: sum(x["edit_attempts"] == 0 for x in R), "{:.0f}"),
]
print(f"{'metric':44s} {'old all':>9s} {'new all':>9s} | {'old pair':>9s} {'new pair':>9s}  (paired n={len(paired)})")
for name, fn, fmt in lines:
    vals = [fn(R["old"]), fn(R["new"]), fn(P["old"]), fn(P["new"])]
    print(f"{name:44s} " + " ".join(f"{fmt.format(v):>9s}" for v in vals[:2]) + " | " + " ".join(f"{fmt.format(v):>9s}" for v in vals[2:]))
json.dump(R, open("/tmp/run_stats_rows.json", "w"))

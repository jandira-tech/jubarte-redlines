"""Summarize how agents drive jubarte in harbor claude-code trial logs.

Usage: python3 -I jubarte_usage.py JOB_DIR [JOB_DIR ...] [--json OUT]

Per trial it reads the timestamped Claude Code session log (falling back to
the stream-json agent log), pairs every tool call with its result, and records
which jubarte subcommands ran, their exit codes and refusal lines, how long
jubarte itself took versus the model, and when the first edit was attempted.
The verifier's own counts (redlines, total_revisions, comments_added) decide
whether anything actually landed in the deliverable.

Each trial lands in one bucket:
  A  never attempted a jubarte edit/add      -> model-side (budget spent reading/thinking)
  B  attempted, but nothing landed           -> jubarte refusals or plan never applied
  C  changes landed in the deliverable
  U  finished but the verifier produced no reward (judge failure); excluded
  X  cancelled because harbor stopped mid-trial; excluded
"""

import collections
import datetime as dt
import glob
import json
import os
import re
import statistics
import sys

JUB = re.compile(r"(?:/skills/contract-redliner/bin/|/app/|\./)jubarte\s+(?P<sub>[a-z-]+)")
EXIT = re.compile(r"Exit code (\d+)")
ERRLINE = re.compile(r"^error: .*$", re.M)
REFUSAL = re.compile(r"^error: (?:plan refused: )?([A-Z][A-Z_]{4,})\b", re.M)
BYPASS = re.compile(r"python-docx|from docx import|import docx\b|docx\.Document\(|unzip\s|zipfile\.ZipFile|"
                    r"word/document\.xml|libreoffice|soffice|pandoc")
WRITE_OUT = re.compile(r"-o\s+/app/contract\.docx")


def ts(s):
    return dt.datetime.fromisoformat(s.replace("Z", "+00:00")) if s else None


def result_text(c):
    if isinstance(c, str):
        return c
    if isinstance(c, list):
        return "\n".join(x.get("text", "") for x in c if isinstance(x, dict))
    return ""


def norm_error(msg):
    msg = re.sub(r"'[^']*'|\"[^\"]*\"|`[^`]*`", "<s>", msg)
    msg = re.sub(r"\bp\d+\b|\bop-\d+\b|\b[rc]\d+(\+\d+)?\b|\d+", "#", msg)
    return msg[:150]


def load_calls(trial_dir):
    sess = sorted(glob.glob(os.path.join(trial_dir, "agent", "sessions", "projects", "*", "*.jsonl")),
                  key=os.path.getsize, reverse=True)
    src = sess[0] if sess else os.path.join(trial_dir, "agent", "claude-code.txt")
    if not os.path.exists(src):
        return [], None, None
    calls, order, first_ts, last_ts = {}, [], None, None
    for line in open(src, errors="replace"):
        if '"tool_use"' not in line and '"tool_result"' not in line and '"timestamp"' not in line:
            continue
        try:
            o = json.loads(line)
        except json.JSONDecodeError:
            continue
        t = ts(o.get("timestamp"))
        if t:
            first_ts = first_ts or t
            last_ts = t
        for part in (o.get("message") or {}).get("content") or []:
            if not isinstance(part, dict):
                continue
            if part.get("type") == "tool_use":
                inp = part.get("input") or {}
                calls[part["id"]] = {"tool": part.get("name"), "cmd": inp.get("command") or "",
                                     "input": inp, "t_use": t}
                order.append(part["id"])
            elif part.get("type") == "tool_result" and part.get("tool_use_id") in calls:
                c = calls[part["tool_use_id"]]
                c["out"] = result_text(part.get("content"))
                c["is_error"] = bool(part.get("is_error"))
                c["t_res"] = t
    return [calls[i] for i in order], first_ts, last_ts


def analyze_trial(d):
    name = os.path.basename(d.rstrip("/"))
    calls, t0, t1 = load_calls(d)
    t = {"trial": name, "tool_calls": len(calls), "jub_calls": 0, "jub_fail": 0, "edit_attempts": 0,
         "edit_fail": 0, "refusals": [], "errors": [], "bypass": 0, "help_reads": 0,
         "jub_secs": 0.0, "tool_secs": 0.0, "span_secs": (t1 - t0).total_seconds() if t0 and t1 else None,
         "first_edit_secs": None, "first_write_secs": None, "last_error": None,
         "reward": None, "redlines": None, "revisions": None, "comments": None, "exception": None,
         "finished": False, "subs": collections.Counter(), "sub_fail": collections.Counter()}
    rf = os.path.join(d, "result.json")
    if os.path.exists(rf):
        r = json.load(open(rf))
        t["finished"] = bool(r.get("finished_at"))
        v = (r.get("verifier_result") or {}).get("rewards") or {}
        t["reward"], t["redlines"] = v.get("reward"), v.get("redlines")
        t["revisions"], t["comments"] = v.get("total_revisions"), v.get("comments_added")
        e = r.get("exception_info")
        t["exception"] = e.get("exception_type") if e else None
    for c in calls:
        cmd, out = c["cmd"], c.get("out", "")
        dur = (c["t_res"] - c["t_use"]).total_seconds() if c.get("t_res") and c.get("t_use") else 0.0
        t["tool_secs"] += dur
        blob = cmd or json.dumps(c["input"])
        if "jubarte-help-all" in blob or "--help" in cmd:
            t["help_reads"] += 1
        if cmd and BYPASS.search(cmd) and "contract" in cmd and "jubarte" not in cmd.split(BYPASS.search(cmd).group(0))[0][-40:]:
            t["bypass"] += 1
        subs = [m.group("sub") for m in JUB.finditer(cmd)]
        if not subs:
            continue
        t["jub_calls"] += len(subs)
        t["jub_secs"] += dur
        t["subs"].update(subs)
        m = EXIT.search(out[:200])
        code = int(m.group(1)) if m else (1 if c.get("is_error") else 0)
        errs = ERRLINE.findall(out)
        failed = code != 0 or bool(errs)
        is_edit = bool({"edit", "add"} & set(subs)) and "--help" not in cmd and "help" not in subs
        rel = (c["t_use"] - t0).total_seconds() if c.get("t_use") and t0 else None
        if is_edit:
            t["edit_attempts"] += 1
            if t["first_edit_secs"] is None:
                t["first_edit_secs"] = rel
            if WRITE_OUT.search(cmd) and not failed and t["first_write_secs"] is None:
                t["first_write_secs"] = rel
        if failed:
            t["jub_fail"] += 1
            t["sub_fail"][subs[-1]] += 1
            if is_edit:
                t["edit_fail"] += 1
            t["errors"] += [norm_error(e) for e in errs[:3]]
            t["refusals"] += REFUSAL.findall(out)
            if errs:
                t["last_error"] = norm_error(errs[-1])
    landed = bool(t["redlines"] or t["revisions"] or t["comments"])
    t["bucket"] = "C" if landed else ("B" if t["edit_attempts"] else "A")
    if t["finished"] and os.path.exists(rf) and json.load(open(rf)).get("verifier_result") is None:
        t["bucket"] = "X" if t["exception"] == "CancelledError" else "U"
    return t


def corr(xs, ys):
    if len(xs) < 3 or statistics.pstdev(xs) == 0 or statistics.pstdev(ys) == 0:
        return float("nan")
    return statistics.correlation(xs, ys)


def med(xs):
    xs = [x for x in xs if x is not None]
    return f"{statistics.median(xs) / 60:.1f}m" if xs else "-"


def print_report(job, trials):
    print(f"\n######## {os.path.basename(job)}  ({len(trials)} trials with agent logs, "
          f"{sum(t['finished'] for t in trials)} finished)")
    subs, subf, errs, refs = collections.Counter(), collections.Counter(), collections.Counter(), collections.Counter()
    for t in trials:
        subs.update(t["subs"]); subf.update(t["sub_fail"]); errs.update(t["errors"]); refs.update(t["refusals"])
    jc, jf = sum(t["jub_calls"] for t in trials), sum(t["jub_fail"] for t in trials)
    ea, ef = sum(t["edit_attempts"] for t in trials), sum(t["edit_fail"] for t in trials)
    print(f"jubarte invocations={jc}, failing tool calls={jf}; edit/add attempts={ea}, refused/failed={ef} "
          f"({(ef / ea * 100 if ea else 0):.1f}%)")
    print("subcommands: " + ", ".join(f"{k}={v}" + (f" (fail {subf[k]})" if subf[k] else "") for k, v in subs.most_common(12)))
    print("refusal/error codes: " + (", ".join(f"{k}={v}" for k, v in refs.most_common()) or "none"))
    print("top error lines:")
    for k, v in errs.most_common(12):
        print(f"  {v:4d}  {k}")
    span = [t["span_secs"] for t in trials if t["span_secs"]]
    js = sum(t["jub_secs"] for t in trials); tot = sum(span)
    print(f"wall time: jubarte calls {js / 60:.1f}m of {tot / 60:.1f}m total agent time "
          f"({(js / tot * 100 if tot else 0):.1f}%); all tool execution {sum(t['tool_secs'] for t in trials) / 60:.1f}m")
    print(f"median time to first edit attempt {med([t['first_edit_secs'] for t in trials])}, "
          f"to first successful write of /app/contract.docx {med([t['first_write_secs'] for t in trials])}")
    print(f"trials touching the docx outside jubarte: {sum(1 for t in trials if t['bypass'])}")
    fin = [t for t in trials if t["finished"]]
    if not fin:
        return
    for b, label in [("A", "never attempted an edit"), ("B", "attempted, nothing landed"), ("C", "changes landed"),
                     ("U", "UNGRADED: verifier failed, excluded from rewards"),
                     ("X", "CANCELLED: harbor stopped mid-trial, excluded")]:
        g = [t for t in fin if t["bucket"] == b]
        if not g:
            continue
        rw = [t["reward"] for t in g if t["reward"] is not None]
        print(f"  bucket {b} ({label}): n={len(g)} timeouts={sum(1 for t in g if t['exception'] == 'AgentTimeoutError')} "
              f"mean_reward={(statistics.mean(rw) if rw else 0):.3f} median_tool_calls={statistics.median(t['tool_calls'] for t in g):.0f}")
        if b == "B":
            for t in g:
                print(f"      {t['trial']}: attempts={t['edit_attempts']} failed={t['edit_fail']} last_error={t['last_error']}")
    rw = [t for t in fin if t["reward"] is not None and t["bucket"] not in ("U", "X")]
    if len(rw) >= 3:
        R = [t["reward"] for t in rw]
        print("  corr(reward, x): " + ", ".join(f"{k}={corr([t[k] for t in rw], R):+.2f}"
                                                for k in ["jub_fail", "edit_fail", "edit_attempts", "tool_calls", "help_reads"]))
        c = [t for t in rw if t["bucket"] == "C"]
        if len(c) >= 3:
            print("  within bucket C, corr(reward, x): " + ", ".join(
                f"{k}={corr([t[k] for t in c], [t['reward'] for t in c]):+.2f}" for k in ["jub_fail", "edit_fail", "redlines"]))
    turn = collections.defaultdict(list)
    for t in fin:
        if t["bucket"] in ("U", "X"):
            continue
        m = re.search(r"-t(\d)-", t["trial"])
        turn[m.group(1) if m else "?"].append(t)
    print("  by turn: " + "; ".join(
        f"t{k}: n={len(v)} A={sum(t['bucket'] == 'A' for t in v)} B={sum(t['bucket'] == 'B' for t in v)} "
        f"C={sum(t['bucket'] == 'C' for t in v)} reward={statistics.mean(t['reward'] or 0 for t in v):.3f}"
        for k, v in sorted(turn.items())))


def main(argv):
    out_json = None
    if "--json" in argv:
        i = argv.index("--json")
        out_json = argv[i + 1]
        argv = argv[:i] + argv[i + 2:]
    report = {}
    for job in argv:
        job = job.rstrip("/")
        trials = [analyze_trial(d) for d in sorted(glob.glob(f"{job}/*/"))
                  if os.path.exists(os.path.join(d, "agent"))]
        report[job] = trials
        print_report(job, trials)
    if out_json:
        json.dump(report, open(out_json, "w"), indent=1, default=lambda x: dict(x))


if __name__ == "__main__":
    main(sys.argv[1:])

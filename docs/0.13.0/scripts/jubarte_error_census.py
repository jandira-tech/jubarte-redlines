"""Census of every jubarte error/warning/validate finding in trial logs, clustered.

Usage: python3 -I jubarte_error_census.py OUT.json JOB_DIR [JOB_DIR ...]
"""
import collections, glob, json, os, re, sys
sys.path.insert(0, os.path.dirname(__file__))
from jubarte_usage import load_calls, JUB, EXIT, norm_error

LINE = re.compile(r"^(?:error|warning|Error|Warning)[: ].*$|^.*(?:panicked|RUST_BACKTRACE|internal error|stack backtrace).*$", re.M)
CODE = re.compile(r"\b([A-Z][A-Z0-9]+(?:_[A-Z0-9]+)+)\b")
rows = []
for job in sys.argv[2:]:
    for d in sorted(glob.glob(f"{job}/redline-*/")):
        calls, t0, _ = load_calls(d)
        for c in calls:
            cmd, out = c["cmd"], c.get("out", "") or ""
            subs = [m.group("sub") for m in JUB.finditer(cmd)]
            if not subs:
                continue
            m = EXIT.search(out[:200])
            code = int(m.group(1)) if m else (1 if c.get("is_error") else 0)
            lines = LINE.findall(out)
            if "validate" in subs:
                lines += [l for l in out.splitlines() if CODE.search(l) and ("finding" in l.lower() or re.match(r"\s*[-*]?\s*\[?(error|warn|info)", l, re.I) or re.match(r"\s*[A-Z][A-Z_]{5,}", l))]
            dur = (c["t_res"] - c["t_use"]).total_seconds() if c.get("t_res") and c.get("t_use") else None
            rows.append({"job": os.path.basename(job), "trial": os.path.basename(d.rstrip("/")), "subs": subs,
                         "exit": code, "dur": dur, "min": round((c["t_use"] - t0).total_seconds() / 60, 1) if c.get("t_use") and t0 else None,
                         "lines": lines[:12], "cmd": cmd[:600], "out_head": out[:1500] if (code or lines) else ""})
json.dump(rows, open(sys.argv[1], "w"))
clusters = collections.defaultdict(list)
for r in rows:
    for l in r["lines"]:
        cm = CODE.search(l)
        key = (r["subs"][-1], cm.group(1) if cm else "", norm_error(l)[:110])
        clusters[key].append(r)
    if r["exit"] not in (0, 1, 2) or (r["exit"] and not r["lines"]):
        clusters[(r["subs"][-1], f"EXIT{r['exit']}-nolines" if not r["lines"] else f"EXIT{r['exit']}", "")].append(r)
for job in sorted({r["job"] for r in rows}):
    print(f"\n######## {job}: {sum(r['job']==job for r in rows)} jubarte calls")
    items = [(k, [r for r in v if r["job"] == job]) for k, v in clusters.items()]
    for k, v in sorted(items, key=lambda kv: -len(kv[1])):
        if not v:
            continue
        print(f"{len(v):5d} calls {len({r['trial'] for r in v}):4d} trials | {k[0]:10s} {k[1]:30s} | {k[2]}")

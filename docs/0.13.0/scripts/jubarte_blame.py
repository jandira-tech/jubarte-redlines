"""Pick the trials where jubarte caused a lost score, from jubarte_cause_cases.py
and silent_settle_scan.py output, and export the evidence.

Usage: python3 -I jubarte_blame.py CASES.json SILENT.json EXPORT_DIR
"""
import collections, json, os, re, shutil, sys

ROOT = "/Users/arthrod/temp/T/redline-bench"
LATE = 40
rows = json.load(open(sys.argv[1]))
silent = json.load(open(sys.argv[2]))
dest = sys.argv[3]

# Right outcome, invisible change: the rubric wanted the text gone (or restored) and it is,
# but jubarte's reject/accept left no tracked change, so the judge cannot see it.
RIGHT = {
    "silent reject of their insertion": {"rejects", "deletes", "removes", "strikes"},
    "silent accept of their deletion": {"deletes", "removes", "strikes"},
    "silent restore (reject of their deletion)": {"rejects", "restores", "reinstates"},
}
invisible = collections.defaultdict(list)
for s in silent:
    if s["kind"] and s["verb"] in RIGHT.get(s["kind"], set()):
        # "Rejects ... deletion" must pair with a restore, "Rejects ... insertion" with a reject
        c = s["criteria"].lower()
        if s["verb"] == "rejects" and s["kind"].startswith("silent reject") and "deletion" in c and "insertion" not in c:
            continue
        if s["verb"] == "rejects" and s["kind"].startswith("silent restore") and "insertion" in c and "deletion" not in c:
            continue
        invisible[(s["build"], s["task"])].append(s)

cases = []
for r in rows:
    key = (r["build"], r["task"])
    ev = r["edit_events"]
    clean_dry = [e for e in ev if e["dry"] and not e["failed"]]
    if r["empty"] and r["refusals"]:
        last = ev[-1]
        blocked = not clean_dry or (last["failed"] and not r["writes_ok"])
        if blocked:
            late = r["first_edit_min"] is not None and r["first_edit_min"] >= LATE
            cases.append({**r, "category": "B-final-blow-late-start" if late else "A-blocked-with-time-left",
                          "why": (f"first edit attempt at {r['first_edit_min']} min was refused; "
                                  if late else f"started editing at {r['first_edit_min']} min, ")
                                 + f"{len(r['refusals'])} refusal(s), last at {r['refusals'][-1]['min']} min "
                                 f"({(r['refusals'][-1]['codes'] or ['?'])[0]}); nothing landed, score 0"})
    if key in invisible:
        items = invisible[key]
        cases.append({**r, "category": "C-invisible-settle",
                      "lost_weight": sum(i["weight"] for i in items), "items": items,
                      "why": f"{len(items)} rubric item(s), weight {sum(i['weight'] for i in items)} of "
                             f"{r['total_weight']}, failed although the text outcome was right: jubarte's "
                             f"reject/accept removed the counterparty change with no tracked trace"})

os.makedirs(dest, exist_ok=True)
for c in cases:
    d = os.path.join(dest, c["category"], f"{c['build']}-build__{c['task']}")
    os.makedirs(d, exist_ok=True)
    src = c["dir"]
    for a, b in ((f"{src}/artifacts/app/contract.docx", "agent_output_redline.docx"),
                 (f"{ROOT}/.tasks/{c['task']}/environment/app/contract.docx", "task_input.docx")):
        if os.path.exists(a):
            shutil.copy2(a, os.path.join(d, b))
    job = "muse-1-3-contributor-total" if c["build"] == "old" else "muse-1-3-contributor-total-jubarte-oct-18"
    shutil.copy2(f"{ROOT}/reproduce_out/rejudge/{job}/muse-spark-1.3-contributor/{c['task']}.json",
                 os.path.join(d, "haiku_grade.json"))
    with open(os.path.join(d, "README.md"), "w") as f:
        f.write(f"# {c['task']} ({c['build']} jubarte build)\n\n")
        f.write(f"- category: {c['category']}\n- trial dir: `{src}`\n- Haiku 5.5 score: {c['score']:.4f}\n")
        f.write(f"- agent exception: {c['exception']}\n- first edit attempt: {c['first_edit_min']} min\n")
        f.write(f"- successful writes to /app/contract.docx at: {c['writes_ok'] or 'none'}\n\n")
        f.write(f"## Why jubarte\n\n{c['why']}.\n\n")
        if c.get("items"):
            f.write("## Rubric items lost to invisible settles\n\n")
            for i in c["items"]:
                f.write(f"- **w={i['weight']}** ({i['kind']}) — {i['criteria'].strip()}\n"
                        f"  - in task input: `{i['input']}`; in agent output: `{i['output']}`\n"
                        f"  - judge: {i['justification']}\n")
            f.write("\n")
        f.write("## jubarte edit/add timeline (minutes from agent start)\n\n")
        for e in c["edit_events"]:
            mode = "write" if e["write"] else ("dry-run" if e["dry"] else "other")
            f.write(f"- {e['min']} {mode} {'FAILED ' + ','.join(e['codes']) if e['failed'] else 'ok'}"
                    + (f"\n  - `{e['err'][:280]}`" if e.get("err") else "") + "\n")
json.dump([{k: v for k, v in c.items() if k not in ("edit_events",)} for c in cases],
          open(os.path.join(dest, "cases.json"), "w"), indent=1)
cnt = collections.Counter((c["build"], c["category"]) for c in cases)
for k in sorted(cnt):
    print(k, cnt[k])
for c in sorted(cases, key=lambda c: (c["category"], c["build"], c["task"])):
    print(f"{c['category']:26s} {c['build']} {c['task']} score={c['score']:.3f} {c.get('lost_weight', '')} :: {c['why'][:150]}")

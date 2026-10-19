"""Regenerate each invisible-settle deliverable as `jubarte compare input output`, render it
the way the verifier does, and lay out fake trial dirs that redlinebench-rejudge can grade.
Also lays out a control copy with the original annotated view, to measure judge noise.

Usage: uv run --with python-docx --with lxml python -I compare_rejudge.py CASES.json OUT_DIR
"""
import json, os, shutil, subprocess, sys

ROOT = "/Users/arthrod/temp/T/redline-bench"
cases = [c for c in json.load(open(sys.argv[1])) if c["category"] == "C-invisible-settle"]
out = sys.argv[2]
for c in cases:
    trial = c["dir"].rstrip("/")
    name = os.path.basename(trial)
    sys.path.insert(0, f"{ROOT}/.tasks/{c['task']}/tests")
    from annotated_view import render_inline
    if os.path.exists(f"{trial}/verifier/grade.json"):
        grade = json.load(open(f"{trial}/verifier/grade.json"))
    else:  # in-sandbox judge failed; take the rubric set from the task itself
        t = json.load(open(f"{ROOT}/.tasks/{c['task']}/tests/rubrics.json"))
        grade = {k: t[k] for k in ("task_id", "scenario_id", "side", "level", "author")}
        grade |= {"gate": {"passed": True}, "score": {"per_rubric": [
            {"rubric_id": r["id"], "criteria": r["criteria"], "weight": r["weight"], "category": r.get("category")}
            for r in t["rubrics"]]}}
    fresh_dir = f"{out}/jobs/compare-{c['build']}/{name}"
    ctrl_dir = f"{out}/jobs/control-{c['build']}/{name}"
    for d in (fresh_dir, ctrl_dir):
        os.makedirs(f"{d}/verifier", exist_ok=True)
        shutil.copy2(f"{trial}/result.json", f"{d}/result.json")
        json.dump(grade, open(f"{d}/verifier/grade.json", "w"))
    shutil.copy2(f"{trial}/verifier/annotated_view.md", f"{ctrl_dir}/verifier/annotated_view.md")
    fresh = f"{fresh_dir}/fresh.docx"
    subprocess.run([os.path.expanduser("~/.local/bin/jubarte"), "compare",
                    f"{ROOT}/.tasks/{c['task']}/environment/app/contract.docx",
                    f"{trial}/artifacts/app/contract.docx",
                    "-a", grade["author"], "-o", fresh, "--force"], check=True, capture_output=True)
    view = render_inline(fresh)
    open(f"{fresh_dir}/verifier/annotated_view.md", "w").write(view)
    sys.path.pop(0)
    for m in [k for k in sys.modules if k == "annotated_view" or k.startswith("redline_engine")]:
        del sys.modules[m]
    orig_len = os.path.getsize(f"{trial}/verifier/annotated_view.md")
    print(f"{c['build']} {c['task']} author={grade['author']!r} view chars orig={orig_len} fresh={len(view)}")

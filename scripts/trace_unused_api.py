#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

# SPDX-License-Identifier: AGPL-3.0-only

"""Where did each unused public item come from, and has it shipped?

Reads docs/analysis/UNUSED_PUBLIC_API.md rows (sections A, B, C), then for each item finds
  * the first commit that added its name to the defining file (git log -S, needs full history), and
  * the earliest release whose docs/api/jubarte-vX.api.txt snapshot lists it.
Writes docs/analysis/unused-api-provenance.json for the hand-written report to build on.

Usage: trace_unused_api.py <repo-root>
"""
import json, re, subprocess, sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(sys.argv[1]).resolve()
rep = (ROOT / "docs/analysis/UNUSED_PUBLIC_API.md").read_text()
rows = []
for sec, title in (("A", "## A."), ("B", "## B."), ("C", "## C.")):
    body = rep.split(title)[1].split("\n## ")[0]
    for line in body.splitlines():
        m = re.match(r"\| `([^`]+)` \| ([\w ]+) \| ([^ :]+):(\d+) \|", line)
        if m: rows.append({"section": sec, "qual": m.group(1), "kind": m.group(2).strip(), "file": m.group(3), "line": int(m.group(4))})

snaps = sorted((ROOT / "docs/api").glob("jubarte-v*.api.txt"), key=lambda p: [int(x) for x in re.findall(r"\d+", p.name)])
snap_text = {p.name[len("jubarte-"):-len(".api.txt")]: p.read_text() for p in snaps}

def modpath(file):
    parts = list(Path(file).relative_to("src").with_suffix("").parts)
    if parts == ["lib"]: return []
    if parts[-1] == "mod": parts = parts[:-1]
    return parts

def api_key(r):
    return "::".join(["jubarte", *modpath(r["file"]), *r["qual"].split("::")])

def first_release(r):
    key = api_key(r)
    rx = re.compile(r"\t" + re.escape(key) + r"(?:[ (<:]|$)", re.M)
    for ver, t in snap_text.items():
        if rx.search(t): return ver
    return None

def first_commit(r):
    name = r["qual"].split("::")[-1]
    out = subprocess.run(["git", "-C", str(ROOT), "log", "--reverse", "--format=%h\t%ad\t%s", "--date=short", f"-S{name}", "--", r["file"]],
                         capture_output=True, text=True).stdout.strip().splitlines()
    if not out: return None
    h, d, s = out[0].split("\t", 2)
    return {"hash": h, "date": d, "subject": s}

def work(r):
    return {**r, "first_release": first_release(r), "origin": first_commit(r)}

with ThreadPoolExecutor(8) as ex: res = list(ex.map(work, rows))
(ROOT / "docs/analysis/unused-api-provenance.json").write_text(json.dumps({"snapshots": list(snap_text), "items": res}, indent=1) + "\n")
from collections import Counter
print(len(res), Counter(r["first_release"] or "unreleased" for r in res))

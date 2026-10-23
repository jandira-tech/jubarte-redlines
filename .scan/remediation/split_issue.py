import json, sys, pathlib
CAP = 65000
HARD = 65536
NL = chr(10)
FENCE = chr(96) * 3
HEADER = "@coderabbitai plan" + NL + "Please enrich, fix and assess the plan below." + NL
ROOT = "/Users/arthrod/temp/T/jubarte-redlines/.scan/remediation"
folder = sys.argv[1]
base = pathlib.Path(ROOT)
issues = base / "issues"
issues.mkdir(parents=True, exist_ok=True)
mapped_path = issues / (folder + ".mapped.json")
plan_path = base / "plans" / (folder + ".md")
patch_path = base / "patches" / (folder + ".patch")
for p in (mapped_path, plan_path, patch_path):
    if not p.is_file():
        print("missing input: " + str(p))
        sys.exit(1)
mapped = json.loads(mapped_path.read_text())
plan = plan_path.read_text()
patch = patch_path.read_text()

def one(s):
    return " ".join(str(s if s is not None else "?").split()) or "?"

def chunk(text, cap):
    parts = []
    cur = ""
    for line in text.split(NL):
        cand = cur + NL + line if cur else line
        if len(cand) > cap and cur:
            parts.append(cur)
            cur = line
        else:
            cur = cand
    if cur:
        parts.append(cur)
    return parts

def tagged(title, parts):
    if not parts:
        parts = ["(empty)"]
    n = len(parts)
    out = []
    for i in range(n):
        out.append(title + " (" + str(i + 1) + "/" + str(n) + ")" + NL + NL + parts[i])
    return out

compact = NL.join(["## Findings tackled"] + [one(f.get("findingId")) + " | " + one(f.get("title")) + " | " + one(f.get("severity")) for f in mapped])
fulls = []
for f in mapped:
    fulls.append("### " + one(f.get("findingId")) + " | " + one(f.get("title")) + NL + "severity: " + one(f.get("severity")) + NL + "summary: " + one(f.get("summary")) + NL + "remediation:" + NL + str(f.get("remediation") or "(none)"))
full_findings = (NL + NL).join(fulls)
pointer_f = "Full texts of the findings above (summary and remediation for every findingId) are posted as follow-up comments on this issue."
pointer_p = "The full TDD red/green plan is posted as a follow-up comment on this issue."
split_note = "Because of the 65536-character cap, the patch is split into numbered parts: this body holds part 1 and the remaining parts are follow-up comments."
comments = []
findings_in_body = "no"
plan_in_body = "no"
patch_parts = 1
body = HEADER + NL + "## Findings tackled" + NL + full_findings + NL + NL + "## TDD red/green plan" + NL + plan + NL + NL + "## Patch" + NL + FENCE + "diff" + NL + patch + NL + FENCE + NL
if len(body) <= CAP:
    findings_in_body = "full"
    plan_in_body = "yes"
else:
    body = HEADER + NL + compact + NL + NL + pointer_f + NL + NL + "## TDD red/green plan" + NL + plan + NL + NL + "## Patch" + NL + FENCE + "diff" + NL + patch + NL + FENCE + NL
    comments.extend(tagged("## Full finding texts", chunk(full_findings, CAP - 200)))
    if len(body) <= CAP:
        plan_in_body = "yes"
    else:
        body = HEADER + NL + compact + NL + NL + pointer_f + NL + pointer_p + NL + NL + "## Patch" + NL + FENCE + "diff" + NL + patch + NL + FENCE + NL
        comments.extend(tagged("## Full TDD red/green plan", chunk(plan, CAP - 200)))
        if len(body) <= CAP:
            plan_in_body = "comment"
        else:
            prefix = HEADER + NL + compact + NL + NL + pointer_f + NL + pointer_p + NL + split_note + NL + NL
            parts = chunk(patch, CAP - len(prefix) - 200)
            patch_parts = len(parts)
            body = prefix + "## Patch (part 1/" + str(patch_parts) + ")" + NL + FENCE + "diff" + NL + parts[0] + NL + FENCE + NL
            comments.extend(tagged("## Patch (continued)", parts[1:]))
            plan_in_body = "comment"
files = [issues / (folder + ".body.md")]
files[0].write_text(body)
for i in range(len(comments)):
    p = issues / (folder + ".comment-" + str(i + 1).zfill(2) + ".md")
    p.write_text(comments[i])
    files.append(p)
bad = 0
print("folder " + folder + ": body " + str(len(body)) + " chars, comments " + str(len(comments)) + ", findings_in_body=" + findings_in_body + ", plan_in_body=" + plan_in_body + ", patch_parts=" + str(patch_parts))
for p in files:
    n = len(p.read_text())
    mark = "OK" if n <= HARD else "OVER-CAP"
    if n > HARD:
        bad = 1
    print(str(p) + " " + str(n) + " " + mark)
sys.exit(bad)

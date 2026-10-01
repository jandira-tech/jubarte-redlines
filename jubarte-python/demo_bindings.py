# Demo: jubarte_redlines python bindings, different variations
import sys
sys.path.insert(0, "tests")
from docx_fixture import docx, para
import jubarte_redlines as jb

def show(title):
    print(f"\n{'='*62}\n{title}\n{'='*62}")

original = docx(para("Alpha Beta Gamma.") + para("Second paragraph here.") + para("Third line stays."))
modified = docx(para("Alpha Beta DELTA.") + para("Completely rewritten text.") + para("Third line stays."))

# --- 1. compare_documents: default author ---
show("1. compare_documents (default author)")
redline = jb.compare_documents(original, modified)
print(f"redline bytes: {len(redline)}")

# --- 2. compare_documents: custom author + ISO date ---
show("2. compare_documents (custom author + date)")
redline2 = jb.compare_documents(original, modified, author="Reviewer", date="2026-01-15T10:00:00Z")
print(f"redline bytes: {len(redline2)}")

# --- 3. get_revisions vs get_revisions_json ---
show("3. get_revisions / get_revisions_json")
for rev in jb.get_revisions(redline):
    print(f"  {rev['type']:>8} author={rev.get('author')!r} text={rev.get('text')!r}")
import json
recs = json.loads(jb.get_revisions_json(redline))
print(f"  JSON: {len(recs)} revisions")

# --- 4. accept / reject ---
show("4. accept_revisions / reject_revisions")
accepted = jb.accept_revisions(redline)
rejected = jb.reject_revisions(redline)
acc_md = jb.read.__doc__  # placeholder
doc_acc = jb.document.Document.from_bytes(accepted)
doc_rej = jb.document.Document.from_bytes(rejected)
print("accepted markdown:", doc_acc.markdown())
print("rejected markdown:", doc_rej.markdown())

# --- 5. Document API: read, compare, changes, inspect ---
show("5. Document API (compare, changes, inspect)")
d = jb.document.Document.from_bytes(original)
m = jb.document.Document.from_bytes(modified)
res = d.compare(m, author="Agent")
print("compare -> Document, sha256:", res.sha256()[:16], "…")
for ch in res.changes():
    print(f"  change id={ch.id} kind={ch.kind} text={ch.text!r}")
snap = d.inspect()
print("snapshot paragraphs:", snap.summary.paragraphs if hasattr(snap, "summary") else len(snap.paragraphs))
print("markdown:", d.markdown())

# --- 6. EditPlan ---
show("6. EditPlan (replace + insert + comment)")
plan = (jb.models.EditPlan(author="Editor")
        .replace(0, find="Beta", replacement="REPLACED")
        .insert(0, text="Inserted run.", after="Beta")
        .insert_paragraph(0, runs=["A whole new paragraph."], position="after")
        .comment(0, text="Check this.", find="Alpha"))
result = d.edit(plan)
print("report:", result.report)
print("clean markdown:\n" + result.clean.markdown())
print("redline revisions:", len(result.redline.revisions()))

# --- 7. docx_to_pdf + to_png ---
show("7. docx_to_pdf / to_png")
pdf = jb.docx_to_pdf(redline)
print(f"PDF bytes: {len(pdf)}, header: {pdf[:8]!r}")
d2 = jb.document.Document.from_bytes(original)
pngs = d2.to_png(dpi=72.0)
print(f"PNG pages: {len(pngs)}, first PNG header: {pngs[0][:8]!r}")

# --- 8. module CLI: python -m jubarte_redlines ---
show("8. python -m jubarte_redlines --help")
import subprocess
r = subprocess.run([sys.executable, "-m", "jubarte_redlines", "--help"], capture_output=True, text=True,
                   env={"PYTHONPATH": "python", "PATH": "/usr/bin:/bin"})
print((r.stdout or r.stderr)[:600])

# --- 9. error handling ---
show("9. JubarteError on garbage input")
try:
    jb.get_revisions(b"not a docx")
except jb.JubarteError as e:
    print("caught JubarteError:", str(e)[:100])

print("\nAll variations OK ✅")

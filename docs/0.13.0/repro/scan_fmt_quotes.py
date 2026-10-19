"""Count, per 0.12.0 deliverable, agent-authored formatting changes (rPrChange) and
straight-quote insertions that replace curly quotes; compare with the input.
Usage: scan_fmt_quotes.py JOB_DIR"""
import glob, json, os, re, sys, zipfile, collections
from lxml import etree
W = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
q = lambda t: f"{{{W}}}{t}"
ROOT = "/Users/arthrod/temp/T/redline-bench"
tot = collections.Counter(); ex = collections.defaultdict(list)
for d in sorted(glob.glob(f"{sys.argv[1]}/redline-*/")):
    task = re.search(r"redline-s\d+-t\d+-g\d+[a-z]", d).group(0)
    out = d + "artifacts/app/contract.docx"
    if not os.path.exists(out): continue
    author = json.load(open(f"{ROOT}/.tasks/{task}/tests/rubrics.json"))["author"]
    try: doc = etree.fromstring(zipfile.ZipFile(out).read("word/document.xml"))
    except Exception: continue
    rpr = [e for e in doc.iter(q("rPrChange")) if e.get(q("author")) == author]
    ppr = [e for e in doc.iter(q("pPrChange")) if e.get(q("author")) == author]
    # unrequested un-bolding/un-italic: old rPr had b/i/u that the new rPr lacks
    lost = 0
    for e in rpr:
        old = e.find(q("rPr")); new = e.getparent()
        for tag in ("b", "i", "u"):
            if old is not None and old.find(q(tag)) is not None and new.find(q(tag)) is None:
                lost += 1
                r = new.getparent(); txt = "".join(t.text or "" for t in r.iter(q("t")))
                if len(ex["lost"]) < 8: ex["lost"].append((task, tag, txt[:50]))
                break
    # straight quote inserted by author where a curly quote is deleted by author nearby
    swaps = 0
    for p in doc.iter(q("p")):
        ins = "".join(t.text or "" for i in p.iter(q("ins")) if i.get(q("author")) == author for t in i.iter(q("t")))
        dels = "".join(t.text or "" for d_ in p.iter(q("del")) if d_.get(q("author")) == author for t in d_.iter(q("delText")))
        n = min(ins.count('"') + ins.count("'"), sum(dels.count(c) for c in "“”‘’"))
        if n:
            swaps += n
            if len(ex["swap"]) < 6: ex["swap"].append((task, ins[:60], dels[:60]))
    if rpr or ppr or swaps:
        tot["trials_rpr"] += bool(rpr); tot["rpr"] += len(rpr); tot["lost"] += lost; tot["trials_lost"] += bool(lost)
        tot["ppr"] += len(ppr); tot["trials_ppr"] += bool(ppr); tot["swaps"] += swaps; tot["trials_swaps"] += bool(swaps)
print(dict(tot))
for k, v in ex.items():
    print(k); [print("  ", x) for x in v]

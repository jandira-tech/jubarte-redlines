"""Accept-all / reject-all paragraph texts of a docx body, honouring paragraph-mark revisions.
Usage: views.py FILE accept|reject [grep]"""
import sys, zipfile, re
from lxml import etree
W = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
q = lambda t: f"{{{W}}}{t}"
def view(path, mode):
    body = etree.fromstring(zipfile.ZipFile(path).read("word/document.xml")).find(q("body"))
    out, cur = [], ""
    for p in body.iter(q("p")):
        for el in p.iter(q("t"), q("delText"), q("tab")):
            anc = {a.tag for a in el.iterancestors() if a.tag in (q("ins"), q("del"), q("moveFrom"), q("moveTo"))}
            if el.tag == q("delText") or q("del") in anc or q("moveFrom") in anc:
                if mode == "reject": cur += el.text or ("\t" if el.tag == q("tab") else "")
                continue
            if q("ins") in anc or q("moveTo") in anc:
                if mode == "accept": cur += el.text or ("\t" if el.tag == q("tab") else "")
                continue
            cur += el.text or ("\t" if el.tag == q("tab") else "")
        mark = p.find(f"{q('pPr')}/{q('rPr')}")
        mdel = mark is not None and mark.find(q("del")) is not None
        mins = mark is not None and mark.find(q("ins")) is not None
        if (mode == "accept" and mdel) or (mode == "reject" and mins):
            continue  # paragraph mark removed: merge with the next paragraph
        out.append(cur); cur = ""
    if cur: out.append(cur)
    return out
if __name__ == "__main__":
    v = view(sys.argv[1], sys.argv[2])
    pat = sys.argv[3] if len(sys.argv) > 3 else None
    for i, t in enumerate(v):
        if pat is None or re.search(pat, t): print(i, repr(t[:150]))

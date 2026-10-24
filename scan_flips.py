#!/usr/bin/env python3
"""Scan tests/ docx fixtures: which tables would the new signal flip?
A table flips when: tblW is auto/absent/zero, no tblLayout fixed, grid valid
(all gridCol have w:w, none w:type), and some cell's tcW is absent/auto/nil/
zero. Also: vMerge continuation cells' tcW coverage in Word files."""
import os, re, sys, zipfile
from collections import Counter

W = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
root = sys.argv[1] if len(sys.argv) > 1 else "tests"
pat = re.compile(r"<w:tbl>.*?</w:tbl>", re.S)
tr_pat = re.compile(r"<w:tr(?:\s[^>]*)?>.*?</w:tr>", re.S)
tc_pat = re.compile(r"<w:tc(?:\s[^>]*)?>.*?</w:tc>", re.S)

def tcw_state(tc):
    m = re.search(r"<w:tcW\s([^>]*?)/>", tc)
    if not m:
        return "absent"
    a = m.group(1)
    t = re.search(r'w:type="([^"]*)"', a)
    w = re.search(r'w:w="([^"]*)"', a)
    t = t.group(1) if t else None
    w = w.group(1) if w else None
    if t is None:
        return "notype(w=%s)" % w
    if t == "dxa":
        return "dxa0" if (w is None or float(w or 0) <= 0) else "dxa"
    if t == "pct":
        return "pct0" if (w is None or float(w or 0) <= 0) else "pct"
    if t == "auto":
        return "auto(w=%s)" % w
    return t

stats = Counter()
flips = []
vmerge = Counter()
files = 0
for dirpath, _, names in os.walk(root):
    for n in names:
        if not n.endswith(".docx"):
            continue
        path = os.path.join(dirpath, n)
        try:
            z = zipfile.ZipFile(path)
            parts = [p for p in z.namelist() if re.match(r"word/(document|header\d*|footer\d*|footnotes|endnotes|comments)\.xml$", p)]
            for part in parts:
                xml = z.read(part).decode("utf8", "ignore")
                for ti, tbl in enumerate(pat.findall(xml)):
                    tblpr = re.search(r"<w:tblPr>.*?</w:tblPr>", tbl, re.S)
                    tblpr = tblpr.group(0) if tblpr else ""
                    mw = re.search(r"<w:tblW\s([^>]*?)/>", tblpr)
                    wtype = re.search(r'w:type="([^"]*)"', mw.group(1)).group(1) if mw else None
                    ww = re.search(r'w:w="([^"]*)"', mw.group(1)).group(1) if mw else None
                    fixed = '<w:tblLayout w:type="fixed"' in tblpr
                    grid = re.search(r"<w:tblGrid>.*?</w:tblGrid>", tbl, re.S)
                    gcols = re.findall(r"<w:gridCol\s([^>]*?)/>", grid.group(0)) if grid else []
                    valid = bool(gcols) and all(('w:w=' in c) and ('w:type=' not in c) for c in gcols)
                    autofit_tblw = (wtype in (None, "auto")) or (wtype == "dxa" and (ww is None or float(ww) == 0))
                    cells = []
                    for tr in tr_pat.findall(tbl):
                        for tc in tc_pat.findall(tr):
                            vm = re.search(r"<w:vMerge(?:\s([^>]*?))?/>", tc)
                            vm = (vm.group(1) or "" if vm else "").strip()
                            cells.append((tcw_state(tc), vm))
                    autos = [c for c, _ in cells if c not in ("dxa", "pct")]
                    key = ("fixed" if fixed else "autofit", wtype, valid)
                    stats[key + ("flip" if autos else "keep",)] += 1
                    for st, vm in cells:
                        if vm != "" and "restart" not in vm:  # continuation
                            vmerge[st] += 1
                    if autos and not fixed and autofit_tblw and valid:
                        flips.append((path, part, ti, Counter(autos)))
        except Exception as e:
            stats[("error", str(e)[:30], "")] += 1
        files += 1

print("files scanned:", files)
for k, v in sorted(stats.items(), key=lambda kv: -kv[1]):
    print(v, k)
print("\nvMerge continuation cell tcW states across all tables:")
for k, v in vmerge.most_common():
    print(v, k)
print("\ntables that FLIP (autofit, valid grid, >=1 auto tcW):", len(flips))
for f in flips[:20]:
    print("  ", f)

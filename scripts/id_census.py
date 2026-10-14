# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only
"""Object census against short id A plus the inline-object spellings (2026-10-13):
sN, ccN, pN.bmK / bm:NAME, pN.fieldK, pN.linkK, pN.imgK, pN.ccK, pN.eqK, tocN.
Each object is counted once, Fallback copies excluded. 'reach' says how it is named:
  id       a short id names it
  name     only by its name (bm:NAME): a bookmark outside any paragraph
  markdown no id, but the Markdown says it (runs -> text/formatting, note refs -> [^n])
  none     nothing names it

  uv run -p 3.13 --with lxml python scripts/id_census.py tests/corpus
"""

import sys
import zipfile
from collections import Counter, defaultdict
from concurrent.futures import ProcessPoolExecutor
from pathlib import Path

from lxml import etree

W = (
    "http://schemas.openxmlformats.org/wordprocessingml/2006/main",
    "http://purl.oclc.org/ooxml/wordprocessingml/main",
)
M = "http://schemas.openxmlformats.org/officeDocument/2006/math"
MC = "http://schemas.openxmlformats.org/markup-compatibility/2006"
STORIES = ("document", "header", "footer", "footnotes", "endnotes", "comments")
PROPS = ("rPr", "pPr", "tblPr", "trPr", "tcPr", "sectPr", "tblPrEx", "rPrChange", "pPrChange")

SPELL = {
    "body paragraph": "p12",
    "story paragraph": "header1.p0",
    "comment paragraph": "c5",
    "text-box paragraph": "p14.box0.p0",
    "table": "t0",
    "nested table": "t0.row1.cell2.t0",
    "row": "t0.row1",
    "cell": "t0.row1.cell2",
    "header/footer part": "header1",
    "footnote/endnote": "footnotes.p3",
    "comment": "c5",
    "text revision": "r12",
    "paragraph-mark revision": "r12",
    "property revision": "r12",
    "move": "r12",
    "table-structure revision": "r12",
    "section": "s0",
    "table of contents": "toc0",
    "bookmark": "p12.bm0 / bm:NAME",
    "field": "p12.field0",
    "hyperlink": "p12.link0",
    "drawing/VML/OLE": "p12.img0",
    "equation": "p12.eq0",
    "content control (inline)": "p12.cc0",
    "content control (block)": "cc0",
    "run": "(markdown text and formatting)",
    "note reference": "(markdown [^n])",
}


def split(el):
    t = el.tag
    if not isinstance(t, str) or t[0] != "{":
        return None, None
    uri, name = t[1:].split("}")
    return uri, name


def attr(el, uri, name):
    return el.get(f"{{{uri}}}{name}")


def sdt_is_toc(el, uri):
    for g in el.iter(f"{{{uri}}}docPartGallery"):
        if (attr(g, uri, "val") or "").startswith("Table of Contents"):
            return True
    return False


def field_code(el, uri):
    """The instruction of a complex field starting at fldChar begin: instrText up to separate/end."""
    run = el.getparent()
    code = []
    node = run
    depth = 0
    while node is not None:
        for c in node.iter():
            u, n = split(c)
            if n == "fldChar" and c is not el:
                t = attr(c, u, "fldCharType")
                if t == "begin":
                    depth += 1
                elif t in ("separate", "end"):
                    if depth == 0:
                        return "".join(code).strip()
                    if t == "end":
                        depth -= 1
            elif n == "instrText" and depth == 0:
                code.append(c.text or "")
        nxt = node.getnext()
        while nxt is None and node.getparent() is not None and split(node.getparent())[1] != "body":
            node = node.getparent()
            nxt = node.getnext()
        node = nxt
        if len(code) > 50:
            break
    return "".join(code).strip()


def kind_of(el, story):
    uri, name = split(el)
    if uri is None:
        return None
    anc = [split(a) for a in el.iterancestors()]
    names = [n for u, n in anc]
    if (MC, "Fallback") in anc:
        return None
    in_p = "p" in names
    if uri == M:
        if name == "oMathPara" or (name == "oMath" and "oMathPara" not in names):
            return "equation", in_p
        return None
    if uri not in W:
        return None
    in_props = any(n in PROPS for n in names[:3])
    if name == "p":
        if "txbxContent" in names:
            return "text-box paragraph", True
        return {"document": "body paragraph", "comments": "comment paragraph"}.get(
            story, "story paragraph"
        ), True
    if name == "tbl":
        return ("nested table" if "tc" in names else "table"), True
    if name in ("tr", "tc"):
        return {"tr": "row", "tc": "cell"}[name], True
    if name in ("footnote", "endnote"):
        if attr(el, uri, "type") in ("separator", "continuationSeparator", "continuationNotice"):
            return None
        return "footnote/endnote", True
    if name == "comment":
        return "comment", True
    if name in ("ins", "del"):
        # A row's own w:trPr/w:ins|w:del inserts or deletes the row.
        if names[0] == "trPr":
            return "table-structure revision", True
        return ("paragraph-mark revision" if in_props else "text revision"), True
    if name in (
        "rPrChange",
        "pPrChange",
        "sectPrChange",
        "tblPrChange",
        "trPrChange",
        "tcPrChange",
        "tblPrExChange",
        "numberingChange",
    ):
        return "property revision", True
    if name in ("moveFrom", "moveTo"):
        return "move", True
    if name in ("tblGridChange", "cellIns", "cellDel", "cellMerge"):
        return "table-structure revision", True
    if name == "r":
        return "run", True
    if name == "bookmarkStart":
        return "bookmark", in_p
    if name == "fldSimple":
        code = (attr(el, uri, "instr") or "").strip()
        return ("table of contents" if code.upper().startswith("TOC") else "field"), in_p
    if name == "fldChar" and attr(el, uri, "fldCharType") == "begin":
        code = field_code(el, uri)
        if code.upper().startswith("TOC"):
            # A TOC inside a TOC content control is counted once, by the control.
            if any(n == "sdtContent" for n in names):
                sdt = next(a for a in el.iterancestors() if split(a)[1] == "sdt")
                if sdt_is_toc(sdt, uri):
                    return "field", in_p
            return "table of contents", True
        return "field", in_p
    if name == "hyperlink":
        return "hyperlink", in_p
    if name in ("drawing", "pict", "object"):
        return "drawing/VML/OLE", in_p
    if name == "sdt":
        if sdt_is_toc(el, uri):
            return "table of contents", True
        return ("content control (inline)" if in_p else "content control (block)"), True
    if name == "sectPr" and "sectPrChange" not in names:
        return "section", True
    if name in ("footnoteReference", "endnoteReference"):
        return "note reference", True
    return None


def census(path):
    c = Counter()
    docs = Counter()
    try:
        z = zipfile.ZipFile(path)
    except Exception:
        return c, docs
    for info in z.infolist():
        part = info.filename
        if not (part.startswith("word/") and part.endswith(".xml") and part.count("/") == 1):
            continue
        stem = part[5:-4]
        story = next(
            (s for s in STORIES if stem.startswith(s) and not stem.startswith(("commentsEx", "commentsIds"))),
            None,
        )
        if story is None:
            continue
        try:
            root = etree.fromstring(z.read(part))
        except Exception:
            continue
        if story in ("header", "footer"):
            c["header/footer part", "id"] += 1
        for el in root.iter():
            k = kind_of(el, story)
            if not k:
                continue
            kind, in_p = k
            if kind in ("run", "note reference"):
                reach = "markdown"
            elif kind in ("bookmark",) and not in_p:
                reach = "name"
            elif kind in ("field", "hyperlink", "drawing/VML/OLE", "equation") and not in_p:
                reach = "none"
            else:
                reach = "id"
            c[kind, reach] += 1
    for k in {k for k, _ in c}:
        docs[k] = 1
    return c, docs


def main():
    files = sorted(f for f in Path(sys.argv[1]).rglob("*.docx") if not f.name.startswith("~$"))
    total, docs = Counter(), Counter()
    with ProcessPoolExecutor() as pool:
        for c, d in pool.map(census, files, chunksize=8):
            total.update(c)
            docs.update(d)
    kinds = defaultdict(Counter)
    for (k, r), n in total.items():
        kinds[k][r] += n
    by_reach = Counter()
    for (k, r), n in total.items():
        by_reach[r] += n
    grand = sum(by_reach.values())
    print(f"{len(files)} documents, {grand:,} objects\n")
    print("| Reach | Objects | Share |\n|-|-:|-:|")
    for r in ("id", "name", "markdown", "none"):
        print(f"| {r} | {by_reach[r]:,} | {100 * by_reach[r] / grand:.2f}% |")
    named = by_reach["id"] + by_reach["name"]
    nonrun = grand - sum(kinds[k].total() for k in ("run", "note reference"))
    print(
        f"\nWithout runs and note references: {nonrun:,} objects, {100 * named / nonrun:.2f}% named by an id or a name\n"
    )
    print("| Object | Count | Docs | Spelling | Not reachable |\n|-|-:|-:|-|-:|")
    for k, cnt in sorted(kinds.items(), key=lambda kv: -kv[1].total()):
        print(
            f"| {k} | {cnt.total():,} | {docs[k]:,} | `{SPELL[k]}` | {cnt['none']:,}{' (' + format(cnt['name'], ',') + ' by name only)' if cnt['name'] else ''} |"
        )


if __name__ == "__main__":
    main()

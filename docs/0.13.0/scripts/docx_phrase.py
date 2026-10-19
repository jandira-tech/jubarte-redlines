"""Where does a phrase sit in a .docx: plain text, someone's tracked insertion, or a tracked deletion?

Usage: python3 -I docx_phrase.py FILE.docx "phrase" ["phrase" ...]
"""
import re, sys, zipfile
import xml.etree.ElementTree as ET

W = "{http://schemas.openxmlformats.org/wordprocessingml/2006/main}"


def norm(s):
    s = s.replace("’", "'").replace("‘", "'").replace("“", '"').replace("”", '"')
    return re.sub(r"\s+", " ", s).lower()


def paragraphs(path):
    root = ET.fromstring(zipfile.ZipFile(path).read("word/document.xml"))
    for p in root.iter(W + "p"):
        chars = []

        def walk(el, state):
            for ch in el:
                tag = ch.tag.replace(W, "")
                st = state
                if tag in ("ins", "del"):
                    st = f"{tag}:{ch.get(W + 'author')}"
                if tag in ("t", "delText") and ch.text:
                    chars.extend((c, st) for c in ch.text)
                elif tag == "tab":
                    chars.append((" ", st))
                elif tag != "p":
                    walk(ch, st)
        walk(p, "plain")
        yield chars


def q(s):
    return s.translate(str.maketrans("\u2019\u2018\u201c\u201d\u00a0", "''\"\" "))


def locate(path, phrase):
    pat = re.compile(r"\s+".join(re.escape(w) for w in q(phrase).split()), re.I)
    hits = []
    for chars in paragraphs(path):
        for view, keep in (("current", lambda s: not s.startswith("del:")),
                           ("original", lambda s: not s.startswith("ins:"))):
            kept = [(c, s) for c, s in chars if keep(s)]
            raw = q("".join(c for c, _ in kept))
            for m in pat.finditer(raw):
                states = sorted({s for _, s in kept[m.start():m.end()]})
                hits.append((view, states, raw[max(0, m.start() - 40):m.end() + 40]))
    return hits


if __name__ == "__main__":
    path = sys.argv[1]
    for ph in sys.argv[2:]:
        h = locate(path, ph)
        views = sorted({v for v, _, _ in h})
        print(f"  {ph[:60]!r}: views={views or 'NOT FOUND'}")
        for v, st, raw in h[:2]:
            print(f"     {v}: states={st} :: {raw[:120]}")

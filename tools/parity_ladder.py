#!/usr/bin/env python3
"""Word-parity ladder: tiny, named findings from (A, B, word_redline) triples.

Corpus: neurotic_docx_bench corpus_sanity/word_based layout —
  centralized_mapping.csv joins docx_source/{base,next}.docx to the
  Word-generated redline in docx_redlines_word/.

For each pair we produce OUR redline (jubarte CLI), then climb the ladder;
the FIRST failing level names the problem class:

  L0 recon    our delText-stream == text(A), ins-stream == text(B)
              (whitespace-insensitive). Fails => we corrupt content.
  GT  gate    Word's redline must itself pass L0, else pair excluded
              (finding 'gt-invalid', no parity levels run).
  L1 opseq    coalesced (eq|ins|del, text) sequence == Word's.
  L2 inventory  revision-element counts by local name == Word's.
  L3 histogram  element qnames systematically present in exactly one side.

Signatures: Word-independent defect detectors on OUR output (S-* findings).
Each is ~10 lines; add new ones to SIGNATURES.

Ratchet: findings are compared against tools/parity_baseline.tsv.
  sweep  -> exit 1 listing NEW findings (regressions); prints FIXED rows.
  bless  -> rewrite baseline from current findings.
  mine   -> corpus-wide qname histogram diff ours-vs-Word (hypothesis feed).

stdlib only. Usage:
  python3 tools/parity_ladder.py sweep  [--corpus DIR] [--limit N] [--only STEM]
  python3 tools/parity_ladder.py bless  [...]
  python3 tools/parity_ladder.py mine   [...]
"""
from __future__ import annotations

import argparse
import csv
import os
import re
import subprocess
import sys
import zipfile
import xml.etree.ElementTree as ET
from collections import Counter

W = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
MC = "http://schemas.openxmlformats.org/markup-compatibility/2006"
WPS = "http://schemas.microsoft.com/office/word/2010/wordprocessingShape"
HERE = os.path.dirname(os.path.abspath(__file__))
CRATE = os.path.dirname(HERE)
DEFAULT_CORPUS = "/Users/arthrod/temp/T/neurotic_docx_bench/corpus/word_based"
DEFAULT_BIN = os.path.join(CRATE, "target", "release", "jubarte")
BASELINE = os.path.join(HERE, "parity_baseline.tsv")
OUT_DIR = os.path.join(CRATE, "_scratch", "parity_ladder")

REVISION_ELEMENTS = {
    "ins", "del", "moveFrom", "moveTo", "moveFromRangeStart", "moveFromRangeEnd",
    "moveToRangeStart", "moveToRangeEnd", "rPrChange", "pPrChange", "tblPrChange",
    "trPrChange", "tcPrChange", "sectPrChange", "numberingChange", "cellIns",
    "cellDel", "cellMerge", "customXmlInsRangeStart", "customXmlInsRangeEnd",
    "customXmlDelRangeStart", "customXmlDelRangeEnd", "delText", "delInstrText",
}


def read_document_xml(path):
    with zipfile.ZipFile(path) as z:
        return ET.fromstring(z.read("word/document.xml"))


def norm(s):
    return re.sub(r"\s+", "", s)


def mc_children(el):
    """Children with mc:AlternateContent resolved to its first mc:Choice
    (Fallback duplicates the same content and must not be double-counted)."""
    if el.tag == f"{{{MC}}}AlternateContent":
        for c in el:
            if c.tag == f"{{{MC}}}Choice":
                return list(c)
        return [c for c in el if c.tag == f"{{{MC}}}Fallback"]
    return list(el)


def source_text(root):
    parts = []

    def rec(el):
        if el.tag == f"{{{W}}}t":
            parts.append(el.text or "")
        for c in mc_children(el):
            rec(c)

    rec(root)
    return norm("".join(parts))


def redline_walk(root):
    """Return op list [(op, text)] with op in eq/ins/del, doc order, coalesced."""
    ops = []

    def emit(op, text):
        if not text:
            return
        if ops and ops[-1][0] == op:
            ops[-1][1] += text
        else:
            ops.append([op, text])

    def rec(el, in_ins, in_del):
        tag = el.tag
        if tag == f"{{{W}}}ins" or tag == f"{{{W}}}moveTo":
            in_ins = True
        elif tag == f"{{{W}}}del" or tag == f"{{{W}}}moveFrom":
            in_del = True
        if tag == f"{{{W}}}delText":
            emit("del", norm(el.text or ""))
        elif tag == f"{{{W}}}t":
            if in_ins:
                emit("ins", norm(el.text or ""))
            elif in_del:
                emit("del", norm(el.text or ""))
            else:
                emit("eq", norm(el.text or ""))
        for c in mc_children(el):
            rec(c, in_ins, in_del)

    rec(root, False, False)
    return [(op, t) for op, t in ops]


def recon(ops):
    """(original_text, modified_text) from an op list."""
    orig = "".join(t for op, t in ops if op in ("eq", "del"))
    mod = "".join(t for op, t in ops if op in ("eq", "ins"))
    return orig, mod


def local(tag):
    return tag.rsplit("}", 1)[-1]


def rev_inventory(root):
    c = Counter()
    for el in root.iter():
        name = local(el.tag)
        if name in REVISION_ELEMENTS:
            c[name] += 1
    return c


def qname_histogram(root):
    return Counter(el.tag for el in root.iter())


# ---------- signatures (each: fn(root) -> list of detail strings) ----------

def sig_bare_wps_drawing(root):
    """w:drawing containing wps shapes but no mc:AlternateContent ancestor —
    Word wraps these in AlternateContent(Choice wps / Fallback pict); bare
    emission is the strict01 repair-dialog trigger."""
    out = []

    def rec(el, in_ac):
        if el.tag == f"{{{MC}}}AlternateContent":
            in_ac = True
        if el.tag == f"{{{W}}}drawing" and not in_ac:
            if any(c.tag.startswith(f"{{{WPS}}}") for c in el.iter()):
                out.append("bare wps drawing (no AlternateContent/pict fallback)")
        for c in el:
            rec(c, in_ac)

    rec(root, False)
    return out[:1]  # one finding per file is enough


def sig_instrtext_in_del(root):
    """w:instrText inside w:del must be w:delInstrText (Word: always)."""
    n = 0

    def rec(el, in_del):
        nonlocal n
        if el.tag == f"{{{W}}}del":
            in_del = True
        if el.tag == f"{{{W}}}instrText" and in_del:
            n += 1
        for c in el:
            rec(c, in_del)

    rec(root, False)
    return [f"{n}x instrText inside w:del (want delInstrText)"] if n else []


def sig_rsid_leftover(root):
    n = sum(1 for el in root.iter() for a in el.attrib if "rsid" in local(a).lower())
    return [f"{n}x rsid attributes remain"] if n else []


def sig_empty_revision_wrappers(root):
    """Empty w:ins/w:del in CONTENT position. Empty ones inside w:rPr /
    w:trPr are legal paragraph-mark / row revision markers — excluded."""
    n = 0
    for parent in root.iter():
        if local(parent.tag) in ("rPr", "trPr", "ctrlPr"):
            continue
        n += sum(1 for c in parent if local(c.tag) in ("ins", "del") and len(c) == 0)
    return [f"{n}x empty w:ins/w:del wrappers"] if n else []


def sig_duplicate_docpr_ids(root):
    ids = [el.get("id") for el in root.iter() if local(el.tag) == "docPr"]
    dupes = [i for i, c in Counter(ids).items() if i is not None and c > 1]
    return [f"duplicate docPr ids: {sorted(dupes)[:5]}"] if dupes else []


SIGNATURES = {
    "S-bare-wps-drawing": sig_bare_wps_drawing,
    "S-instrtext-in-del": sig_instrtext_in_del,
    "S-rsid-leftover": sig_rsid_leftover,
    "S-empty-ins-del": sig_empty_revision_wrappers,
    "S-dup-docpr-id": sig_duplicate_docpr_ids,
}

# ---------------------------------------------------------------------------


def load_pairs(corpus):
    """Yield (stem, pathA, pathB, path_word_redline_or_None)."""
    src = os.path.join(corpus, "docx_source")
    gt_dir = os.path.join(corpus, "docx_redlines_word")
    with open(os.path.join(corpus, "centralized_mapping.csv"), newline="") as f:
        for row in csv.DictReader(f):
            a = os.path.join(src, row["docx_source_base"])
            b = os.path.join(src, row["docx_source_next"])
            gt_name = row.get("redline_docx_word") or row.get("redline_docx") or ""
            gt = os.path.join(gt_dir, gt_name) if gt_name else None
            if gt and not os.path.isfile(gt):
                gt = None
            if os.path.isfile(a) and os.path.isfile(b):
                yield row["pair_stem"], a, b, gt


def run_ours(binary, a, b, stem):
    os.makedirs(OUT_DIR, exist_ok=True)
    out = os.path.join(OUT_DIR, stem + ".ours.docx")
    r = subprocess.run([binary, a, b, "-o", out, "--force"], capture_output=True, text=True, timeout=300)
    if r.returncode != 0:
        return None, f"exit {r.returncode}: {(r.stderr or r.stdout)[:200].strip()}"
    return out, None


def opseq_delta(ours, word):
    """First diverging op index + short context, or None."""
    for i, (x, y) in enumerate(zip(ours, word)):
        if x != y:
            return f"op[{i}] ours={x[0]}:{x[1][:40]!r} word={y[0]}:{y[1][:40]!r}"
    if len(ours) != len(word):
        return f"op count ours={len(ours)} word={len(word)}"
    return None


def ladder(stem, a, b, gt, binary):
    """Return list of (finding_key, detail). finding_key is stable for ratchet."""
    findings = []
    out, err = run_ours(binary, a, b, stem)
    if out is None:
        return [("CRASH", err)]
    try:
        ours_root = read_document_xml(out)
    except Exception as e:
        return [("BADZIP", str(e)[:120])]

    ta, tb = source_text(read_document_xml(a)), source_text(read_document_xml(b))
    ours_ops = redline_walk(ours_root)
    ro, rm = recon(ours_ops)
    if ro != ta:
        findings.append(("L0-original", f"recon len {len(ro)} vs src {len(ta)}"))
    if rm != tb:
        findings.append(("L0-modified", f"recon len {len(rm)} vs src {len(tb)}"))

    for key, fn in SIGNATURES.items():
        for detail in fn(ours_root):
            findings.append((key, detail))

    if findings and any(k.startswith("L0") for k, _ in findings):
        return findings  # content broken; parity levels meaningless
    if gt is None:
        return findings

    word_root = read_document_xml(gt)
    word_ops = redline_walk(word_root)
    wo, wm = recon(word_ops)
    if wo != ta or wm != tb:
        findings.append(("gt-invalid", "Word GT fails reconstruction; excluded"))
        return findings

    d = opseq_delta(ours_ops, word_ops)
    if d:
        findings.append(("L1-opseq", d))
        return findings  # L2/L3 would restate the same divergence

    oi, wi = rev_inventory(ours_root), rev_inventory(word_root)
    diff = {k: (oi.get(k, 0), wi.get(k, 0)) for k in set(oi) | set(wi) if oi.get(k, 0) != wi.get(k, 0)}
    if diff:
        findings.append(("L2-inventory", " ".join(f"{k}:{o}v{w}" for k, (o, w) in sorted(diff.items()))))

    oh, wh = qname_histogram(ours_root), qname_histogram(word_root)
    onlyo = sorted(local(q) for q in oh if q not in wh)
    onlyw = sorted(local(q) for q in wh if q not in oh)
    if onlyo or onlyw:
        findings.append(("L3-histogram", f"only-ours={onlyo[:8]} only-word={onlyw[:8]}"))
    return findings


def cmd_sweep(args, bless=False):
    rows = set()
    pairs = list(load_pairs(args.corpus))
    if args.only:
        pairs = [p for p in pairs if args.only in p[0]]
    if args.limit:
        pairs = pairs[: args.limit]
    for i, (stem, a, b, gt) in enumerate(pairs):
        for key, detail in ladder(stem, a, b, gt, args.bin):
            rows.add(f"{stem}\t{key}\t{detail}")
        if (i + 1) % 20 == 0:
            print(f"  …{i + 1}/{len(pairs)}", file=sys.stderr)
    rows = sorted(rows)
    swept = {p[0] for p in pairs}
    scoped = bool(args.only or args.limit)
    base = set()
    if os.path.isfile(BASELINE):
        base = {l.rstrip("\n") for l in open(BASELINE) if l.strip()}
    if bless:
        if scoped:  # merge: replace only swept pairs' rows, keep the rest
            rows = sorted({r for r in base if r.split("\t")[0] not in swept} | set(rows))
        with open(BASELINE, "w") as f:
            f.write("\n".join(rows) + ("\n" if rows else ""))
        print(f"blessed {len(rows)} findings -> {BASELINE}")
        return 0
    if scoped:  # only judge swept pairs
        base = {r for r in base if r.split("\t")[0] in swept}
    # ratchet keys ignore the volatile detail column
    strip = lambda s: "\t".join(s.split("\t")[:2])
    cur_keys, base_keys = {strip(r) for r in rows}, {strip(r) for r in base}
    new = sorted(cur_keys - base_keys)
    fixed = sorted(base_keys - cur_keys)
    detail_of = {strip(r): r for r in rows}
    print(f"{len(pairs)} pairs, {len(rows)} findings ({len(new)} NEW, {len(fixed)} fixed)")
    for k in new:
        print(f"NEW   {detail_of.get(k, k)}")
    for k in fixed:
        print(f"FIXED {k}   (run bless to shrink baseline)")
    return 1 if new else 0


def cmd_mine(args):
    ours_h, word_h, n = Counter(), Counter(), 0
    for stem, a, b, gt in load_pairs(args.corpus):
        if gt is None:
            continue
        out, err = run_ours(args.bin, a, b, stem)
        if out is None:
            continue
        ours_h += Counter(set(qname_histogram(read_document_xml(out))))
        word_h += Counter(set(qname_histogram(read_document_xml(gt))))
        n += 1
        if args.limit and n >= args.limit:
            break
    print(f"# files-containing-qname across {n} pairs (ours vs word)")
    for q in sorted(set(ours_h) | set(word_h), key=lambda q: -(abs(ours_h[q] - word_h[q]))):
        o, w = ours_h[q], word_h[q]
        if (o == 0) != (w == 0) or abs(o - w) > n // 4:
            print(f"{local(q)}\t{o}\t{w}")
    return 0


def main():
    p = argparse.ArgumentParser()
    p.add_argument("mode", choices=["sweep", "bless", "mine"])
    p.add_argument("--corpus", default=DEFAULT_CORPUS)
    p.add_argument("--bin", default=DEFAULT_BIN)
    p.add_argument("--limit", type=int, default=0)
    p.add_argument("--only", default="")
    args = p.parse_args()
    if args.mode == "sweep":
        sys.exit(cmd_sweep(args))
    if args.mode == "bless":
        sys.exit(cmd_sweep(args, bless=True))
    sys.exit(cmd_mine(args))


if __name__ == "__main__":
    main()

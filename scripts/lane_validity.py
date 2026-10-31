#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

# SPDX-License-Identifier: AGPL-3.0-only

"""Which validator errors in the release's redlines are jubarte's, and which came with the sources?

`python3 scripts/lane_validity.py SAMPLE_CSV --root BENCH_ROOT --redlines DIR`
validates every redline of a release_info redline sample (DIR/<key>.docx,
as scripts/redline-sweep.sh writes them), both of its sources (the
sample's base and next, relative to the bench root) and the sample's own
Word redline (its docx column) with tools/validate-docx, the
OpenXmlValidator CLI. An error signature is (error id, part family,
description with revision element names folded — w:ins/w:del become
w:rev, and a bare ins/del that names an element in a "The element 'ins'"
phrase, a `<ins>` tag or an xpath step becomes rev; a quoted 'ins'/'del'
attribute name or value is no element and stays itself — element at the
xpath's end); the part family strips a leading `redline[AB]_` and the
digits before the extension, so a renamed part still meets its source
(jubarte writes redlineB_header3.xml where a source had header3.xml).

The redline's signatures minus those either source already has are its
candidates. A candidate the sample's Word redline shares is WORD: Word's
own writing produced the same shape. The exemption is per kind, not per
occurrence — the counts print beside it (`ours N / word M`) so extra
occurrences stay visible. What remains is OWN: jubarte wrote it. A
redline the validator cannot produce or open (ABSENT, OPEN_FAILED, EXIT)
is always OWN and never WORD, whatever the references say. The tool
prints the redlines with a WORD or OWN signature and a table of the OWN
ones; a source or Word redline the validator cannot open is listed as
UNREADABLE SOURCE — the references are validated even when the redline
is clean, so the unreadable count is true — and is never counted clean
or OWN beyond a broken output (checklist before#24, #1757, #1798; it
replaces the 0.11.2 tool that lived outside any repository).

Exit 0 when no redline has an OWN signature; 1 when one has, or a redline
is missing (a sweep pointed at the wrong folder validates nothing); 2 when
the CSV is no redline sample, a named file is missing, or the validator is
not built (`dotnet build -c Release tools/validate-docx`). Plain Python 3,
stdlib only.
"""

from __future__ import annotations

import argparse
import collections
import csv
import re
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
VALIDATOR = ROOT / "tools/validate-docx/bin/Release/net8.0/validate-docx"
Kinds = collections.Counter
Signature = tuple[str, str, str, str]  # error id, part family, description, xpath tail

# a word/ header or footer a compare wrote can be renamed redline[AB]_N, and
# part numbers differ between a source and its redline: the family drops both
FAMILY = re.compile(r"(?:^|(?<=/))redline[AB]_|\d+(?=[.])")
# ins/del are the same revision shape only as element names: prefixed
# (w:ins), in a tag (<ins>), an xpath step (/ins) or an "element 'ins'"
# phrase. A quoted 'ins'/'del' attribute name or value stays itself.
FOLD_NAME = re.compile(r"\b([A-Za-z][\w.-]*):(?:ins|del)\b")
FOLD_BARE = re.compile(r"(?:(?<=<)|(?<=</)|(?<=/)|(?<=element '))(?:ins|del)\b")
# pseudo-signatures kinds() gives a file it could not validate
PSEUDO = {"ABSENT", "OPEN_FAILED", "EXIT"}


def signature(err: str, msg: str, part: str, xpath: str) -> Signature:
    """The comparison key one validator line stands for (#1798)."""
    family = FAMILY.sub("", part)
    tail = xpath.rsplit("/", 1)[-1] if xpath else ""
    tail = fold(re.sub(r"\[\d+\]", "", tail))
    # a bare ins/del tail is itself the element name, not a value
    tail = "rev" if tail in ("ins", "del") else tail
    return err, family, fold(msg), tail


def fold(text: str) -> str:
    """Fold revision element names to rev, leave other tokens alone."""
    return FOLD_BARE.sub("rev", FOLD_NAME.sub(r"\1:rev", text))


def kinds(validator: Path, path: Path) -> Kinds:
    """The validator's error signatures in PATH, counted."""
    if not path.is_file():
        return Kinds({signature("ABSENT", "", "", ""): 1})
    out = subprocess.run([str(validator), str(path)], capture_output=True, text=True)
    got = Kinds()
    for line in out.stdout.splitlines():
        cells = line.split("\t")
        if len(cells) > 1:
            cells += [""] * (5 - len(cells))
            got[signature(*cells[1:5])] += 1
    if out.returncode not in (0, 1) and not got:
        got[signature("EXIT", str(out.returncode), "", "")] += 1
    return got


def unreadable(k: Kinds) -> bool:
    """A file the validator could not open (OPEN_FAILED, EXIT)."""
    return any(s[0] in PSEUDO - {"ABSENT"} for s in k)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("sample", type=Path, help="release_info/sample_redline_<version>_<stamp>.csv")
    ap.add_argument("--root", type=Path, required=True, help="the bench root the sample's paths are relative to")
    ap.add_argument("--redlines", type=Path, required=True, help="the folder holding <key>.docx per pair")
    ap.add_argument("--validator", type=Path, default=VALIDATOR)
    ap.add_argument("--jobs", type=int, default=3)
    args = ap.parse_args()

    if not args.validator.is_file():
        print(f"lane_validity: no validator at {args.validator} — dotnet build -c Release tools/validate-docx",
              file=sys.stderr)
        return 2
    with args.sample.open(newline="", encoding="utf-8") as f:
        rows = list(csv.DictReader(f))
    if not rows or not {"key", "base", "next"} <= set(rows[0]):
        print(f"lane_validity: {args.sample} is no release_info redline sample (key, base, next)", file=sys.stderr)
        return 2
    absent = [r[c] for r in rows for c in ("base", "next", "docx")
              if r.get(c) and not (args.root / r[c]).is_file()]
    if absent:
        print(f"lane_validity: {len(absent)} named file(s) missing under {args.root}: " + ", ".join(absent[:5]),
              file=sys.stderr)
        return 2

    def one(r: dict) -> tuple[str, Kinds, Kinds, dict[Signature, tuple[int, int]], list[str]]:
        mine = kinds(args.validator, args.redlines / f"{r['key']}.docx")
        inputs: dict[str, Kinds] = {c: kinds(args.validator, args.root / r[c]) for c in ("base", "next")}
        if r.get("docx"):
            inputs["docx"] = kinds(args.validator, args.root / r["docx"])
        word = inputs.pop("docx", Kinds())
        dead = [r[c] for c, k in inputs.items() if unreadable(k)]
        if r.get("docx") and unreadable(word):
            dead.append(r["docx"])
        if not mine:
            return r["key"], mine, mine, {}, dead
        # a redline-side pseudo signature (ABSENT, OPEN_FAILED, EXIT) is
        # always its own failure — never WORD, never excused by a dead input
        broke = Kinds({s: n for s, n in mine.items() if s[0] in PSEUDO})
        sources = set().union(*inputs.values()) if inputs else set()
        left = Kinds({s: n for s, n in mine.items() if s[0] not in PSEUDO and s not in sources})
        # the WORD exemption is per kind; keep both counts so extra
        # occurrences stay visible
        shared = {s: (n, word[s]) for s, n in left.items() if s in word}
        # an unreadable input proves nothing: list the redline, report no OWN
        own = broke + (Kinds() if dead else Kinds({s: n for s, n in left.items() if s not in shared}))
        return r["key"], mine, own, shared, dead

    with ThreadPoolExecutor(max(1, args.jobs)) as pool:
        done = list(pool.map(one, rows))
    flagged = [d for d in done if d[1]]
    own = [d for d in done if d[2]]
    unread = [d for d in done if d[4]]
    print(f"{len(rows)} redlines; {len(flagged)} with validator errors; "
          f"{len(own)} with a signature neither source nor Word's redline has; "
          f"{len(unread)} with an unreadable source")
    table = Kinds()
    for _, _, new, _, _ in own:
        table.update(new.keys())
    for (err, family, msg, tail), n in table.most_common():
        print(f"  {n:4} redlines  {err}  {family}  {tail}  {msg[:80]}")
    for key, _, new, shared, dead in done:
        if shared:
            print("WORD", key, {s: f"ours {o} / word {w}" for s, (o, w) in shared.items()})
        if new:
            print("OWN", key, dict(new))
        for name in dead:
            print("UNREADABLE SOURCE", key, name)
    return 1 if own else 0


if __name__ == "__main__":
    sys.exit(main())

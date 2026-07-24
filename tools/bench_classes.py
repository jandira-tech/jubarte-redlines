#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Bucket sub-90 bench fixtures into defect classes.

Usage: python3 tools/bench_classes.py [bench.jsonl]

Reads the two latest jubarte-rust script_redlines rows (164 = word_based,
196 = randomized) and groups every fixture with overall_score < 90.
"""
from __future__ import annotations

import json
import os
import sys
from collections import defaultdict
from pathlib import Path

# Default: sibling of this checkout (or BENCH_DIR). No developer-home hardcode.
# Explicit BENCH_DIR must not fall through to unrelated relative paths (CR #3645867376).
_here = Path(__file__).resolve().parent
_crate = _here.parent
if "BENCH_DIR" in os.environ:
    _bench_root = Path(os.environ["BENCH_DIR"])
    _default_candidates = [_bench_root / "results" / "bench.jsonl"]
else:
    _bench_root = _crate.parent / "neurotic_docx_bench"
    _default_candidates = [
        _bench_root / "results" / "bench.jsonl",
        Path("../neurotic_docx_bench/results/bench.jsonl"),
        Path("../../neurotic_docx_bench/results/bench.jsonl"),
    ]
if len(sys.argv) > 1:
    path = Path(sys.argv[1])
else:
    path = next((p for p in _default_candidates if p.is_file()), _default_candidates[0])
rows = [json.loads(line) for line in path.open() if line.strip()]
jr = [
    r
    for r in rows
    if r.get("vendor") == "jubarte-rust" and r.get("benchmark") == "script_redlines"
]
# Prefer the two corpora used by the Ratchet-1 gate (164 word_based, 196
# randomized); fall back to any n_docs if those are absent.
latest: dict[int, dict] = {}
for r in jr:
    n = r.get("n_docs")
    if n is not None:
        latest[n] = r
if 164 in latest or 196 in latest:
    latest = {k: v for k, v in latest.items() if k in (164, 196)}


def classify(stem: str, d: dict) -> str:
    if d.get("page_count_oracle") != d.get("page_count_candidate"):
        return "C1-page-structure"
    if "comments" in stem:
        return "C2-comments"
    if "word_tolerated" in stem or "repaired" in stem:
        return "C3-tolerated-input"
    if "suggesting" in stem:
        return "C4-preexisting-revisions"
    # one-pager formatting-only heuristic: same page count already, short stems
    # with style/demo tokens → C5
    style_tokens = (
        "bold",
        "italic",
        "centered",
        "aligned",
        "heading",
        "quarterly",
        "demo",
        "subtitle",
        "red_",
        "blue_",
        "spacing",
    )
    if any(t in stem for t in style_tokens):
        return "C5-formatting"
    return "C5-content-diff"


buckets_all: dict[str, list[tuple[str, float, str, str]]] = defaultdict(list)

for n, r in sorted(latest.items()):
    corpus = "word_based" if n == 164 else ("randomized" if n == 196 else f"n={n}")
    buckets: dict[str, list] = defaultdict(list)
    for stem, d in (r.get("per_doc") or {}).items():
        s = d.get("overall_score")
        if s is None or s >= 90:
            continue
        cls = classify(stem, d)
        pages = f'{d.get("page_count_oracle", "?")}/{d.get("page_count_candidate", "?")}'
        buckets[cls].append((round(float(s), 2), stem, pages))
        buckets_all[cls].append((corpus, round(float(s), 2), stem, pages))

    print(f"\n== {corpus} (n={n}, {r.get('tool_version', '?')}) ==")
    for c in sorted(buckets):
        mem = sorted(buckets[c])
        print(f"  {c}: {len(mem)} fixtures")
        for s, stem, pages in mem:
            print(f"    {s:6}  p{pages}  {stem[:80]}")

print("\n== AGGREGATE class membership ==")
for c in sorted(buckets_all):
    print(f"  {c}: {len(buckets_all[c])} fixture-instances")

#!/usr/bin/env -S uv run --script

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

# SPDX-License-Identifier: AGPL-3.0-only

# /// script
# dependencies = ["tree-sitter", "tree-sitter-rust"]
# ///
"""Candidate duplicated logic in a Rust crate, from tree-sitter syntax trees.
Usage: find_duplicates.py <repo-root> [candidates.json]. Writes docs/analysis/DUPLICATE_CANDIDATES.md (needs docs/symbols/symbols.json
from gen_symbol_manifest.py); the curated findings in DUPLICATED_LOGIC.md are written by hand from this output.
"""
import json, sys, hashlib
from collections import defaultdict
from pathlib import Path
import tree_sitter_rust as tsr
from tree_sitter import Language, Parser

ROOT = Path(sys.argv[1]).resolve()
OUT = Path(sys.argv[2]) if len(sys.argv) > 2 else None
MD = ROOT / 'docs' / 'analysis' / 'DUPLICATE_CANDIDATES.md'
LANG = Language(tsr.language()); parser = Parser(LANG)
sym = json.loads((ROOT / "docs/symbols/symbols.json").read_text())["symbols"]
test_lines = defaultdict(list)  # not needed; symbols.json already flags tests
prod = {(s["file"], s["line"]): s for s in sym if s["kind"] in ("function", "method") and not s["test"]}

LITS = {"string_literal", "raw_string_literal", "integer_literal", "float_literal", "char_literal", "boolean_literal"}
def seq(node, ident=False):
    out = []
    stack = [node]
    while stack:
        n = stack.pop()
        t = n.type
        if t in ("line_comment", "block_comment"): continue
        if t in LITS: out.append("LIT"); continue
        if t in ("identifier", "field_identifier", "type_identifier"):
            out.append("ID:" + n.text.decode() if ident else "ID"); continue
        out.append(t)
        stack.extend(reversed(n.children))
    return out

funcs = []
for p in sorted((ROOT / "src").rglob("*.rs")):
    rel = str(p.relative_to(ROOT)); src = p.read_bytes()
    tree = parser.parse(src)
    stack = [tree.root_node]
    while stack:
        n = stack.pop()
        if n.type == "function_item":
            s = prod.get((rel, n.start_point[0] + 1))
            body = n.child_by_field_name("body")
            if s is not None and body is not None:
                sq = seq(body)
                funcs.append({"id": s["i"], "name": s["name"], "qual": s["qualified"], "file": rel, "line": s["line"],
                              "end": s["end_line"], "kind": s["kind"], "sig": s["signature"], "seq": sq,
                              "text": src[body.start_byte:body.end_byte].decode("utf8", "replace")})
        stack.extend(n.children)

MIN_NODES = 40
big = [f for f in funcs if len(f["seq"]) >= MIN_NODES]

# 1. type-2 clones: identical normalized tree
groups = defaultdict(list)
for f in big:
    groups[hashlib.sha1(" ".join(f["seq"]).encode()).hexdigest()].append(f)
exact = [g for g in groups.values() if len(g) > 1]

# 2. near clones: Jaccard of node-type 4-grams
def shingles(sq, k=4): return {hash(tuple(sq[i:i + k])) for i in range(len(sq) - k + 1)}
sh = {f["id"]: shingles(f["seq"]) for f in big}
inv = defaultdict(list)
for f in big:
    for h in sh[f["id"]]: inv[h].append(f["id"])
byid = {f["id"]: f for f in big}
cand = defaultdict(int)
for h, ids in inv.items():
    if 1 < len(ids) <= 12:
        for i in range(len(ids)):
            for j in range(i + 1, len(ids)): cand[ids[i], ids[j]] += 1
exact_pairs = set()
for g in exact:
    for a in g:
        for b in g: exact_pairs.add((a["id"], b["id"]))
near = []
for (a, b) in cand.keys():
    if (a, b) in exact_pairs: continue
    A, B = sh[a], sh[b]
    if min(len(A), len(B)) < 25: continue
    j = len(A & B) / len(A | B)
    if j >= 0.80 and f'{byid[a]["file"]}{byid[a]["line"]}' != f'{byid[b]["file"]}{byid[b]["line"]}':
        near.append((j, a, b))
near.sort(reverse=True)

# 3. same name defined in several places (free functions, and methods of different types)
by_name = defaultdict(list)
for s in sym:
    if s["kind"] in ("function", "method") and not s["test"] and s["name"] not in ("new", "default", "fmt", "from", "into", "eq", "hash", "clone", "next", "drop", "deref", "try_from", "main", "serialize", "deserialize", "from_str", "as_ref", "borrow", "index", "build", "run", "get", "set", "len", "is_empty", "parse", "name", "value", "kind", "id", "text", "write", "read", "iter", "visit", "apply", "finish", "reset", "push", "pop", "insert", "remove", "contains", "clear"):
        by_name[s["name"]].append(s)
same_name = {k: v for k, v in by_name.items() if len({(x["file"], x.get("impl_type", "")) for x in v}) > 1 and len(v) <= 8}

# 4. same concept, different spelling: sorted word set of the snake_case name
def words(n): return tuple(sorted(w for w in n.lower().split("_") if w))
concept = defaultdict(list)
for s in sym:
    if s["kind"] in ("function", "method") and not s["test"] and len(words(s["name"])) >= 2:
        concept[words(s["name"])].append(s)
concept = {k: v for k, v in concept.items() if len({x["name"] for x in v}) > 1}

def brief(f): return {"qual": f["qual"], "file": f["file"], "line": f["line"], "end": f["end"], "nodes": len(f["seq"]), "sig": f["sig"]}
res = {
    "exact": [[brief(f) for f in g] for g in sorted(exact, key=lambda g: -len(g[0]["seq"]))],
    "near": [{"jaccard": round(j, 3), "a": brief(byid[a]), "b": brief(byid[b])} for j, a, b in near],
    "same_name": {k: [{"qual": x["qualified"], "file": x["file"], "line": x["line"], "sig": x["signature"], "doc": x["doc"]} for x in v] for k, v in same_name.items()},
    "concept": {" ".join(k): [{"name": x["name"], "qual": x["qualified"], "file": x["file"], "line": x["line"], "sig": x["signature"], "doc": x["doc"]} for x in v] for k, v in concept.items()},
}
if OUT: OUT.write_text(json.dumps(res))
def loc(x): return f"`{x['qual']}` ({x['file'].replace('src/', '')}:{x['line']})"
L = ["# Duplicate-logic candidates (tool output)\n",
     "Generated by `scripts/find_duplicates.py` from tree-sitter syntax trees. These are *candidates*: a human read decides which are the same job done two ways. The curated findings are in [DUPLICATED_LOGIC.md](DUPLICATED_LOGIC.md).\n",
     f"Functions with a body of at least {MIN_NODES} syntax nodes (tests excluded): {len(big)}.\n",
     "## 1. Identical bodies after renaming identifiers and literals\n",
     "Groups of functions whose syntax trees are the same once names and literal values are ignored. Cross-file groups first; same-file groups are usually x/y or u16/u32 siblings.\n"]
cross = [g for g in res["exact"] if len({f["file"] for f in g}) > 1]
same = [g for g in res["exact"] if len({f["file"] for f in g}) == 1]
L.append(f"### Across files ({len(cross)} groups)\n")
L.extend("- " + "; ".join(loc(f) for f in g) + f" ({g[0]['nodes']} nodes)" for g in cross)
L.append(f"\n### Within one file ({len(same)} groups)\n")
L.extend("- " + "; ".join(loc(f) for f in g) + f" ({g[0]['nodes']} nodes)" for g in same)
L += ["", "## 2. Near-identical bodies in different files (4-gram Jaccard of node types >= 0.80)\n"]
for p in res["near"]:
    if p["a"]["file"] != p["b"]["file"]:
        L.append(f"- {p['jaccard']}: {loc(p['a'])} ~ {loc(p['b'])}")
L += ["", "## 3. Same words in the name, different spelling or order\n",
      "Function names with the same set of words but a different name (`has_ancestor` / `ancestor_has`). Includes inverse pairs such as `rgb_to_hsl` / `hsl_to_rgb`, which are not duplicates.\n"]
for k, v in res["concept"].items():
    L.append("- " + "; ".join(f"{loc(x | {'qual': x['qual']})}" for x in v))
L += ["", "## 4. The same name defined in three or more files\n"]
for k, v in sorted(res["same_name"].items(), key=lambda kv: -len({x['file'] for x in kv[1]})):
    fl = {x['file'] for x in v}
    if len(fl) >= 3: L.append(f"- `{k}`: " + ", ".join(f"{x['file'].replace('src/', '')}:{x['line']}" for x in v))
MD.parent.mkdir(parents=True, exist_ok=True); MD.write_text("\n".join(L) + "\n")
if OUT: OUT.write_text(json.dumps(res))
print({"functions>=%d nodes" % MIN_NODES: len(big), "exact_groups": len(exact), "near_pairs": len(near), "same_name": len(same_name), "concept": len(concept)})

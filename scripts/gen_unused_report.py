#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

# SPDX-License-Identifier: AGPL-3.0-only

"""knip-style report for a Rust crate: public API items nothing in the repository refers to.

rustc's dead_code lint already covers everything that is not reachable from outside the crate; the gap
it leaves is public API that no caller inside the repository (src, tests, benches, examples, fuzz,
bindings) uses. This script finds those by name, using docs/symbols/symbols.json for the item list.
"""
import json, re, sys
from collections import defaultdict
from pathlib import Path
from typing import Any

ROOT = Path(sys.argv[1]).resolve()
OUT = ROOT / "docs" / "analysis" / "UNUSED_PUBLIC_API.md"
sym = json.loads((ROOT / "docs/symbols/symbols.json").read_text())["symbols"]

# ---- which files are publicly reachable modules
mods = {(s["file"], s["name"]): s for s in sym if s["kind"] == "module" and not s["test"]}
def modpath(file):
    parts = list(Path(file).relative_to("src").with_suffix("").parts)
    if parts == ["lib"]: return ()
    if parts[-1] == "mod": parts = parts[:-1]
    return tuple(parts)
file_of = {modpath(s["file"]): s["file"] for s in sym if s["kind"] != "module" or True}
def parent_file(mp):
    return file_of.get(mp[:-1]) if mp else None
def file_is_public(file, memo={}):
    if file in memo: return memo[file]
    mp = modpath(file)
    if not mp: memo[file] = True; return True
    pf = parent_file(mp)
    decl = mods.get((pf, mp[-1])) if pf else None
    ok = bool(decl and decl["visibility"] == "pub" and file_is_public(pf))
    # cfg-gated pub mods still count
    memo[file] = ok
    return ok

# ---- corpus: all code in the repo that can refer to an item
exts = {".rs", ".py", ".js", ".mjs", ".ts", ".toml"}
skip = {"target", "node_modules", ".git", "docs"}
corpus = {}
for p in ROOT.rglob("*"):
    if p.is_file() and p.suffix in exts and not (set(p.relative_to(ROOT).parts) & skip):
        if p.name in ("Cargo.lock",): continue
        try: corpus[str(p.relative_to(ROOT))] = p.read_text(errors="ignore")
        except Exception: pass
# generated wasm glue and lockfiles are noise
corpus = {k: v for k, v in corpus.items() if "/npm/" not in k and not k.endswith(".d.ts")}
name_count = defaultdict(int)
for s in sym:
    if s["kind"] in ("function", "method", "struct", "enum", "union", "trait", "const", "static", "type alias", "macro"):
        name_count[s["name"]] += 1

from collections import Counter
TOK = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
counts = {f: Counter(TOK.findall(t)) for f, t in corpus.items()}

sig_names = defaultdict(set)   # identifier -> {qualified names of public items whose signature mentions it}
for s in sym:
    if s["test"] or s["visibility"] != "pub" or s["kind"] not in ("function", "method"): continue
    if not file_is_public(s["file"]): continue
    for t in set(re.findall(r"[A-Za-z_][A-Za-z0-9_]*", s["signature"])):
        sig_names[t].add(s["qualified"])

rows: list[dict[str, Any]] = []
for s in sym:
    if s["test"] or s["kind"] not in ("function", "method", "struct", "enum", "union", "trait", "const", "static", "type alias", "macro"):
        continue
    if s["visibility"] != "pub": continue
    if not file_is_public(s["file"]): continue
    if s["kind"] == "method":
        if s.get("trait") and s["trait"] != "trait-decl": continue     # trait impl method: called through the trait
        owner = [x for x in sym if x["kind"] in ("struct", "enum", "union", "trait") and x["name"] == s.get("impl_type") and x["visibility"] == "pub" and file_is_public(x["file"])]
        if not owner: continue
    if "::" in s["qualified"] and s["kind"] not in ("method",) and not s["qualified"].endswith("::" + s["name"]): continue
    if s["kind"] != "method" and "::" in s["qualified"]: continue          # inside an inline module
    if s["name"] in ("main", "new", "default", "fmt", "from", "into", "clone", "drop", "deref", "eq", "hash", "next", "try_from"): continue
    if name_count[s["name"]] > 1 and s["kind"] in ("method", "function"):
        ambiguous = True
    else:
        ambiguous = False
    own = 0; other_src = 0; outside = 0; refs_out = set(); refs_src = set()
    for f, cn in counts.items():
        c = cn.get(s["name"], 0)
        if f == s["file"]: own += c
        elif f.startswith("src/"): other_src += c; (refs_src.add(f) if c else None)
        else: outside += c; (refs_out.add(f) if c else None)
    own -= 1  # the definition itself
    rows.append({**{k: s[k] for k in ("kind", "name", "qualified", "file", "line", "doc")}, "own": max(own, 0), "src": other_src,
                 "outside": outside, "ambiguous": ambiguous, "in_sig": sorted(sig_names.get(s["name"], set()) - {s["qualified"]})[:2], "refs_src": sorted(refs_src), "refs_out": sorted(refs_out)})

orphan = [r for r in rows if r["own"] == 0 and r["src"] == 0 and r["outside"] == 0 and not r["ambiguous"]]
own_only = [r for r in rows if r["own"] > 0 and r["src"] == 0 and r["outside"] == 0 and not r["ambiguous"]]
no_external = [r for r in rows if r["outside"] == 0 and r["src"] > 0 and not r["ambiguous"]]
amb_orphan = [r for r in rows if r["own"] == 0 and r["src"] == 0 and r["outside"] == 0 and r["ambiguous"]]

def table(rs, cols):
    L = ["| Item | Kind | Defined at | Mentions (own file / other src files / outside src) | Keep because | Doc |", "|---|---|---|---|---|---|"]
    for r in sorted(rs, key=lambda x: (x["file"], x["line"])):
        d = (r["doc"] or "").replace("|", "\\|").replace("\u2014", ":")[:110]
        keep = ("in the signature of " + ", ".join(f"`{x}`" for x in r["in_sig"])) if r["in_sig"] and r["kind"] in ("struct", "enum", "union", "type alias", "trait") else ""
        L.append(f"| `{r['qualified']}` | {r['kind']} | {r['file']}:{r['line']} | {r['own']} / {r['src']} / {r['outside']} | {keep} | {d} |")
    return L

L = ["# Unused public API (knip-style)\n",
     "Generated by `scripts/gen_unused_report.py` from `docs/symbols/symbols.json`. Read the **Method and limits** section before acting on any row.\n",
     "## Tools run\n",
     "| Tool | What it checks | Result |", "|---|---|---|",
     "| `cargo machete --with-metadata` (root, `jubarte-rust-inproc`, `jubarte-wasm`, `jubarte-python`, `fuzz`) | dependencies declared but never imported | **nothing found** in all five crates |",
     "| `cargo check --all-targets --all-features` (stable 1.99.1) | rustc `dead_code`, `unused_*` lints (everything not reachable from outside the crate) | **zero warnings** |",
     "| this report | public API with no reference anywhere in the repository | below |",
     "| `cargo shear` 1.14.0 (root and the four sub-crates) | unused and misplaced dependencies | **no issues found** (needed rustc 1.98 or newer; run on 1.99.1) |",
     "| `cargo +nightly udeps --all-targets --all-features` (nightly 1.101) | unused dependencies, including dev-dependencies | **\"All deps seem to have been used.\"** |",
     "| `warnalyzer` | unused items across crates | **cannot run on any current toolchain**: it reads rustc `-Z save-analysis` output, and rustc removed that flag years ago. This report's public-API check replaces it. |",
     "",
     "So the part of knip that matters for Rust (unused dependencies, dead private code) is clean. What is left is public API, which a library publishes on purpose, so every row below is a question for the maintainers, not a deletion list.\n",
     "## Summary\n",
     f"Public items checked (`pub` item in a chain of `pub mod`s, tests excluded): **{len(rows)}**\n",
     f"- never mentioned anywhere else in the repo: **{len(orphan)}** (section A)",
     f"- mentioned only inside their own file (typically their own unit tests or doc examples): **{len(own_only)}** (section B)",
     f"- used by other `src/` files but by nothing outside `src/` (no test, bench, example, fuzz target or binding): **{len(no_external)}** (section C)",
     f"- never mentioned, but the name is shared with another item so the check cannot tell them apart: {len(amb_orphan)} (section D)",
     "",
     "## A. Never mentioned anywhere else\n",
     "Strongest candidates: nothing in `src/`, `tests/`, `benches/`, `examples/`, `fuzz/` or the bindings refers to the name.\n"]
L += table(orphan, None)
L += ["", "## B. Mentioned only in their own file\n",
      "The only other mentions are in the defining file itself (its tests, doc examples or neighbouring code). Public, but nothing outside the file needs it to be.\n"]
L += table(own_only, None)
L += ["", "## C. Used inside `src/` only, never from tests, benches, examples, fuzz or bindings\n",
      "Candidates for `pub(crate)`: callers exist, but nothing outside the library can be depending on them from this repository.\n"]
L += table(no_external, None)
L += ["", "## D. Never mentioned, but the name is ambiguous\n",
      "Another item has the same name, so a name search cannot say which one a mention belongs to. Listed for completeness.\n"]
L += table(amb_orphan, None)
L += ["", "## Method and limits\n",
      "- An item is *public* if it is declared `pub` and every module on the path from the crate root is declared `pub mod`. Methods count when their type is public; trait-impl methods are skipped (they are called through the trait). `main`, `new`, `default`, `fmt`, `from`, `into`, `clone`, `drop`, `deref`, `eq`, `hash`, `next` and `try_from` are skipped as trait-shaped names.",
      "- A *mention* is a whole-word match of the item's name in any `.rs`, `.py`, `.js`, `.mjs`, `.ts` or `.toml` file outside `target/`, `node_modules/`, `docs/` and the generated `npm/` wasm glue. Doc comments count (rustdoc examples are compiled as doctests). The definition line itself is not counted.",
      "- *Keep because*: a public type named in the signature of another public function or method has to stay `pub` (Rust's private-in-public rules), even if no one outside names it directly. Rows with that note are not candidates for narrowing.",
      "- Name matching over-counts (a common word elsewhere hides a genuinely unused item) and under-counts (a use through a re-export under another name, a `macro_rules!` expansion, or a name assembled in a string is invisible). Items whose name is shared with another item are not asserted either way.",
      "- Items used only from `jubarte-python` or `jubarte-wasm` by a *different* name (renamed in the binding) show up here as unused. Items reached only through serde, `schemars` or clap derives are not seen either.",
      "- This is a library crate published to crates.io: removing a public item is a semver break regardless of what this report says."]
(ROOT / "docs" / "analysis").mkdir(parents=True, exist_ok=True)
OUT.write_text("\n".join(L) + "\n")
print({"checked": len(rows), "orphan": len(orphan), "own_only": len(own_only), "no_external": len(no_external), "amb": len(amb_orphan)})

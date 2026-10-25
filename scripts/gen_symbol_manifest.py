#!/usr/bin/env -S uv run --script

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

# SPDX-License-Identifier: AGPL-3.0-only

# /// script
# dependencies = ["tree-sitter", "tree-sitter-rust"]
# ///
"""Static symbol manifest for a Rust crate's src/ tree, built with tree-sitter.

Usage: symbols.py <repo-root> [--out docs/symbols]
Writes per-kind manifests, one per-file function manifest, and symbols.json.
No type information is available, so call resolution is name based and every
resolved edge carries a confidence tag (exact / by-name / ambiguous / external).
"""
from typing import Any
import json, re, sys
from collections import defaultdict
from pathlib import Path
import tree_sitter_rust as tsr
from tree_sitter import Language, Parser

ROOT = Path(sys.argv[1]).resolve()
OUT = ROOT / (sys.argv[sys.argv.index("--out") + 1] if "--out" in sys.argv else "docs/symbols")
SRC = ROOT / "src"
LANG = Language(tsr.language())
parser = Parser(LANG)

KEYWORDS = {"if", "while", "for", "match", "loop", "return", "let", "in", "else", "move", "as", "fn", "mut", "ref",
            "unsafe", "async", "await", "break", "continue", "where", "impl", "dyn", "pub", "use", "mod", "struct",
            "enum", "trait", "type", "const", "static", "crate", "super", "self", "Self"}
KIND_NODES = {
    "struct_item": "struct", "enum_item": "enum", "union_item": "union", "trait_item": "trait",
    "type_item": "type alias", "const_item": "const", "static_item": "static", "macro_definition": "macro",
    "mod_item": "module", "impl_item": "impl",
}

def txt(n): return n.text.decode("utf8", "replace") if n is not None else ""

def mod_path(p: Path):
    parts = list(p.relative_to(SRC).with_suffix("").parts)
    if parts == ["lib"]: return ()
    if parts and parts[-1] == "mod": parts = parts[:-1]
    return tuple(parts)

def first_sentence(lines, limit=320):
    s = " ".join(l.strip() for l in lines).strip()
    s = re.sub(r"\s+", " ", s)
    if not s: return ""
    m = re.match(r"(.+?[.!?])(\s|$)", s)
    out = m.group(1) if m else s
    return out if len(out) <= limit else out[: limit - 1].rstrip() + "..."

def doc_before(node, src_lines):
    """Outer doc comment (///) directly above node (skipping attributes); fallback plain // comment."""
    docs, plain = [], []
    prev = node.prev_sibling
    while prev is not None and prev.type in ("attribute_item", "line_comment", "block_comment"):
        t = txt(prev)
        if prev.type == "line_comment":
            if t.startswith("///") and not t.startswith("////"): docs.insert(0, t[3:])
            elif t.startswith("//!"): break
            elif t.startswith("//"): plain.insert(0, t[2:])
        elif prev.type == "block_comment" and t.startswith("/**"):
            docs.insert(0, t[3:-2])
        prev = prev.prev_sibling
    d = first_sentence(docs)
    if d: return d, "doc"
    p = first_sentence(plain)
    return (p, "comment") if p else ("", "none")

def attrs_before(node):
    out, prev = [], node.prev_sibling
    while prev is not None and prev.type in ("attribute_item", "line_comment", "block_comment"):
        if prev.type == "attribute_item": out.insert(0, re.sub(r"\s+", "", txt(prev)))
        prev = prev.prev_sibling
    return out

def vis_of(node):
    for c in node.children:
        if c.type == "visibility_modifier": return txt(c)
    return ""

def type_head(n):
    """Impl target type text without generics, e.g. 'Foo' or 'a::Foo'."""
    if n is None: return ""
    if n.type == "generic_type": n = n.child_by_field_name("type") or n
    if n.type == "reference_type": return "&" + type_head(n.child_by_field_name("type"))
    return re.sub(r"<.*", "", txt(n)).strip()

def signature(node, src):
    body = node.child_by_field_name("body")
    end = body.start_byte if body is not None else node.end_byte
    s = src[node.start_byte:end].decode("utf8", "replace")
    return re.sub(r"\s+", " ", s).strip().rstrip(";").strip()


STD_TYPES = {"Vec", "String", "str", "Option", "Result", "HashMap", "HashSet", "BTreeMap", "BTreeSet", "VecDeque", "Cow",
             "Path", "PathBuf", "Rc", "Arc", "RefCell", "Cell", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16",
             "i32", "i64", "i128", "isize", "f32", "f64", "bool", "char", "Self", "Range", "Instant", "Duration", "Error"}
STD_METHODS = set("""iter iter_mut into_iter map filter filter_map flat_map collect fold any all find position count enumerate
zip rev skip take chain cloned copied peekable next last min max sum product get get_mut insert remove push pop push_str
extend len is_empty contains contains_key entry or_insert or_default or_insert_with keys values clone to_string to_owned
to_vec as_str as_ref as_mut as_slice as_bytes as_deref unwrap expect unwrap_or unwrap_or_else unwrap_or_default ok err
ok_or ok_or_else map_err and_then or_else is_some is_none is_ok is_err take replace first last_mut split splitn trim
starts_with ends_with join concat parse bytes chars lines read write flush sort sort_by sort_by_key dedup retain drain
truncate clear borrow borrow_mut into from try_into try_from default cmp partial_cmp eq ne hash fmt abs min_by max_by
saturating_sub saturating_add checked_add checked_sub wrapping_add iter_mut windows chunks binary_search with_capacity
reserve resize swap split_at find_map flatten unzip rposition skip_while take_while step_by is_some_and map_or
map_or_else get_or_insert_with then_some then lock unwrap_err downcast set write_all read_to_end seek lines trim_start
trim_end strip_prefix strip_suffix split_once rsplit char_indices to_lowercase to_uppercase eq_ignore_ascii_case
is_ascii_digit is_alphabetic is_whitespace is_empty copy_from_slice extend_from_slice first_mut last iter_mut name value
finish""".split())

def norm_type(t):
    t = re.sub(r"\s+", " ", t or "").strip()
    t = re.sub(r"^(&\s*('\w+\s+)?(mut\s+)?|\*(const|mut)\s+)+", "", t)
    t = re.sub(r"^dyn\s+|^impl\s+", "", t)
    m = re.match(r"(?:Box|Rc|Arc)<(.+)>$", t)
    if m: return norm_type(m.group(1))
    if t.startswith("[") or t.startswith("("): return "[]"
    t = re.sub(r"<.*", "", t)
    return t.split("::")[-1].strip()

class Item:
    __slots__ = (
        "attrs",
        "calls",
        "doc",
        "docsrc",
        "end",
        "env",
        "file",
        "has_body",
        "id",
        "impl_type",
        "is_test",
        "kind",
        "line",
        "modpath",
        "name",
        "parent",
        "qual",
        "sig",
        "trait",
        "vis",
    )
    # Every slot is set in __init__, None when the caller leaves it out.
    attrs: Any
    calls: Any
    doc: Any
    docsrc: Any
    end: Any
    env: Any
    file: Any
    has_body: Any
    id: Any
    impl_type: Any
    is_test: Any
    kind: Any
    line: Any
    modpath: Any
    name: Any
    parent: Any
    qual: Any
    sig: Any
    trait: Any
    vis: Any
    def __init__(self, **kw):
        for k in self.__slots__: setattr(self, k, kw.get(k))

items = []
STRUCT_FIELDS = defaultdict(dict)
files = sorted(p for p in SRC.rglob("*.rs"))
FILE_TEST = {}

# test-only files: declared by `#[cfg(test)] mod x;` from a parent
mod_decl_test = set()
for p in files:
    src = p.read_bytes()
    tree = parser.parse(src)
    for n in tree.root_node.children:
        if n.type == "mod_item" and n.child_by_field_name("body") is None:
            if any(a == "#[cfg(test)]" for a in attrs_before(n)):
                nm = txt(n.child_by_field_name("name"))
                base = mod_path(p)
                mod_decl_test.add((*base, nm))
FILE_TEST = {p: any(mod_path(p)[:k] in mod_decl_test for k in range(1, len(mod_path(p)) + 1)) for p in files}

class Walker:
    def __init__(self, path, src):
        self.path, self.src = path, src
        self.rel = str(path.relative_to(ROOT))
        self.base = mod_path(path)
        self.src_lines = src.decode("utf8", "replace").splitlines()

    def walk(self, node, mods, impl, fn, test):
        for c in node.children:
            self.visit(c, mods, impl, fn, test)

    def mk(self, node, kind, name, mods, impl, test, parent, **kw):
        doc, dsrc = doc_before(node, self.src_lines)
        it = Item(kind=kind, name=name, file=self.rel, line=node.start_point[0] + 1, end=node.end_point[0] + 1,
                  vis=vis_of(node), doc=doc, docsrc=dsrc, is_test=test or FILE_TEST[self.path],
                  impl_type=norm_type(impl[0]) if impl else "", trait=impl[1] if impl else "", modpath=self.base + tuple(mods),
                  calls=[], parent=parent, attrs=attrs_before(node), **kw)
        items.append(it)
        return it

    def visit(self, node, mods, impl, fn, test):
        t = node.type
        if t == "mod_item":
            name = txt(node.child_by_field_name("name"))
            attrs = attrs_before(node)
            tst = test or any(a == "#[cfg(test)]" for a in attrs)
            body = node.child_by_field_name("body")
            self.mk(node, "module", name, mods, impl, tst, None, sig=f"mod {name}", qual=name, has_body=body is not None)
            if body is not None: self.walk(body, [*mods, name], None, None, tst)
            return
        if t == "impl_item":
            ty = type_head(node.child_by_field_name("type"))
            tr = type_head(node.child_by_field_name("trait"))
            tst = test or any(a.startswith("#[cfg(test)") for a in attrs_before(node))
            label = f"<{ty} as {tr}>" if tr else ty
            self.mk(node, "impl", label, mods, None, tst, None, sig=signature(node, self.src), qual=label, has_body=True)
            body = node.child_by_field_name("body")
            if body is not None: self.walk(body, mods, (ty, tr), None, tst)
            return
        if t == "trait_item":
            name = txt(node.child_by_field_name("name"))
            tst = test or any(a.startswith("#[cfg(test)") for a in attrs_before(node))
            self.mk(node, "trait", name, mods, None, tst, None, sig=signature(node, self.src), qual=name, has_body=True)
            body = node.child_by_field_name("body")
            if body is not None: self.walk(body, mods, (name, "trait-decl"), None, tst)
            return
        if t in ("function_item", "function_signature_item"):
            name = txt(node.child_by_field_name("name"))
            attrs = attrs_before(node)
            tst = test or any(a == "#[test]" or a.startswith("#[cfg(test)") or "::test]" in a for a in attrs)
            kind = "method" if impl else "function"
            if impl and impl[1] == "trait-decl": qual = f"{impl[0]}::{name}"
            elif impl and impl[1]: qual = f"<{impl[0]} as {impl[1]}>::{name}"
            elif impl: qual = f"{impl[0]}::{name}"
            else: qual = "::".join([*mods, name]) if mods else name
            if fn is not None: qual = f"{fn.qual}::{{fn {name}}}"; kind = "function"
            it = self.mk(node, kind, name, mods, impl if fn is None else None, tst, fn.id if fn else None,
                         sig=signature(node, self.src), qual=qual,
                         has_body=node.child_by_field_name("body") is not None)
            it.id = f"{self.rel}::{qual}@{it.line}"
            body = node.child_by_field_name("body")
            if body is not None:
                self.collect_calls(body, it)
                self.walk(body, mods, None if fn is None and False else impl, it, tst)
            return
        if t in KIND_NODES and t not in ("mod_item", "impl_item", "trait_item"):
            nm = node.child_by_field_name("name")
            name = txt(nm) if nm is not None else ""
            if t == "macro_definition": name = txt(node.child_by_field_name("name"))
            tst = test or any(a.startswith("#[cfg(test)") for a in attrs_before(node))
            qual = "::".join([*mods, name]) if mods else name
            if impl: qual = f"{impl[0]}::{name}"
            it = self.mk(node, KIND_NODES[t], name, mods, impl, tst, fn.id if fn else None, sig=signature(node, self.src), qual=qual, has_body=True)
            it.id = f"{self.rel}::{qual}@{it.line}"
            if t == "enum_item":
                # variants are listed in the signature-less enum body; record them as a compact attr
                body = node.child_by_field_name("body")
                vs = [txt(v.child_by_field_name("name")) for v in (body.children if body else []) if v.type == "enum_variant"]
                it.attrs = [*it.attrs, f"variants={','.join(vs)}"]
            if t == "struct_item":
                body = node.child_by_field_name("body")
                if body is not None and body.type == "field_declaration_list":
                    for f in body.children:
                        if f.type == "field_declaration":
                            STRUCT_FIELDS[name][txt(f.child_by_field_name("name"))] = norm_type(txt(f.child_by_field_name("type")))
                    fs = [txt(f.child_by_field_name("name")) for f in body.children if f.type == "field_declaration"]
                    it.attrs = [*it.attrs, f"fields={','.join(fs)}"]
            return
        # other nodes: descend (macro_rules bodies are token trees; skip)
        if t in ("token_tree",): return
        for c in node.children:
            self.visit(c, mods, impl, fn, test)

    # ---- call collection (stops at nested fn items; nested closures belong to the outer fn)
    def collect_calls(self, node, it):
        env = {}
        fnode = node.parent
        params = fnode.child_by_field_name("parameters") if fnode is not None else None
        if params is not None:
            for prm in params.children:
                if prm.type == "parameter":
                    pat = prm.child_by_field_name("pattern")
                    if pat is not None and pat.type in ("identifier", "mut_pattern"):
                        env[txt(pat).replace("mut ", "").strip()] = norm_type(txt(prm.child_by_field_name("type")))
        it.env = env
        stack = [node]
        while stack:
            n = stack.pop()
            t = n.type
            if t == "function_item": continue
            if t == "let_declaration":
                pat = n.child_by_field_name("pattern"); ty = n.child_by_field_name("type"); val = n.child_by_field_name("value")
                if pat is not None and pat.type in ("identifier", "mut_pattern"):
                    nm = txt(pat).replace("mut ", "").strip(); tt = ""
                    if ty is not None: tt = norm_type(txt(ty))
                    elif val is not None:
                        v = val
                        while v is not None and v.type in ("try_expression", "reference_expression", "await_expression"):
                            v = v.child_by_field_name("value") or (v.children[0] if v.children else None)
                        if v is not None and v.type == "struct_expression": tt = norm_type(txt(v.child_by_field_name("name")))
                        elif v is not None and v.type == "call_expression":
                            fcall = v.child_by_field_name("function")
                            if fcall is not None and fcall.type == "scoped_identifier":
                                q = txt(fcall.child_by_field_name("path")).split("::")[-1]
                                if q[:1].isupper() and q != "Self": tt = norm_type(q)
                                elif q == "Self" and it.impl_type: tt = it.impl_type
                    if tt: env[nm] = tt
            if t == "call_expression":
                f = n.child_by_field_name("function")
                self.record_call_target(f, it, n.start_point[0] + 1)
            elif t == "macro_invocation":
                m = n.child_by_field_name("macro")
                it.calls.append({"form": "macro", "name": re.sub(r".*::", "", txt(m)), "path": txt(m), "recv": "", "line": n.start_point[0] + 1})
                for c in n.children:
                    if c.type == "token_tree": self.scan_tokens(c, it)
                continue
            stack.extend(reversed(n.children))

    def record_call_target(self, f, it, line):
        if f is None: return
        if f.type == "generic_function": f = f.child_by_field_name("function")
        if f.type == "identifier":
            it.calls.append({"form": "bare", "name": txt(f), "path": txt(f), "recv": "", "line": line})
        elif f.type == "scoped_identifier":
            nm = txt(f.child_by_field_name("name")); path = re.sub(r"<[^<>]*>", "", txt(f))
            it.calls.append({"form": "path", "name": nm, "path": path, "recv": "", "line": line})
        elif f.type == "field_expression":
            nm = txt(f.child_by_field_name("field")); recv = txt(f.child_by_field_name("value"))
            it.calls.append({"form": "method", "name": nm, "path": nm, "recv": recv if len(recv) < 40 else recv[:40], "line": line})
        # closures / parenthesised calls: not nameable

    def scan_tokens(self, tt, it):
        ch = tt.children
        for i, c in enumerate(ch):
            if c.type == "token_tree":
                self.scan_tokens(c, it)
            if c.type == "identifier" and txt(c) not in KEYWORDS and i + 1 < len(ch):
                nx = ch[i + 1]
                if nx.type == "token_tree" and txt(nx).startswith("("):
                    prev = ch[i - 1] if i else None
                    if prev is not None and prev.type == ".":
                        it.calls.append({"form": "method", "name": txt(c), "path": txt(c), "recv": "?", "line": c.start_point[0] + 1})
                    elif prev is not None and prev.type == "::":
                        segs = [txt(c)]; j = i - 1
                        while j >= 1 and ch[j].type == "::" and ch[j - 1].type in ("identifier", "self", "super", "crate"):
                            segs.insert(0, txt(ch[j - 1])); j -= 2
                        it.calls.append({"form": "path", "name": txt(c), "path": "::".join(segs), "recv": "", "line": c.start_point[0] + 1})
                    else:
                        it.calls.append({"form": "bare", "name": txt(c), "path": txt(c), "recv": "", "line": c.start_point[0] + 1})
                elif nx.type == "!":
                    it.calls.append({"form": "macro", "name": txt(c), "path": txt(c), "recv": "", "line": c.start_point[0] + 1})

# ---------------- parse everything
for p in files:
    src = p.read_bytes()
    tree = parser.parse(src)
    Walker(p, src).walk(tree.root_node, [], None, None, False)

# assign ids
seen = defaultdict(int)
for it in items:
    if not it.id:
        it.id = f"{it.file}::{it.kind}:{it.qual}@{it.line}"

# ---------------- indexes
fns = [i for i in items if i.kind in ("function", "method")]
free_by_name = defaultdict(list); meth_by_name = defaultdict(list); meth_by_type = defaultdict(list)
for f in fns:
    if f.kind == "function": free_by_name[f.name].append(f)
    else:
        meth_by_name[f.name].append(f); meth_by_type[f.impl_type, f.name].append(f)
mod_files = {}  # tail module name -> files
file_mod = {}
for p in files:
    mp = mod_path(p); rel = str(p.relative_to(ROOT)); file_mod[rel] = mp
    if mp: mod_files.setdefault(mp[-1], []).append(rel)
manifest_deps = {}
mf = ROOT / "docs" / "src-manifest.json"
if mf.exists():
    j = json.loads(mf.read_text())
    manifest_deps = {k: [d["file"] for d in v["depends_on"]] for k, v in j.items()}
cargo = (ROOT / "Cargo.toml").read_text()
EXT = {"std", "core", "alloc"} | {m.group(1).replace("-", "_") for m in re.finditer(r"^([A-Za-z0-9_-]+)\s*=", cargo.split("[dependencies]")[1].split("\n[")[0], re.M)}
EXT |= {"quick_xml_041", "zip", "serde", "serde_json", "image", "clap", "similar", "sha1", "sha2", "base64", "ttf_parser", "rustybuzz"}

PRELUDE_CTORS = {"Some", "Ok", "Err", "Box", "Vec", "String", "None", "Default", "From", "Into", "drop", "Self"}

CRATE_TYPES = {i.name for i in items if i.kind in ("struct", "enum", "union", "trait", "type alias")}

def recv_type(recv, it):
    r = recv.strip()
    r = re.sub(r"^(&\s*(mut\s+)?|\*)+", "", r).strip()
    if not r or r == "?": return ""
    parts = r.split(".")
    if not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", parts[0]): return ""
    cur = it.impl_type if parts[0] == "self" else (it.env or {}).get(parts[0], "")
    if parts[0] == "self" and it.trait and it.trait != "trait-decl": cur = it.impl_type
    for f in parts[1:]:
        if not cur: return ""
        cur = STRUCT_FIELDS.get(cur, {}).get(f, "")
    if cur in STD_TYPES or cur == "[]": return cur or ""
    return cur

def resolve(call, it):
    """Return (confidence, [target items]) for a call made from `it`."""
    name, form, path = call["name"], call["form"], call["path"]
    if form == "macro": return "macro", []
    if form == "bare":
        loc = [f for f in free_by_name.get(name, []) if f.file == it.file]
        if loc: return "exact", loc
        deps = set(manifest_deps.get(it.file, []))
        dep = [f for f in free_by_name.get(name, []) if f.file in deps]
        if len(dep) == 1: return "exact", dep
        if dep: return "ambiguous", dep
        glob = free_by_name.get(name, [])
        if len(glob) == 1: return "by-name", glob
        if glob: return "ambiguous", glob
        return "external", []
    if form == "path":
        segs = path.split("::")
        q = segs[:-1]
        if not q: return "external", []
        head = q[-1]
        if q[0] in EXT and q[0] not in ("crate",):
            if q[0] != "self": return "external", []
        if head == "Self" or (head[:1].isupper()):
            ty = it.impl_type if head == "Self" else head
            c = meth_by_type.get((ty, name), [])
            if head == "Self" and not c:
                c = [m for m in meth_by_type.get((it.impl_type, name), [])]
            if len(c) == 1: return "exact", c
            if c: return "ambiguous", c
            # enum variant / assoc fn of an external type
            return "external", []
        # module-qualified
        if head in ("crate", "self", "super") and len(q) == 1:
            pool = [f for f in free_by_name.get(name, [])]
            if head == "self": pool = [f for f in pool if f.file == it.file]
            if len(pool) == 1: return "exact", pool
            return ("ambiguous", pool) if pool else ("external", [])
        cand_files = set(mod_files.get(head, []))
        if head == "self": cand_files = {it.file}
        pool = [f for f in free_by_name.get(name, []) if f.file in cand_files]
        if len(pool) == 1: return "exact", pool
        if pool: return "ambiguous", pool
        # `module::function` where module is an inline mod: match modpath tail
        pool = [f for f in free_by_name.get(name, []) if f.modpath and f.modpath[-1] == head]
        if len(pool) == 1: return "exact", pool
        if pool: return "ambiguous", pool
        return "external", []
    if form == "method":
        rt = recv_type(call["recv"], it)
        if rt:
            if rt in CRATE_TYPES:
                c = meth_by_type.get((rt, name), [])
                if c: return ("exact" if len(c) == 1 else "ambiguous"), c
                return "external", []   # derived / inherited / std-trait method on a crate type
            return "external", []       # receiver is a std or otherwise foreign type
        if name in STD_METHODS: return "external", []
        c = meth_by_name.get(name, [])
        if not c: return "external", []
        if len(c) == 1: return "by-name", c
        return "ambiguous", c
    return "external", []

callers = defaultdict(list)
for it in fns:
    agg = {}
    for call in it.calls:
        conf, tg = resolve(call, it)
        key = (conf, call["form"], call["path"] if call["form"] != "method" else "." + call["name"], tuple(t.id for t in tg))
        e = agg.setdefault(key, {"conf": conf, "form": call["form"], "path": key[2], "targets": [t.id for t in tg], "lines": []})
        e["lines"].append(call["line"])
    it.calls = list(agg.values())
    for e in it.calls:
        if e["conf"] in ("exact", "by-name") or (e["conf"] == "ambiguous" and len(e["targets"]) <= 3):
            for tid in e["targets"]:
                callers[tid].append((it.id, e["conf"]))

byid = {i.id: i for i in items}

# ---------------- output
OUT.mkdir(parents=True, exist_ok=True)
for old in list(OUT.rglob("*.md")): old.unlink()
(OUT / "functions").mkdir(exist_ok=True)

def esc(s): return s.replace("|", "\\|")
def lines_str(ls):
    ls = sorted(set(ls)); return ",".join(map(str, ls[:6])) + ("..." if len(ls) > 6 else "")

def does(i):
    if i.docsrc == "doc": return i.doc
    if i.docsrc == "comment": return f"{i.doc} _(from a code comment)_"
    return "_no doc comment_"

def short(tid, rel_from=None):
    t = byid[tid]
    f = t.file.replace("src/", "")
    return f"`{t.qual}` ({f}:{t.line})"

stats = defaultdict(int)
perfile = defaultdict(list)
for i in items: perfile[i.file].append(i)

def fn_entry(i, L):
    flags = []
    if i.vis: flags.append(i.vis)
    if i.kind == "method" and i.trait == "trait-decl": flags.append("trait decl")
    elif i.kind == "method" and i.trait: flags.append(f"impl {i.trait}")
    if not i.has_body: flags.append("no body")
    L.append(f"#### `{i.qual}`  (L{i.line}-{i.end}{', ' + ', '.join(flags) if flags else ''})")
    L.append(f"`{esc(i.sig)}`\n")
    L.append(f"- **Does:** {does(i)}")
    ex = [e for e in i.calls if e["conf"] == "external"]
    mc = [e for e in i.calls if e["conf"] == "macro"]
    inn = [e for e in i.calls if e["conf"] in ("exact", "by-name", "ambiguous")]
    if inn:
        parts = []
        for e in inn:
            tg = e["targets"]
            if e["conf"] == "ambiguous" and len(tg) > 3:
                parts.append(f"`{e['path']}` (ambiguous, {len(tg)} candidates) L{lines_str(e['lines'])}")
            elif e["conf"] == "ambiguous":
                parts.append(f"`{e['path']}` (ambiguous: " + "; ".join(short(t) for t in tg) + f") L{lines_str(e['lines'])}")
            else:
                tag = "" if e["conf"] == "exact" else " (by name)"
                parts.append(f"{short(tg[0])}{tag} L{lines_str(e['lines'])}")
        L.append("- **Calls (crate):** " + "; ".join(parts))
    if ex: L.append("- **Calls (std/external/unresolved):** " + ", ".join(sorted({f"`{e['path']}`" for e in ex})))
    if mc: L.append("- **Macros:** " + ", ".join(sorted({f"`{e['path']}!`" for e in mc})))
    if not i.calls: L.append("- **Calls:** none")
    cb = callers.get(i.id, [])
    if cb:
        seen_c = []
        for cid, conf in cb:
            if cid not in [s[0] for s in seen_c]: seen_c.append((cid, conf))
        shown = seen_c[:12]
        s = "; ".join(short(c) + ("" if cf == "exact" else " (?)") for c, cf in shown)
        if len(seen_c) > 12: s += f"; +{len(seen_c) - 12} more"
        L.append(f"- **Called by:** {s}")
    else:
        L.append("- **Called by:** none found in src")
    L.append("")

for rel, its in sorted(perfile.items()):
    fl = [i for i in its if i.kind in ("function", "method")]
    if not fl: continue
    prod = [i for i in fl if not i.is_test]; tst = [i for i in fl if i.is_test]
    L = [f"# Functions and methods in `{rel}`\n",
         f"{len(prod)} non-test, {len(tst)} test. Part of the [symbol manifest](../README.md).\n"]
    for i in sorted(prod, key=lambda x: x.line): fn_entry(i, L)
    if tst:
        L.append(f"## Test functions ({len(tst)})\n")
        L.extend(f"- `{i.qual}` (L{i.line}): {does(i)}" for i in sorted(tst, key=lambda x: x.line))
        L.append("")
    out = OUT / "functions" / (rel.replace("src/", "").replace("/", "__").replace(".rs", "") + ".md")
    out.write_text("\n".join(L))

def kind_manifest(title, fname, kinds, cols=("Name", "Where", "Vis", "Does")):
    sel = [i for i in items if i.kind in kinds and not i.is_test]
    L = [f"# {title}\n", f"{len(sel)} items (non-test). Part of the [symbol manifest](README.md).\n"]
    L.append("| Name | Defined at | Vis | Does | Detail |")
    L.append("|---|---|---|---|---|")
    for i in sorted(sel, key=lambda x: (x.file, x.line)):
        detail = ""
        for a in i.attrs:
            if a.startswith("variants="): detail = "variants: " + a[9:]
            elif a.startswith("fields="): detail = "fields: " + a[7:]
        if i.kind == "impl": detail = esc(i.sig)[:160]
        if i.kind in ("const", "static", "type alias"): detail = esc(i.sig)[:160]
        L.append(f"| `{esc(i.qual)}` | {i.file}:{i.line} | {i.vis or 'private'} | {esc(does(i))} | {esc(detail)[:300]} |")
    (OUT / fname).write_text("\n".join(L) + "\n")
    return len(sel)

counts = {}
counts["struct"] = kind_manifest("Structs", "structs.md", ("struct",))
counts["enum"] = kind_manifest("Enums", "enums.md", ("enum", "union"))
counts["trait"] = kind_manifest("Traits", "traits.md", ("trait",))
counts["impl"] = kind_manifest("Impl blocks", "impls.md", ("impl",))
counts["const_static"] = kind_manifest("Constants and statics", "consts.md", ("const", "static"))
counts["type alias"] = kind_manifest("Type aliases", "type_aliases.md", ("type alias",))
counts["macro"] = kind_manifest("Macros", "macros.md", ("macro",))
counts["module"] = kind_manifest("Modules", "modules.md", ("module",))

allfn = [i for i in fns]
prodfn = [i for i in allfn if not i.is_test]
documented = [i for i in prodfn if i.docsrc == "doc"]
fromc = [i for i in prodfn if i.docsrc == "comment"]
nodoc = [i for i in prodfn if i.docsrc == "none"]
edges = defaultdict(int)
for i in prodfn:
    for e in i.calls: edges[e["conf"]] += len(e["lines"])

R = ["# Symbol manifest for `src/`\n",
     "Every function, method, struct, enum, trait, impl block, constant, static, type alias, macro and module under `src/`, extracted statically with tree-sitter (Rust grammar). No LLM agents and no compiler were involved. Regenerate with `scripts/gen_symbol_manifest.py` (see the top of that file).\n",
     "## What is in here\n",
     "| Manifest | Items |", "|---|---:|",
     f"| [functions/](functions/) (one file per source file; functions and methods) | {len(prodfn)} non-test + {len(allfn) - len(prodfn)} test |",
     f"| [structs.md](structs.md) | {counts['struct']} |",
     f"| [enums.md](enums.md) (enums and unions) | {counts['enum']} |",
     f"| [traits.md](traits.md) | {counts['trait']} |",
     f"| [impls.md](impls.md) | {counts['impl']} |",
     f"| [consts.md](consts.md) (consts and statics) | {counts['const_static']} |",
     f"| [type_aliases.md](type_aliases.md) | {counts['type alias']} |",
     f"| [macros.md](macros.md) (`macro_rules!`) | {counts['macro']} |",
     f"| [modules.md](modules.md) | {counts['module']} |",
     "| [symbols.json](symbols.json) | everything above, including test items and every call edge |", "",
     "## How to read a function entry\n",
     "- **Does** is the item's own `///` doc comment (first sentence). If there is none, a plain `//` comment directly above it is used and marked as such. Otherwise it says _no doc comment_: nothing is guessed. Coverage is below.",
     "- **Calls (crate)** lists the crate functions this one calls, as `qualified name (file:line)` and the call-site lines. **Calls (std/external/unresolved)** lists paths that did not resolve to a crate function (standard library, dependencies, enum variants, tuple-struct constructors). **Macros** lists macro invocations.",
     "- **Called by** is the inverse of the crate call edges.",
     "",
     "### Call-resolution confidence\n",
     "tree-sitter gives syntax, not types, so a call is matched to a definition by name:\n",
     "- no tag: unique match found by a rule that is safe without types (same-file free function, `Self::f` / `Type::f` with a matching impl, `module::f` with a matching module file, `self.f()` on the same impl type).",
     "- _(by name)_: a method or free function with that name exists exactly once in the crate and nothing better matched. A `.len()` on a std type would be mis-attributed if the crate defined one `len` method, so read these as likely, not certain.",
     "- _ambiguous_: several crate definitions share the name; candidates are listed (or counted when more than four). In **Called by**, a `(?)` marks edges of this kind.",
     "- calls through trait objects, generics, closures stored in variables, function pointers and `.map(Self::f)`-style references are not seen as calls.",
     "- calls inside macro arguments (`assert_eq!(f(x), y)`, `format!`) are recovered from the token tree by pattern (identifier followed by parentheses), which is looser than the call-expression path.",
     "",
     "## Documentation coverage (non-test functions and methods)\n",
     f"- with a `///` doc comment: **{len(documented)}** ({100 * len(documented) / max(1, len(prodfn)):.0f}%)",
     f"- with only a plain `//` comment above: {len(fromc)}",
     f"- no description in the source: **{len(nodoc)}**. These say _no doc comment_; their name, signature and call list are the available information.",
     "",
     "## Call edges (non-test functions, counting call sites)\n",
     "| Confidence | Call sites |", "|---|---:|"]
R.extend(f"| {k} | {edges.get(k, 0):,} |" for k in ("exact", "by-name", "ambiguous", "external", "macro"))
R.append("")
(OUT / "README.md").write_text("\n".join(R))

index = {it.id: n for n, it in enumerate(items)}
data = []
for n, i in enumerate(items):
    d = {"i": n, "kind": i.kind, "name": i.name, "qualified": i.qual, "file": i.file, "line": i.line, "end_line": i.end,
         "visibility": i.vis or "private", "signature": i.sig, "doc": i.doc, "doc_source": i.docsrc, "test": bool(i.is_test)}
    if i.impl_type: d["impl_type"] = i.impl_type
    if i.trait: d["trait"] = i.trait
    if i.kind in ("function", "method") and not i.is_test:
        d["calls"] = [{"confidence": e["conf"], "form": e["form"], "path": e["path"], "lines": sorted(set(e["lines"])),
                       **({"targets": [index[t] for t in e["targets"]]} if e["targets"] else {})} for e in i.calls]
        cb = sorted({(index[c], cf) for c, cf in callers.get(i.id, [])})
        if cb: d["called_by"] = [{"i": c, "confidence": cf} for c, cf in cb]
    data.append(d)
(OUT / "symbols.json").write_text(json.dumps({"note": "calls[].targets and called_by[].i are indexes into this list's 'i' field", "symbols": data}, separators=(",", ":")) + "\n")
print({"items": len(items), "fns": len(allfn), "prod": len(prodfn), "documented": len(documented), "edges": dict(edges), **counts})

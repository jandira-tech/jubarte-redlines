# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only
"""Where the Rust source accepts an id: for scripts/cli_locations.py.

Reads `src/` as text, with no build:

- the clap fields of `src/cli.rs` whose doc or value name speaks of an id,
  with their `value_parser` (none: the value reaches a parser after clap);
- the id parsers: every function of `src/ids.rs` that reads a token
  (`&str` in, `Result`/`Option`/`bool` out, or a serde `_de`), and every
  function elsewhere that refuses a token with an id message
  (`expected pN…`, `expected a change id…`), with its doc and messages;
- a call graph of the non-test source, walked up from each parser to the
  CLI layer (`src/cli.rs`, `src/bin/`), so each flag is linked to the
  parsers that read its value.

Functions are named, never numbered by line, so an unrelated edit does
not move the output.
"""

from __future__ import annotations

import re
from collections import defaultdict, deque
from dataclasses import dataclass, field
from pathlib import Path

FN_DEF = re.compile(
    r"^(?P<indent>\s*)(?:pub(?:\([^)]*\))?\s+)?(?:const\s+)?(?:async\s+)?(?:unsafe\s+)?fn\s+(?P<name>\w+)"
)
IMPL = re.compile(r"^impl(?:<[^{]*?>)?\s+(?:(?P<trait>[\w:<>, ']+?)\s+for\s+)?(?:\w+::)*(?P<ty>[A-Za-z_]\w*)")
# Method names std and serde also use: a bare `.name(` with one of these is
# never taken for a call to our method of that name.
STD_METHODS = {
    "all",
    "any",
    "as_ref",
    "as_str",
    "clone",
    "cmp",
    "collect",
    "contains",
    "count",
    "default",
    "deref",
    "drop",
    "eq",
    "extend",
    "filter",
    "find",
    "first",
    "flat_map",
    "fmt",
    "from",
    "get",
    "hash",
    "insert",
    "into",
    "is_empty",
    "iter",
    "join",
    "last",
    "len",
    "map",
    "new",
    "next",
    "open",
    "parse",
    "position",
    "push",
    "read",
    "remove",
    "run",
    "scan",
    "sort",
    "split",
    "take",
    "to_string",
    "write",
}
# A refusal that names the id grammar: the mark of a parser outside ids.rs.
ID_MESSAGE = re.compile(r'"[^"]*expected (?:pN|a (?:change|comment|paragraph|location) id|an id)[^"]*"')
ID_WORD = re.compile(r"\b(?:p|r|c|t)\d+\b|\bWHERE\b|\bID\b|\bIDS\b")
CLI_LAYER = ("src/cli.rs", "src/bin/")


@dataclass
class Fn:
    key: str  # `name` or `Type::name`
    name: str
    file: str
    module: str  # `ids`, `edit::flags`, `bin::jubarte`
    impl: str | None
    signature: str
    doc: str
    body: str
    calls: set[str] = field(default_factory=set)


def module_of(rel: str) -> str:
    parts = Path(rel).with_suffix("").parts[1:]  # drop `src`
    if parts and parts[-1] == "mod":
        parts = parts[:-1]
    return "::".join(parts)


def non_test_lines(text: str) -> list[str]:
    """The file up to its `#[cfg(test)] mod tests`."""
    lines = text.splitlines()
    for i, line in enumerate(lines):
        if line.startswith("#[cfg(test)]") and any(
            re.match(r"^(pub\s+)?mod\s+\w+", following) for following in lines[i + 1 : i + 3]
        ):
            return lines[:i]
    return lines


def doc_above(lines: list[str], at: int) -> str:
    doc: list[str] = []
    i = at - 1
    while i >= 0 and (lines[i].strip().startswith("#[") or lines[i].strip().startswith("///")):
        if lines[i].strip().startswith("///"):
            doc.insert(0, lines[i].strip()[3:].strip())
        i -= 1
    return " ".join(doc)


def functions(root: Path) -> dict[str, Fn]:
    """Every top-level function and impl method of the non-test source."""
    found: dict[str, Fn] = {}
    for path in sorted((root / "src").rglob("*.rs")):
        rel = path.relative_to(root).as_posix()
        lines = non_test_lines(path.read_text(encoding="utf-8"))
        impl: str | None = None
        starts: list[tuple[int, str, str | None]] = []
        for i, line in enumerate(lines):
            if match := IMPL.match(line):
                impl = match.group("ty")
            elif line.startswith("}"):
                impl = None
            match = FN_DEF.match(line)
            if not match:
                continue
            if match.group("indent") == "":
                starts.append((i, match.group("name"), None))
            elif impl and match.group("indent") == "    ":
                # A trait's method (`From::from` carrying a flag into a filter)
                # is kept, but only qualified calls reach it: see STD_METHODS.
                starts.append((i, match.group("name"), impl))
        for n, (i, name, owner) in enumerate(starts):
            # rustfmt closes a body with `}` at the definition's indent.
            indent = "    " if owner else ""
            end = next(
                (j + 1 for j in range(i, len(lines)) if lines[j].rstrip() == f"{indent}}}"),
                starts[n + 1][0] if n + 1 < len(starts) else len(lines),
            )
            body = "\n".join(lines[i:end])
            signature = " ".join(body.split("{", 1)[0].split())
            key = f"{owner}::{name}" if owner else name
            module = module_of(rel)
            # Two methods of one name on one type in different files stay apart.
            unique = key if key not in found else f"{module}::{key}"
            found[unique] = Fn(unique, name, rel, module, owner, signature, doc_above(lines, i), body)
    return found


def link_calls(fns: dict[str, Fn]) -> None:
    """Fill `calls`: a call is `name(`, `Type::name(`, `Self::name(`, a path
    ending in `::name(`, `.name(` for a method whose name is unique, and the
    `value_parser = name` / `deserialize_with = "…::name"` attributes."""
    by_name: dict[str, list[Fn]] = defaultdict(list)
    for fn in fns.values():
        by_name[fn.name].append(fn)
    for fn in fns.values():
        body = fn.body.split("{", 1)[1] if "{" in fn.body else ""
        for name in (
            set(re.findall(r"\b(\w+)\s*(?:\(|::<)", body))
            | set(re.findall(r"::(\w+)\b(?!::)", body))
            | set(re.findall(r'(?:value_parser\s*=\s*|deserialize_with\s*=\s*"[\w:]*?)(\w+)', body))
        ):
            for target in by_name.get(name, []):
                if target is fn:
                    continue
                if target.impl:
                    qualified = re.search(rf"\b(?:{target.impl}|Self)::{name}\b", body) and (
                        target.impl == fn.impl or f"{target.impl}::{name}" in body
                    )
                    method = (
                        len(by_name[name]) == 1
                        and name not in STD_METHODS
                        and re.search(rf"\.{name}\s*\(", body)
                    )
                    if qualified or method:
                        fn.calls.add(target.key)
                elif re.search(
                    rf"(?<![.\w]){name}\s*\(|::{name}\b(?!::)|value_parser\s*=\s*{name}\b|{name}\"", body
                ):
                    same_file = [t for t in by_name[name] if not t.impl and t.file == fn.file]
                    pathed = re.search(rf"\b{target.module.split('::')[-1]}::{name}\b", body)
                    if target in same_file or pathed or len([t for t in by_name[name] if not t.impl]) == 1:
                        fn.calls.add(target.key)


def attribute_uses(root: Path, name: str) -> list[str]:
    """Types whose fields name `name` in an attribute (`deserialize_with`,
    `value_parser`): a parser serde or clap calls."""
    found = set()
    for path in sorted((root / "src").rglob("*.rs")):
        owner = "?"
        for line in non_test_lines(path.read_text(encoding="utf-8")):
            if m := re.match(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:struct|enum)\s+(\w+)", line):
                owner = m.group(1)
            if re.search(
                rf'(?:deserialize_with|with)\s*=\s*"[\w:]*\b{name}"|value_parser\s*=\s*{name}\b', line
            ):
                found.add(f"`{module_of(path.relative_to(root).as_posix())}::{owner}` (attribute)")
    return sorted(found)


def parsers(fns: dict[str, Fn]) -> list[Fn]:
    out = []
    for fn in fns.values():
        if not fn.signature.startswith("pub"):
            continue
        if fn.file == "src/ids.rs":
            first = re.search(r"\(\s*(?:&self,\s*)?\w+:\s*&str", fn.signature)
            returns = re.search(r"->\s*(?:Result|Option|bool)", fn.signature)
            if (first and returns) or fn.name.endswith("_de"):
                out.append(fn)
        elif ID_MESSAGE.search(fn.body):
            out.append(fn)
    return sorted(out, key=lambda f: (f.file, f.key))


def messages(fn: Fn, root: Path) -> list[str]:
    """The refusals a parser can give: its id-message literals and the ids.rs
    constants it names, spelled out."""
    consts = dict(
        re.findall(
            r'pub const (\w+): &str = "([^"]*)";', (root / "src" / "ids.rs").read_text(encoding="utf-8")
        )
    )
    found = [m.strip('"') for m in ID_MESSAGE.findall(fn.body)]
    found += [consts[c] for c in sorted(set(re.findall(r"\b([A-Z_]{6,})\b", fn.body))) if c in consts]
    return sorted(set(found))


@dataclass
class Flag:
    owner: str  # the struct or the `Command` variant
    field: str
    short: str | None
    long: str
    value_name: str
    value_parser: str
    doc: str


def id_flags(root: Path) -> list[Flag]:
    """The clap fields of src/cli.rs whose doc or value name speaks of an id."""
    lines = non_test_lines((root / "src" / "cli.rs").read_text(encoding="utf-8"))
    flags, owner, i = [], "?", 0
    while i < len(lines):
        line = lines[i]
        if m := re.match(r"^pub struct (\w+)", line):
            owner = m.group(1)
        elif m := re.match(r"^    (\w+) \{", line):
            owner = f"Command::{m.group(1)}"
        if line.strip().startswith("#[arg("):
            start, depth, attr = i, 0, ""
            while True:
                attr += lines[i].strip() + " "
                depth += lines[i].count("(") - lines[i].count(")")
                if depth <= 0:
                    break
                i += 1
            j = i + 1
            while j < len(lines) and lines[j].strip().startswith("#["):
                j += 1
            m = re.match(r"^\s*(?:pub\s+)?(\w+):", lines[j]) if j < len(lines) else None
            if m:
                name = m.group(1)
                short = re.search(r"short\s*=\s*'(\w)'", attr) or (
                    re.search(r"\bshort\b(?!\s*=)", attr) and name[0]
                )
                long = re.search(r'long\s*=\s*"([\w-]+)"', attr)
                value_name = re.search(r'value_name\s*=\s*"([^"]+)"', attr)
                value_parser = re.search(r"value_parser\s*=\s*([\w:!()]+)", attr)
                doc = doc_above(lines, start)
                vn = value_name.group(1) if value_name else ""
                # The value itself is an id: an id value name, or a value (not
                # free text) whose doc spells ids.
                if vn in {"WHERE", "ID", "IDS"} or (vn and vn != "TEXT" and ID_WORD.search(doc)):
                    flags.append(
                        Flag(
                            owner,
                            name,
                            short.group(1) if hasattr(short, "group") else short or None,
                            long.group(1) if long else name.replace("_", "-"),
                            vn,
                            value_parser.group(1) if value_parser else "",
                            doc,
                        )
                    )
            i = j
        i += 1
    return flags


def reaches(fns: dict[str, Fn], start: Fn, targets: set[str], limit: int = 7) -> dict[str, list[Fn]]:
    """The shortest call chain from `start` down to each target it reaches."""
    seen = {start.key: [start]}
    out: dict[str, list[Fn]] = {}
    queue = deque([start.key])
    while queue:
        key = queue.popleft()
        path = seen[key]
        if key in targets and key != start.key:
            out[key] = path
            continue
        if len(path) >= limit:
            continue
        for callee in sorted(fns[key].calls):
            if callee not in seen:
                seen[callee] = [*path, fns[callee]]
                queue.append(callee)
    return out


def where(fn: Fn) -> str:
    return f"`{fn.module}::{fn.key}`" if fn.module else f"`{fn.key}`"


def section(root: Path, cell) -> list[str]:
    fns = functions(root)
    link_calls(fns)
    found = parsers(fns)
    flags = id_flags(root)
    lines = [
        "## Where the source accepts an id",
        "",
        "Read from `src/` (`scripts/cli_id_sources.py`), not from `--help`. A flag with no "
        "`value_parser` reaches its parser after clap; the chains below show which.",
        "",
        "### Id flags (src/cli.rs)",
        "",
        "| Owner | Field | Flag | Value | value_parser | Read by (CLI layer) | Doc |",
        "|-|-|-|-|-|-|-|",
    ]
    cli_fns = [f for f in fns.values() if f.file.startswith(CLI_LAYER)]
    targets = {fn.key for fn in found}
    flows: list[str] = []
    reached: set[str] = set()
    for flag in flags:
        spelled = (f"-{flag.short}, " if flag.short else "") + f"--{flag.long}"
        variant = flag.owner.split("::")[-1]
        pattern = re.compile(rf'\.{flag.field}\b|"{flag.long}"|\b{variant}\s*\{{[^}}]*\b{flag.field}\b')
        readers = sorted(
            (f for f in cli_fns if pattern.search(f.body.split("{", 1)[-1])), key=lambda f: f.key
        )
        lines.append(
            f"| {flag.owner} | `{flag.field}` | `{spelled}` | {flag.value_name} | "
            f"{flag.value_parser or '(none)'} | {cell(', '.join(where(f) for f in readers))} | {cell(flag.doc)} |"
        )
        for reader in readers:
            for parser, path in sorted(reaches(fns, reader, targets).items()):
                reached.add(parser)
                row = f"| `{spelled}` | {where(fns[parser])} | {' → '.join(where(s) for s in path)} |"
                if row not in flows:
                    flows.append(row)
    lines += [
        "",
        "### Id parsers",
        "",
        "Every public function that reads an id token: in `src/ids.rs` a `&str` in and a `Result`, "
        "`Option` or `bool` out (or a serde `_de`); elsewhere, any function that refuses a token "
        "with an id message. Doc and refusals come from the source.",
        "",
        "| Parser | Signature | Doc | Refusals | Reached from an id flag |",
        "|-|-|-|-|-|",
    ]
    for fn in found:
        lines.append(
            f"| {where(fn)} | `{cell(fn.signature)}` | {cell(fn.doc)} | "
            f"{cell(' / '.join(messages(fn, root)))} | {'yes' if fn.key in reached else '**no**'} |"
        )
    callers: dict[str, list[Fn]] = defaultdict(list)
    for fn in fns.values():
        for callee in fn.calls:
            callers[callee].append(fn)
    lines += [
        "",
        "### Where each parser is called",
        "",
        "Every non-test function that calls an id parser directly: the places the source "
        "accepts an id value.",
        "",
        "| Parser | Called from |",
        "|-|-|",
    ]
    for fn in found:
        named = sorted({where(c) for c in callers[fn.key]}) + attribute_uses(root, fn.name)
        lines.append(f"| {where(fn)} | {', '.join(named) or '(nothing)'} |")
    lines += [
        "",
        "### From each id flag to its parsers",
        "",
        "The shortest call chain from each function that reads the flag down to each id parser "
        "it reaches. Static: a value carried in a struct (`FlagOp.at`, `ChangeFilter.ids`) and "
        "read later, or passed through a closure or a trait object, is not followed; the table "
        "above has those callers.",
        "",
        "| Flag | Parser | Chain |",
        "|-|-|-|",
        *flows,
    ]
    return lines

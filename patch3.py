import sys

with open("scripts/cli_id_sources.py", "r") as f:
    content = f.read()

old_func = """def messages(fn: Fn, root: Path) -> list[str]:
    \"\"\"The refusals a parser can give: its id-message literals and the ids.rs
    constants it names, spelled out.\"\"\"
    consts = dict(
        re.findall(
            r'pub const (\w+): &str = "([^"]*)";', (root / "src" / "ids.rs").read_text(encoding="utf-8")
        )
    )
    found = [m.strip('"') for m in ID_MESSAGE.findall(fn.body)]
    found += [consts[c] for c in sorted(set(re.findall(r"\\b([A-Z_]{6,})\\b", fn.body))) if c in consts]
    return sorted(set(found))"""

new_func = """_CONSTS_CACHE: dict[Path, dict[str, str]] = {}

def messages(fn: Fn, root: Path) -> list[str]:
    \"\"\"The refusals a parser can give: its id-message literals and the ids.rs
    constants it names, spelled out.\"\"\"
    if root not in _CONSTS_CACHE:
        _CONSTS_CACHE[root] = dict(
            re.findall(
                r'pub const (\\w+): &str = "([^"]*)";', (root / "src" / "ids.rs").read_text(encoding="utf-8")
            )
        )
    consts = _CONSTS_CACHE[root]
    found = [m.strip('"') for m in ID_MESSAGE.findall(fn.body)]
    found.extend(consts[c] for c in set(re.findall(r"\\b([A-Z_]{6,})\\b", fn.body)).intersection(consts))
    return sorted(set(found))"""

content = content.replace(old_func, new_func)

with open("scripts/cli_id_sources.py", "w") as f:
    f.write(content)

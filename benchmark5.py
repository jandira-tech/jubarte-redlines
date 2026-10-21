import time
import re
from pathlib import Path
import sys

sys.path.append('scripts')
import cli_id_sources

def messages_original(fn: cli_id_sources.Fn, root: Path) -> list[str]:
    consts = dict(
        re.findall(
            r'pub const (\w+): &str = "([^"]*)";', (root / "src" / "ids.rs").read_text(encoding="utf-8")
        )
    )
    found = [m.strip('"') for m in cli_id_sources.ID_MESSAGE.findall(fn.body)]
    found += [consts[c] for c in sorted(set(re.findall(r"\b([A-Z_]{6,})\b", fn.body))) if c in consts]
    return sorted(set(found))

def messages_no_cache_intersection(fn: cli_id_sources.Fn, root: Path) -> list[str]:
    consts = dict(
        re.findall(
            r'pub const (\w+): &str = "([^"]*)";', (root / "src" / "ids.rs").read_text(encoding="utf-8")
        )
    )
    found = [m.strip('"') for m in cli_id_sources.ID_MESSAGE.findall(fn.body)]
    found.extend(consts[c] for c in set(re.findall(r"\b([A-Z_]{6,})\b", fn.body)).intersection(consts))
    return sorted(set(found))

_cache = {}
def messages_cache_original(fn: cli_id_sources.Fn, root: Path) -> list[str]:
    if root not in _cache:
        _cache[root] = dict(re.findall(r'pub const (\w+): &str = "([^"]*)";', (root / "src" / "ids.rs").read_text(encoding="utf-8")))
    consts = _cache[root]
    found = [m.strip('"') for m in cli_id_sources.ID_MESSAGE.findall(fn.body)]
    found += [consts[c] for c in sorted(set(re.findall(r"\b([A-Z_]{6,})\b", fn.body))) if c in consts]
    return sorted(set(found))

def run_benchmark():
    root = Path('.')
    fns = cli_id_sources.functions(root)

    start_time = time.time()
    for _ in range(50):
        for fn in fns.values():
            messages_original(fn, root)
    end_time = time.time()
    print(f"Original execution time: {end_time - start_time:.4f} seconds")

    start_time = time.time()
    for _ in range(50):
        for fn in fns.values():
            messages_no_cache_intersection(fn, root)
    end_time = time.time()
    print(f"No cache + intersection execution time: {end_time - start_time:.4f} seconds")

    start_time = time.time()
    for _ in range(50):
        for fn in fns.values():
            messages_cache_original(fn, root)
    end_time = time.time()
    print(f"Cache + original execution time: {end_time - start_time:.4f} seconds")

run_benchmark()

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Stage the legacy Ring-2 mapping's exact sources without modifying the bench.

Prefer the checked-in originals, otherwise resolve their legacy names through
corpus/word/pools/word_based_pairs.csv. Pool base/next paths omit .docx and are
relative to corpus/word. notices/RENAMED.csv recovers legacy aliases with their
recorded SHA-256, restricted to the word_based pool. Copies keep the sweep
independent of bench symlinks.
Missing sources fail with their affected pairs; this never re-blesses a baseline.
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import os
import shutil
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MAPPING = ROOT / "tests/corpus/neurotic_docx_bench/corpus/word_based/centralized_mapping.csv"


def read_rows(path: Path, required: set[str]) -> list[dict[str, str]]:
    with path.open(newline="", encoding="utf-8") as stream:
        reader = csv.DictReader(stream)
        if not required.issubset(reader.fieldnames or ()):
            raise ValueError(f"{path}: missing columns {sorted(required)}")
        rows = list(reader)
    if not rows or any(not row.get(key) for row in rows for key in required):
        raise ValueError(f"{path}: empty table or required value")
    return rows


def source_candidates(mapping, pool, word: Path, local: Path, renames=()):
    """Pure name resolution: never guess a hash prefix or choose an ambiguous alias."""
    names = {row[key] for row in mapping for key in ("docx_source_base", "docx_source_next")}
    aliases: dict[str, set[Path]] = {}
    for row in pool:
        for side in ("base", "next"):
            name = row[f"{side}_name"] + ".docx"
            relative = Path(row[side] + ".docx")
            if relative.is_absolute() or ".." in relative.parts:
                raise ValueError(f"pool path escapes corpus/word: {relative}")
            aliases.setdefault(name, set()).add(word / relative)
    pool_paths = {path for paths in aliases.values() for path in paths}
    hashes = {}
    for row in renames:
        original = Path(row["original"])
        path = word / row["new"]
        if (row["set"] != "word_based" or original.parent.name != "docx_source"
                or original.name not in names or path not in pool_paths):
            continue
        if path in hashes and hashes[path] != row["sha256"]:
            raise ValueError(f"conflicting rename hashes for {path}")
        aliases.setdefault(original.name, set()).add(path)
        hashes[path] = row["sha256"]
    candidates = {}
    for name in sorted(names):
        if Path(name).name != name or not name.endswith(".docx"):
            raise ValueError(f"mapping source must be a .docx filename: {name}")
        paths = aliases.get(name, set())
        if len(paths) > 1:
            raise ValueError(f"ambiguous pool alias {name}: {sorted(map(str, paths))}")
        candidates[name] = [local / name, *sorted(paths)]
    return candidates, hashes


def stage(mapping_path: Path, bench: Path, out: Path) -> bool:
    mapping = read_rows(mapping_path, {"pair_stem", "docx_source_base", "docx_source_next"})
    word = bench / "corpus/word"
    pool_path = word / "pools/word_based_pairs.csv"
    pool = read_rows(pool_path, {"base", "next", "base_name", "next_name"})
    ledger = word / "notices/RENAMED.csv"
    renames = read_rows(ledger, {"original", "new", "sha256", "set"}) if ledger.is_file() else []
    candidates, hashes = source_candidates(mapping, pool, word, mapping_path.parent / "docx_source", renames)
    sources = {}
    missing = []
    for name, paths in candidates.items():
        available = [path for path in paths if path.is_file()]
        if not available:
            missing.append(name)
            continue
        for path in available:
            if path in hashes and hashlib.sha256(path.read_bytes()).hexdigest() != hashes[path]:
                raise ValueError(f"{name}: sha256 mismatch against rename ledger for {path}")
        source = available[0]
        if any(path.read_bytes() != source.read_bytes() for path in available[1:]):
            raise ValueError(f"{name}: local original and pool have different bytes")
        dest = out / name
        if dest.exists() and not dest.is_symlink() and dest.read_bytes() != source.read_bytes():
            raise ValueError(f"refusing to overwrite {dest}: different bytes")
        sources[name] = source
    out.mkdir(parents=True, exist_ok=True)
    for name, source in sources.items():
        dest = out / name
        # A stale/dangling staging link is replaced, never written through.
        if dest.is_symlink():
            dest.unlink()
        if not dest.exists():
            shutil.copyfile(source, dest)
    print(f"staged {len(sources)}/{len(candidates)} sources in {out} (bench: {bench})")
    if missing:
        for name in missing:
            pairs = sorted(row["pair_stem"] for row in mapping
                           if name in (row["docx_source_base"], row["docx_source_next"]))
            print(f"MISSING {name}: {', '.join(pairs)}", file=sys.stderr)
        print("Restore dangling bench links with `uv run bench fixtures download` from the bench root; "
              "rerun staging. If originals remain unavailable, report them to the maintainer; "
              "only the maintainer may authorize re-blessing. Baseline unchanged.", file=sys.stderr)
    return not missing


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source_dir", type=Path, help="destination for the sweep's legacy source names")
    parser.add_argument("--mapping", type=Path, default=MAPPING)
    args = parser.parse_args(argv)
    bench = Path(os.environ.get("NEUROTIC_DOCX_BENCH", str(ROOT.parent / "neurotic_docx_bench")))
    try:
        return 0 if stage(args.mapping, bench, args.source_dir) else 1
    except (OSError, ValueError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())

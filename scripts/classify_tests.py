#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Classify integration tests into the seven suites (plan A8).

    scripts/classify_tests.py            # print `<file> <suite>` + counts
    scripts/classify_tests.py tests/foo.rs
                                         # print the suite of one file

The rule is A8 of docs/refactoring_best_practices/implementation_plan.md:
a test file belongs to the suite of the highest layer it references, where a
reference is any `jubarte::<module>` path (code, `use` groups and intra-doc
links alike, scanned exactly the way `crate_paths` in
tests/crate_layering.rs scans `crate::` paths, comments counting as
whitespace), mapped through that guard's `MODULES` table (which the script
reads from the guard file, so the two never drift). A file also references
the cli layer when it names `CARGO_BIN_EXE_jubarte` or opens with
`#![cfg(feature = "cli")]` or `#![cfg(feature = "self-update")]`.

Layers, lowest first: dom opc redlines render doc markdown edit cli. The
suite of a file referencing nothing, or only dom and/or opc, is `it_core`;
every other layer maps to `it_<layer>`. The standalone tests of A8 (run as
their own binaries, never moved) are skipped.

Exit status is 0 even when counts differ from the plan; the plan's expected
counts are a baseline-time expectation, not an invariant. The invariant the
suite_membership guard enforces is that every file lands somewhere.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
GUARD = ROOT / "tests" / "crate_layering.rs"

# A8's standalone binaries, minus facade_surface (T4.0 creates it and adds
# it here and to tests/suite_membership.rs). Keep the three lists in sync.
STANDALONE = [
    "adoption",
    "bench_fixture_paths_resolve",
    "cli_version",
    "convert_docx_to_pdf",
    "coverage_gates",
    "crate_layering",
    "deny_build_scripts",
    "foreign_alignment",
    "legacy_doc",
    "legacy_doc_convert",
    "md_fixed_point",
    "m_validity_ring1",
    "perf_atom_clone_allocs",
    "perf_contract",
    "perf_serialize_prefix_allocs",
    "readme_coverage",
    "release_metadata",
    "repo_paths_fit_windows",
    "schema_consistency",
    "suite_membership",
    "tuning_audit",
]


def load_tables() -> tuple[list[str], dict[str, str]]:
    """`LAYERS` and `MODULES` from tests/crate_layering.rs, parsed or dead."""
    text = GUARD.read_text(encoding="utf-8")

    def const_block(name: str) -> str:
        start = text.index(f"const {name}")
        end = text.index("];", start)
        return text[start:end]

    layers: list[str] = []
    layer_block = const_block("LAYERS")
    for quoted in _quoted_strings(layer_block):
        layers.append(quoted)
    if len(layers) < 2:
        raise SystemExit(f"cannot parse LAYERS from {GUARD}")

    modules: dict[str, str] = {}
    module_block = const_block("MODULES")
    pairs = re.findall(r'\("([^"]+)",\s*"([^"]+)"\)', module_block)
    if not pairs:
        raise SystemExit(f"cannot parse MODULES from {GUARD}")
    for key, layer in pairs:
        modules[key] = layer
    leftovers = re.sub(r'\("([^"]+)",\s*"([^"]+)"\)', "", module_block)
    leftovers = leftovers.replace("const MODULES: &[(&str, &str)] = &[", "")
    for token in ("(", ")", ",", "&", " ", "\n", "\t"):
        leftovers = leftovers.replace(token, "")
    if leftovers:
        raise SystemExit(f"unparsed MODULES content: {leftovers!r}")
    return layers, modules


def _quoted_strings(block: str) -> list[str]:
    return re.findall(r'"([^"]*)"', block)


def is_ident(char: str) -> bool:
    return char.isascii() and (char.isalnum() or char == "_")


def comment_len(s: str) -> int | None:
    """Byte length of the comment `s` opens with, or None (port of the guard)."""
    if s.startswith("//"):
        newline = s.find("\n")
        return len(s) if newline < 0 else newline + 1
    if not s.startswith("/*"):
        return None
    depth, i = 1, 2
    while i + 1 < len(s) and depth > 0:
        if s[i] == "/" and s[i + 1] == "*":
            depth += 1
            i += 2
        elif s[i] == "*" and s[i + 1] == "/":
            depth -= 1
            i += 2
        else:
            i += 1
    return i if depth == 0 else len(s)


def skip_trivia(s: str) -> str:
    while True:
        s = s.lstrip()
        length = comment_len(s)
        if length is None:
            return s
        s = s[length:]


def path_segments(text: str) -> tuple[list[str], str]:
    segments: list[str] = []
    rest = skip_trivia(text)
    while True:
        segment = rest[2:] if rest.startswith("r#") else rest
        end = 0
        while end < len(segment) and is_ident(segment[end]):
            end += 1
        if end == 0:
            return segments, rest
        segments.append(segment[:end])
        after = skip_trivia(segment[end:])
        if after.startswith("::"):
            rest = skip_trivia(after[2:])
        else:
            return segments, segment[end:]


def collect_path(rest: str, prefix: list[str], paths: list[list[str]]) -> None:
    stripped = skip_trivia(rest)
    if stripped.startswith("{"):
        collect_group(stripped[1:], prefix, paths)
        return
    segments, remainder = path_segments(rest)
    stripped = skip_trivia(remainder)
    if stripped.startswith("{"):
        collect_group(stripped[1:], prefix + segments, paths)
        return
    leaf = (prefix + segments)[:2]
    if leaf:
        paths.append(leaf)


def collect_group(group: str, prefix: list[str], paths: list[list[str]]) -> None:
    depth, item_start, index = 1, 0, 0
    while index < len(group):
        if group[index] == "/" and (length := comment_len(group[index:])) is not None:
            index += length
            continue
        char = group[index]
        if char == "{":
            depth += 1
        elif char in ("}", ",") and depth == 1:
            item = group[item_start:index].strip()
            if item:
                collect_path(item, prefix, paths)
            item_start = index + 1
            if char == "}":
                break
        elif char == "}":
            depth -= 1
        index += 1


def keyword_paths(text: str, keyword: str) -> list[list[str]]:
    """Every `keyword::<path>` in `text`, `{...}` groups expanded (port)."""
    paths: list[list[str]] = []
    frm = 0
    while (found := text.find(keyword, frm)) >= 0:
        start = found
        frm = start + len(keyword)
        if start > 0 and is_ident(text[start - 1]):
            continue
        rest = skip_trivia(text[frm:])
        if not rest.startswith("::"):
            continue
        collect_path(rest[2:], [], paths)
    return paths


def opens_with_cli_cfg(text: str) -> bool:
    """The file's first item attribute gates it on cli or self-update."""
    head = skip_trivia(text)
    if not head.startswith("#!["):
        return False
    attr_end = head.find("]")
    if attr_end < 0:
        return False
    attr = head[:attr_end]
    return 'feature = "cli"' in attr or 'feature = "self-update"' in attr


def layer_of(segments: list[str], modules: dict[str, str]) -> str | None:
    if not segments:
        return None
    if len(segments) > 1:
        override = modules.get(f"{segments[0]}::{segments[1]}")
        if override is not None:
            return override
    return modules.get(segments[0])


def classify(path: Path, layers: list[str], modules: dict[str, str]) -> str:
    text = path.read_text(encoding="utf-8", errors="replace")
    referenced: set[str] = set()
    for segments in keyword_paths(text, "jubarte"):
        layer = layer_of(segments, modules)
        if layer is not None:
            referenced.add(layer)
    if "CARGO_BIN_EXE_jubarte" in text or opens_with_cli_cfg(text):
        referenced.add("cli")
    if not referenced or referenced <= {"dom", "opc"}:
        return "it_core"
    return "it_" + max(referenced, key=layers.index)


def main(argv: list[str]) -> int:
    layers, modules = load_tables()
    tests_dir = ROOT / "tests"
    targets = [tests_dir / name for name in argv] if argv else sorted(
        p for p in tests_dir.iterdir()
        if p.suffix == ".rs" and p.stem not in STANDALONE
    )
    counts: dict[str, int] = {}
    for path in targets:
        if not path.is_file() or path.suffix != ".rs":
            print(f"not a test file: {path}", file=sys.stderr)
            return 2
        if path.stem in STANDALONE:
            print(f"standalone, never classified: {path.name}", file=sys.stderr)
            return 2
        suite = classify(path, layers, modules)
        counts[suite] = counts.get(suite, 0) + 1
        print(f"tests/{path.name} {suite}")
    for suite in sorted(counts):
        print(f"{suite}: {counts[suite]}", file=sys.stderr)
    print(f"total: {sum(counts.values())}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

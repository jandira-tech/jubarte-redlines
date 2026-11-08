#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Refuse a Linux binary whose glibc symbol versions exceed a floor.

`python3 scripts/check_glibc_floor.py jubarte --max 2.28` reads the binary's
version needs (`readelf -V`, else `objdump -T`) and exits 1 when any
`GLIBC_x.y` is newer than the floor. `--musl` refuses any glibc need and,
on a real binary, a dynamic link. `--symbols FILE` parses readelf text
instead of a binary, which is how the tests feed a version dump.
"""

from __future__ import annotations

import argparse
import re
import shutil
import subprocess
import sys
from pathlib import Path

GLIBC = re.compile(r"GLIBC_(\d+)\.(\d+)")


def glibc_versions(text: str) -> list[tuple[int, int]]:
    """Every `GLIBC_x.y` in `text`, once, oldest first."""
    return sorted({(int(major), int(minor)) for major, minor in GLIBC.findall(text)})


def parse_floor(text: str) -> tuple[int, int]:
    major, minor = text.split(".", 1)
    return int(major), int(minor)


def newer_than(versions: list[tuple[int, int]], floor: tuple[int, int]) -> list[tuple[int, int]]:
    return [version for version in versions if version > floor]


def version_text(binary: Path) -> str:
    """`readelf -V` of `binary`, or `objdump -T` when readelf is absent."""
    for argv in (["readelf", "-V", str(binary)], ["objdump", "-T", str(binary)]):
        if shutil.which(argv[0]) is None:
            continue
        result = subprocess.run(argv, capture_output=True, text=True)
        if result.returncode == 0:
            return result.stdout + result.stderr
    print(f"readelf or objdump is required to read {binary}", file=sys.stderr)
    raise SystemExit(2)


def static_from_program_headers(text: str) -> bool:
    """True when `readelf -lW` text has no INTERP header.

    A static-pie Rust musl binary is `DYN`, like a PIE linked against
    glibc; only a dynamic one asks for a program interpreter.
    """
    return "Program Headers:" in text and not re.search(r"^\s*INTERP\b", text, re.M)


def static_from_descriptions(file_out: str, ldd_out: str) -> bool:
    """True when `file` or `ldd` describes a binary that needs no loader.

    `file` prints `statically linked` or `static-pie linked`; glibc's `ldd`
    prints `statically linked` for a static-pie and `not a dynamic
    executable` for a plain static binary.
    """
    if "statically linked" in file_out or "static-pie linked" in file_out:
        return True
    return "statically linked" in ldd_out or "not a dynamic executable" in ldd_out


def is_static(binary: Path) -> bool:
    """True when `binary` needs no dynamic loader: no INTERP program header
    (`readelf -lW`), else what `file` or `ldd` say."""
    if shutil.which("readelf"):
        result = subprocess.run(["readelf", "-lW", str(binary)], capture_output=True, text=True)
        if result.returncode == 0:
            return static_from_program_headers(result.stdout)
    file_out = ""
    if shutil.which("file"):
        file_out = subprocess.run(["file", str(binary)], capture_output=True, text=True).stdout
    ldd_out = ""
    if shutil.which("ldd"):
        result = subprocess.run(["ldd", str(binary)], capture_output=True, text=True)
        ldd_out = result.stdout + result.stderr
    return static_from_descriptions(file_out, ldd_out)


def spell(version: tuple[int, int]) -> str:
    return f"GLIBC_{version[0]}.{version[1]}"


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", nargs="?", type=Path, help="ELF to inspect")
    parser.add_argument("--symbols", type=Path, help="readelf -V text, instead of a binary")
    parser.add_argument("--max", default="2.28", help="newest allowed GLIBC_x.y (default 2.28)")
    parser.add_argument("--musl", action="store_true", help="refuse any glibc need; require a static binary")
    args = parser.parse_args(argv)
    if args.symbols is None and args.binary is None:
        parser.error("pass a binary or --symbols")
    if args.symbols is not None:
        text = args.symbols.read_text(encoding="utf-8", errors="replace")
    else:
        if not args.binary.is_file():
            print(f"not a file: {args.binary}", file=sys.stderr)
            return 2
        text = version_text(args.binary)
        if args.musl and not is_static(args.binary):
            print(f"{args.binary} is dynamically linked; a musl archive must be static", file=sys.stderr)
            return 1
    found = glibc_versions(text)
    if args.musl and found:
        print("musl archive needs " + ", ".join(spell(v) for v in found), file=sys.stderr)
        return 1
    floor = parse_floor(args.max)
    over = newer_than(found, floor)
    if over:
        print(
            f"needs {', '.join(spell(v) for v in over)}; floor is {spell(floor)}",
            file=sys.stderr,
        )
        return 1
    newest = spell(found[-1]) if found else "none"
    print(f"glibc floor ok: newest {newest}, max {spell(floor)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

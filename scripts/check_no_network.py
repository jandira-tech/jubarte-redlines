#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
"""check_no_network.py [--tree FILE] [--src DIR]: hold jubarte's document
paths off the network (#2018).

Word fetches a remote template, a linked picture and a subdocument the
moment it opens a file (word_exp N1, N2, N4, 2026-10-31). jubarte opens
the same files without a connection because it has no network code on any
document path. This check keeps it so, in two parts:

1. Without the `self-update` feature, no network crate is in the build:
   `cargo tree -e normal --no-default-features --features cli,fast-alloc`
   names none of NETWORK_CRATES. `--tree FILE` reads that output from a
   file instead of running cargo.
2. No Rust source outside the self-update code (`src/update.rs`,
   `src/update/`) names a network API or the self-update crate.

Exit 0: neither found. 1: at least one. 2: usage error.
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# Crates that open sockets or speak HTTP, TLS or DNS.
NETWORK_CRATES = {
    "attohttpc",
    "curl",
    "curl-sys",
    "h2",
    "hickory-resolver",
    "http-body",
    "hyper",
    "hyper-util",
    "isahc",
    "mio",
    "native-tls",
    "reqwest",
    "rustls",
    "self_update",
    "socket2",
    "surf",
    "trust-dns-resolver",
    "ureq",
}

TREE = [
    "cargo",
    "tree",
    "-e",
    "normal",
    "--no-default-features",
    "--features",
    "cli,fast-alloc",
    "--prefix",
    "none",
]

# A crate line in `cargo tree --prefix none` (or the default prefix):
# `name v1.2.3 ...`.
CRATE = re.compile(r"^[\s│├└─|`\\-]*(?P<name>[A-Za-z0-9_-]+) v\d")

# A network API in Rust source. Comments are stripped first.
API = re.compile(
    r"\bstd::net\b|\bTcpStream\b|\bTcpListener\b|\bUdpSocket\b"
    r"|\b(?:ureq|reqwest|hyper|self_update|curl|attohttpc|isahc|socket2)::"
    r"|\buse\s+(?:ureq|reqwest|hyper|self_update|curl|attohttpc|isahc|socket2)\b"
    r"|\bextern\s+crate\s+(?:ureq|reqwest|hyper|self_update|curl)\b"
)

COMMENT = re.compile(r"//[^\n]*")


def network_crates(tree: str) -> list[str]:
    """The network crates named in `cargo tree` output, sorted, once each."""
    found = set()
    for line in tree.splitlines():
        match = CRATE.match(line)
        if match and match.group("name") in NETWORK_CRATES:
            found.add(match.group("name"))
    return sorted(found)


def allowed(path: Path, src: Path) -> bool:
    """The self-update code may reach the network."""
    rel = path.relative_to(src).as_posix()
    return rel == "update.rs" or rel.startswith("update/")


def network_uses(src: Path) -> list[str]:
    """Each `file:line: text` outside the self-update code that names a
    network API."""
    uses = []
    for path in sorted(src.rglob("*.rs")):
        if allowed(path, src):
            continue
        for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            code = COMMENT.sub("", line)
            if API.search(code):
                uses.append(f"{path.relative_to(src.parent).as_posix()}:{number}: {line.strip()}")
    return uses


def main(argv: list[str]) -> int:
    tree_file: Path | None = None
    src = ROOT / "src"
    args = list(argv)
    while args:
        flag = args.pop(0)
        if flag in ("--tree", "--src") and args:
            value = Path(args.pop(0))
            if flag == "--tree":
                tree_file = value
            else:
                src = value
        else:
            print(__doc__.splitlines()[0], file=sys.stderr)
            return 2
    if not src.is_dir():
        print(f"error: {src} is not a directory", file=sys.stderr)
        return 2
    if tree_file is not None:
        try:
            tree = tree_file.read_text(encoding="utf-8")
        except OSError as error:
            print(f"error: {error}", file=sys.stderr)
            return 2
    else:
        run = subprocess.run(TREE, cwd=ROOT, capture_output=True, text=True, check=False)
        if run.returncode != 0:
            print(f"error: cargo tree failed: {run.stderr.strip()}", file=sys.stderr)
            return 2
        tree = run.stdout
    crates = network_crates(tree)
    uses = network_uses(src)
    for crate in crates:
        print(f"network crate without self-update: {crate}")
    for use in uses:
        print(f"network API outside src/update: {use}")
    if crates or uses:
        return 1
    print("no network crate without self-update; no network API outside src/update")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

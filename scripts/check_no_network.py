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
   `cargo tree -e normal --target all --no-default-features` for the
   library, the CLI and the CLI with its allocator names none of
   NETWORK_CRATES. `--tree FILE` reads that output from a file instead.
2. No Rust source outside the self-update code (`src/update.rs`,
   `src/update/`) names a network API, a network crate, or a command that
   fetches (`Command::new("curl")`), comments aside.

It is a tripwire, not a proof: a crate missing from NETWORK_CRATES, a
build script, or a spelling the patterns do not know passes it.

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
    "async-std",
    "attohttpc",
    "curl",
    "curl-sys",
    "h2",
    "hickory-proto",
    "hickory-resolver",
    "http-body",
    "hyper",
    "hyper-util",
    "isahc",
    "minreq",
    "mio",
    "native-tls",
    "openssl",
    "openssl-sys",
    "reqwest",
    "rustls",
    "s2n-tls",
    "self_update",
    "smol",
    "socket2",
    "surf",
    "tokio",
    "trust-dns-resolver",
    "tungstenite",
    "ureq",
}

# Every feature set the shipped artifacts build without `self-update`: the
# library alone, the CLI, and the CLI with its allocator; for every target.
FEATURE_SETS = ["", "cli", "cli,fast-alloc"]


def tree_command(features: str) -> list[str]:
    """`cargo tree` over the normal dependencies of one feature set, on
    every target, one crate per line."""
    command = ["cargo", "tree", "-e", "normal", "--target", "all", "--no-default-features"]
    if features:
        command += ["--features", features]
    return command + ["--prefix", "none"]

# A crate line in `cargo tree --prefix none` (or the default prefix):
# `name v1.2.3 ...`.
CRATE = re.compile(r"^[\s│├└─|`\\-]*(?P<name>[A-Za-z0-9_-]+) v\d")

_CRATES = r"(?:ureq|reqwest|hyper|self_update|curl|attohttpc|isahc|socket2|minreq|tokio|smol|async_std|tungstenite|openssl)"

# A network API in Rust source, or a command that fetches over the network.
# Comments are stripped first, string literals kept. A Unix-domain socket
# (`std::os::unix::net`) is local IPC and reaches no host: the debug-build
# publication test hook in src/bin/jubarte.rs uses one.
API = re.compile(
    r"\bstd::net\b|\bTcpStream\b|\bTcpListener\b|\bUdpSocket\b"
    rf"|\b{_CRATES}::"
    rf"|\buse\s+{_CRATES}\b"
    rf"|\bextern\s+crate\s+{_CRATES}\b"
    r"|Command::new\(\s*\"(?:curl|wget|nc|ncat|ssh|scp|ftp|powershell|Invoke-WebRequest)(?:\.exe)?\""
)

# A comment or a string literal: `//` inside a string is no comment, and a
# `/* … */` block is one. Raw strings (`r"…"`, `r#"…"#`) are matched whole.
TOKEN = re.compile(
    r'(?P<raw>r(?P<hashes>#*)"(?:.|\n)*?"(?P=hashes))'
    r'|(?P<string>"(?:\\.|[^"\\])*")'
    r"|(?P<char>'(?:\\.|[^'\\])')"
    r"|(?P<line>//[^\n]*)"
    r"|(?P<block>/\*(?:.|\n)*?\*/)"
)


def strip_comments(source: str) -> str:
    """`source` with each comment blanked (its newlines kept, so line
    numbers hold) and every string literal left as written."""

    def blank(match: re.Match[str]) -> str:
        text = match.group(0)
        if match.group("line") or match.group("block"):
            return "".join("\n" if c == "\n" else " " for c in text)
        return text

    return TOKEN.sub(blank, source)


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
        text = path.read_text(encoding="utf-8")
        code = strip_comments(text).splitlines()
        for number, (line, kept) in enumerate(zip(text.splitlines(), code), 1):
            if API.search(kept):
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
        trees = []
        for features in FEATURE_SETS:
            run = subprocess.run(tree_command(features), cwd=ROOT, capture_output=True, text=True, check=False)
            if run.returncode != 0:
                print(f"error: cargo tree failed: {run.stderr.strip()}", file=sys.stderr)
                return 2
            trees.append(run.stdout)
        tree = "\n".join(trees)
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

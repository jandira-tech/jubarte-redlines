#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

# SPDX-License-Identifier: AGPL-3.0-only

"""Refuse release evidence scored on another engine than the release tree's.

`python3 scripts/check_evidence_engine.py 0.14.0` reads the commit both of
release_info/'s results JSONs name (`tools.jubarte.commit`, the commit the
scored candidate was built from) and compares the CLI engine's inputs there
with REV's (default HEAD): assets/, schemas/, keys/ and Cargo.lock byte for
byte; the Rust sources under src/ token for token, so a change to comments
alone (0.12.0's doc comment) is no engine change; and Cargo.toml with the
[package] fields cargo builds nothing from set aside: the version (step 1's
bump), the metadata (step 4's release notes) and the crates.io listing
(0.11.2's keywords). The bump's line in Cargo.lock is set aside the same
way. scripts/release.sh runs it in step 3, so the evidence is proven to be
the release's engine rather than taken on a person's word (#1752).

Exit 0 when they match; 1 naming each differing input; 2 when the results
or the commit cannot be read. Plain Python 3, stdlib only.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import tomllib
from pathlib import Path

# the files the CLI embeds or is built from: Rust sources token for token,
# every other file byte for byte
INPUTS = ("src", "assets", "schemas", "keys")
# [package] fields cargo builds nothing from: the version, the metadata
# tables and the crates.io listing and packaging
LISTING = frozenset({
    "version", "metadata", "description", "keywords", "categories", "readme",
    "homepage", "repository", "documentation", "authors", "license",
    "license-file", "include", "exclude", "publish",
})
RAW_STRING = re.compile(r'[bc]?r(#*)"')
CHAR = re.compile(r"'(?:\\(?:u\{[0-9A-Fa-f_]+\}|x[0-9A-Fa-f]{2}|.)|[^\\'\n])'")


def rust_tokens(text: str) -> str:
    """A Rust source without its comments, whitespace runs collapsed: two
    sources that differ only in comments (doc comments too) or layout
    compile to the same program. String, raw-string and char literals are
    kept verbatim, and block comments nest, as rustc reads them."""
    out, i, n = [], 0, len(text)
    while i < n:
        if text.startswith("//", i):
            i = text.find("\n", i)
            i = n if i < 0 else i
            out.append(" ")
            continue
        if text.startswith("/*", i):
            depth, i = 1, i + 2
            while i < n and depth:
                if text.startswith("/*", i):
                    depth, i = depth + 1, i + 2
                elif text.startswith("*/", i):
                    depth, i = depth - 1, i + 2
                else:
                    i += 1
            out.append(" ")
            continue
        prev = text[i - 1] if i else " "
        raw = RAW_STRING.match(text, i) if not (prev.isalnum() or prev == "_") else None
        if raw:
            end = text.find('"' + raw.group(1), raw.end())
            end = n if end < 0 else end + 1 + len(raw.group(1))
            out.append(text[i:end])
            i = end
            continue
        if text[i] == '"':
            j = i + 1
            while j < n and text[j] != '"':
                j += 2 if text[j] == "\\" else 1
            out.append(text[i:j + 1])
            i = j + 1
            continue
        if text[i] == "'":
            char = CHAR.match(text, i)
            if char:  # a char literal; otherwise a lifetime or a label
                out.append(char.group(0))
                i = char.end()
                continue
        if text[i].isspace():
            while i < n and text[i].isspace():
                i += 1
            out.append(" ")
            continue
        out.append(text[i])
        i += 1
    return re.sub(r" +", " ", "".join(out)).strip()


def git(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(["git", *args], capture_output=True, text=True)


def usage(message: str) -> int:
    print(f"check_evidence_engine: {message}", file=sys.stderr)
    return 2


def evidence_commits(ver: str) -> dict[str, str] | str:
    """{results file: commit} for both results JSONs, or the reason they cannot be read."""
    out = {}
    for kind in ("redline", "conversion"):
        found = sorted(Path("release_info").glob(f"results_{kind}_{ver}_*.json"))
        if len(found) != 1:
            return f"release_info/ holds {len(found)} results_{kind}_{ver}_*.json, not one"
        try:
            commit = json.loads(found[0].read_text())["tools"]["jubarte"]["commit"]
        except (OSError, ValueError, KeyError, TypeError) as e:
            return f"{found[0]} names no tools.jubarte.commit ({e})"
        out[str(found[0])] = commit
    return out


def toml_at(rev: str, path: str) -> dict | None:
    shown = git("show", f"{rev}:{path}")
    return tomllib.loads(shown.stdout) if shown.returncode == 0 else None


def manifest(doc: dict | None) -> dict | None:
    """Cargo.toml without the [package] fields cargo builds nothing from."""
    if doc is None:
        return None
    package = {k: v for k, v in doc.get("package", {}).items() if k not in LISTING}
    return {**doc, "package": package}


def text_at(rev: str, path: str) -> str | None:
    shown = git("show", f"{rev}:{path}")
    return shown.stdout if shown.returncode == 0 else None


def lockfile(doc: dict | None) -> dict | None:
    """Cargo.lock without the versions of this workspace's own crates (step 1 bumps them)."""
    if doc is None:
        return None
    packages = doc.get("package", [])
    local = {p["name"] for p in packages if "source" not in p}
    out = []
    for p in packages:
        p = dict(p)
        if p["name"] in local:
            p.pop("version", None)
        p["dependencies"] = [d.split(" ", 1)[0] if d.split(" ", 1)[0] in local else d
                             for d in p.get("dependencies", [])]
        out.append(p)
    return {**doc, "package": out}


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("version")
    ap.add_argument("--rev", default="HEAD", help="the release tree (default HEAD)")
    args = ap.parse_args()

    commits = evidence_commits(args.version)
    if isinstance(commits, str):
        return usage(commits)
    named = set(commits.values())
    if len(named) != 1:
        listed = ", ".join(f"{f}: {c}" for f, c in commits.items())
        print(f"check_evidence_engine: the results disagree on the scored commit ({listed})", file=sys.stderr)
        return 1
    commit = named.pop()
    for rev in (commit, args.rev):
        if git("cat-file", "-e", f"{rev}^{{commit}}").returncode != 0:
            return usage(f"{rev} is no commit of this repository — git fetch, or the evidence names a build from elsewhere")

    diff = git("diff", "--name-only", commit, args.rev, "--", *INPUTS)
    if diff.returncode != 0:
        return usage(f"git diff {commit} {args.rev} failed: {diff.stderr.strip()}")
    changed, comments = [], []
    for path in diff.stdout.split():
        before, after = text_at(commit, path), text_at(args.rev, path)
        if (path.endswith(".rs") and before is not None and after is not None
                and rust_tokens(before) == rust_tokens(after)):
            comments.append(path)
        else:
            changed.append(path)
    if manifest(toml_at(commit, "Cargo.toml")) != manifest(toml_at(args.rev, "Cargo.toml")):
        changed.append("Cargo.toml (beyond its version, metadata and crates.io listing)")
    if lockfile(toml_at(commit, "Cargo.lock")) != lockfile(toml_at(args.rev, "Cargo.lock")):
        changed.append("Cargo.lock (beyond this workspace's own versions)")
    if changed:
        print(f"check_evidence_engine: the evidence was scored on {commit[:12]}, and the CLI engine "
              f"changed since in {args.rev}:", file=sys.stderr)
        for path in changed:
            print(f"  ! {path}", file=sys.stderr)
        return 1
    for path in comments:
        print(f"  comment-only since the evidence: {path}")
    print(f"evidence engine {commit[:12]}: the CLI engine inputs equal {args.rev}'s")
    return 0


if __name__ == "__main__":
    sys.exit(main())

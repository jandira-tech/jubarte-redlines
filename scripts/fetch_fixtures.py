#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: Apache-2.0

"""Download the test documents that are kept outside this repository.

Some documents the tests read are third-party files whose origin is not
confirmed. They are not tracked here: they live in a public Hugging Face
dataset, and `tests/corpus/hf_fixtures.sha256` names that dataset, the
revision to take and the SHA-256 of every file. This script puts each file
at its path under `tests/corpus/` and refuses one whose hash differs.

    python3 scripts/fetch_fixtures.py           download what is missing
    python3 scripts/fetch_fixtures.py --check   verify only, no network

Only the Python standard library is used. A file already on disk with the
right hash is left alone, so a second run downloads nothing.
"""

from __future__ import annotations

import argparse
import hashlib
import os
import sys
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ROOT / "tests" / "corpus" / "hf_fixtures.sha256"
DEST = ROOT / "tests" / "corpus"
HOST = "https://huggingface.co"
ATTEMPTS = 4


class ManifestError(Exception):
    pass


def read_manifest(path: Path) -> tuple[str, str, list[tuple[str, str]]]:
    """Return (dataset, revision, [(sha256, relative path)])."""
    fields: dict[str, str] = {}
    files: list[tuple[str, str]] = []
    for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        if not line.strip():
            continue
        if line.startswith("#"):
            key, sep, value = line[1:].partition(":")
            if sep and key.strip() in ("dataset", "revision"):
                fields[key.strip()] = value.strip()
            continue
        digest, sep, name = line.partition("  ")
        if not sep or len(digest) != 64 or any(c not in "0123456789abcdef" for c in digest):
            raise ManifestError(f"{path.name}:{number}: expected '<sha256>  <path>'")
        parts = Path(name).parts
        # ":" too: on Windows "C:x.docx" is not absolute, yet joins outside the folder.
        if not name or Path(name).is_absolute() or ".." in parts or "\\" in name or ":" in name:
            raise ManifestError(f"{path.name}:{number}: unsafe path {name!r}")
        files.append((digest, name))
    for key in ("dataset", "revision"):
        if key not in fields:
            raise ManifestError(f"{path.name}: no '# {key}:' line")
    revision = fields["revision"]
    if len(revision) != 40 or any(c not in "0123456789abcdef" for c in revision):
        raise ManifestError(f"{path.name}: the revision must be a full commit hash")
    if not files:
        raise ManifestError(f"{path.name}: lists no file")
    return fields["dataset"], revision, files


def sha256_of(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def state(dest: Path, digest: str, name: str) -> str:
    """'ok', 'missing' or 'differs' for one manifest entry."""
    target = dest / name
    if not target.is_file():
        return "missing"
    return "ok" if sha256_of(target) == digest else "differs"


def download(url: str, target: Path, digest: str) -> None:
    target.parent.mkdir(parents=True, exist_ok=True)
    last: Exception | None = None
    for attempt in range(ATTEMPTS):
        if attempt:
            time.sleep(2**attempt)
        handle, temp = tempfile.mkstemp(dir=target.parent, prefix=".fetch-")
        try:
            request = urllib.request.Request(url, headers={"User-Agent": "jubarte-fetch-fixtures"})
            # Open the temporary file first: a failed request must still close
            # it, or Windows cannot remove it and the real error is lost.
            with os.fdopen(handle, "wb") as out, urllib.request.urlopen(request, timeout=60) as response:
                for block in iter(lambda: response.read(1 << 20), b""):
                    out.write(block)
            got = sha256_of(Path(temp))
            if got != digest:
                raise ManifestError(f"{target.name}: downloaded {got}, the manifest says {digest}")
            os.replace(temp, target)
            return
        except ManifestError:
            Path(temp).unlink(missing_ok=True)
            raise
        except (urllib.error.URLError, OSError) as error:
            Path(temp).unlink(missing_ok=True)
            last = error
    raise OSError(f"{url}: {last}")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--check", action="store_true", help="verify the files on disk; download nothing")
    parser.add_argument("--manifest", type=Path, default=MANIFEST, help=argparse.SUPPRESS)
    parser.add_argument("--dest", type=Path, default=DEST, help=argparse.SUPPRESS)
    parser.add_argument("--host", default=HOST, help=argparse.SUPPRESS)
    args = parser.parse_args(argv)

    try:
        dataset, revision, files = read_manifest(args.manifest)
    except (OSError, ManifestError) as error:
        print(f"fetch_fixtures: {error}", file=sys.stderr)
        return 2

    wanted = [(digest, name, state(args.dest, digest, name)) for digest, name in files]
    todo = [(digest, name, what) for digest, name, what in wanted if what != "ok"]
    if args.check:
        for _, name, what in todo[:10]:
            print(f"fetch_fixtures: {name}: {what}", file=sys.stderr)
        if todo:
            print(
                f"fetch_fixtures: {len(todo)} of {len(files)} files are not as the manifest says.\n"
                "Run: python3 scripts/fetch_fixtures.py",
                file=sys.stderr,
            )
            return 1
        print(f"fetch_fixtures: {len(files)} files match the manifest")
        return 0

    base = f"{args.host}/datasets/{dataset}/resolve/{revision}/"

    def one(entry: tuple[str, str, str]) -> str | None:
        digest, name, _ = entry
        try:
            download(base + urllib.parse.quote(name), args.dest / name, digest)
        except (OSError, ManifestError) as error:
            return str(error)
        return None

    with ThreadPoolExecutor(max_workers=8) as pool:
        failures = [error for error in pool.map(one, todo) if error]
    for error in failures:
        print(f"fetch_fixtures: {error}", file=sys.stderr)
    if failures:
        print(f"fetch_fixtures: {len(failures)} of {len(todo)} downloads failed", file=sys.stderr)
        return 1
    print(f"fetch_fixtures: {len(files)} files present ({len(todo)} downloaded) from {dataset}@{revision[:12]}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

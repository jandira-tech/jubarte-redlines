#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: Apache-2.0

"""Check that every commit of a pull request is signed off.

CONTRIBUTING.md asks for `git commit -s`: a `Signed-off-by:` trailer by
which the author states the Developer Certificate of Origin. This script
reads the commits `BASE..HEAD` and fails on one without a trailer that
names its author or committer. Merge commits and commits by bots are not
checked.

    python3 scripts/check_dco.py BASE HEAD
"""

from __future__ import annotations

import re
import subprocess
import sys

SEP = "\x1e"
FIELD = "\x1f"
TRAILER = re.compile(r"^Signed-off-by:\s*(.+?)\s*<([^<>\s]+)>\s*$", re.MULTILINE | re.IGNORECASE)


def unsigned(log: str) -> list[str]:
    """Return 'hash subject' for each commit in `log` that lacks a sign-off.

    `log` is `git log --format=%H%x1f%an%x1f%ae%x1f%cn%x1f%ce%x1f%P%x1f%B%x1e`.
    """
    missing = []
    for record in log.split(SEP):
        record = record.strip("\n")
        if not record:
            continue
        sha, author, author_mail, committer, committer_mail, parents, body = record.split(FIELD, 6)
        if len(parents.split()) > 1:
            continue
        if author.endswith("[bot]") or author_mail.endswith("[bot]@users.noreply.github.com"):
            continue
        people = {author.lower(), committer.lower()}
        mails = {author_mail.lower(), committer_mail.lower()}
        signed = any(name.lower() in people or mail.lower() in mails for name, mail in TRAILER.findall(body))
        if not signed:
            subject = body.strip().splitlines()[0] if body.strip() else ""
            missing.append(f"{sha[:12]} {subject}")
    return missing


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print("usage: check_dco.py BASE HEAD", file=sys.stderr)
        return 2
    base, head = argv
    result = subprocess.run(
        ["git", "log", "--format=%H%x1f%an%x1f%ae%x1f%cn%x1f%ce%x1f%P%x1f%B%x1e", f"{base}..{head}"],
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        print(f"check_dco: {result.stderr.strip()}", file=sys.stderr)
        return 2
    missing = unsigned(result.stdout)
    if missing:
        print("check_dco: these commits are not signed off by their author:", file=sys.stderr)
        for line in missing:
            print(f"  {line}", file=sys.stderr)
        print(
            f"Sign off with `git commit -s` (see CONTRIBUTING.md). To fix a branch:\n  git rebase --signoff {base}",
            file=sys.stderr,
        )
        return 1
    print("check_dco: every commit is signed off")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

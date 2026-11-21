#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: Apache-2.0

"""scripts/check_dco.py finds commits without the author's sign-off."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_dco

F, S = check_dco.FIELD, check_dco.SEP


def commit(body: str, *, author=("Ana Dev", "ana@example.org"), committer=None, parents="p1", sha="a" * 40) -> str:
    committer = committer or author
    return F.join([sha, *author, *committer, parents, body]) + S + "\n"


class CheckDco(unittest.TestCase):
    def test_a_signed_commit_passes(self) -> None:
        log = commit("fix: x\n\nSigned-off-by: Ana Dev <ana@example.org>\n")
        self.assertEqual(check_dco.unsigned(log), [])

    def test_an_unsigned_commit_is_named(self) -> None:
        log = commit("fix: the thing\n\nbody\n", sha="b" * 40)
        self.assertEqual(check_dco.unsigned(log), ["bbbbbbbbbbbb fix: the thing"])

    def test_someone_elses_sign_off_does_not_count(self) -> None:
        log = commit("fix: x\n\nSigned-off-by: Other Person <other@example.org>\n")
        self.assertEqual(len(check_dco.unsigned(log)), 1)

    def test_the_address_alone_is_enough(self) -> None:
        # a display name that differs from the git name (a nickname)
        log = commit("fix: x\n\nSigned-off-by: A. Dev <ANA@example.org>\n")
        self.assertEqual(check_dco.unsigned(log), [])

    def test_the_committers_sign_off_counts(self) -> None:
        log = commit(
            "fix: x\n\nSigned-off-by: Mia Maint <mia@example.org>\n",
            committer=("Mia Maint", "mia@example.org"),
        )
        self.assertEqual(check_dco.unsigned(log), [])

    def test_a_sign_off_in_the_middle_of_a_line_does_not_count(self) -> None:
        log = commit("fix: x\n\nsee Signed-off-by: Ana Dev <ana@example.org>\n")
        self.assertEqual(len(check_dco.unsigned(log)), 1)

    def test_merges_and_bots_are_not_checked(self) -> None:
        log = commit("Merge branch 'x'\n", parents="p1 p2") + commit(
            "build(deps): bump x\n", author=("dependabot[bot]", "49699333+dependabot[bot]@users.noreply.github.com")
        )
        self.assertEqual(check_dco.unsigned(log), [])

    def test_each_unsigned_commit_of_several_is_listed(self) -> None:
        log = (
            commit("one\n", sha="1" * 40)
            + commit("two\n\nSigned-off-by: Ana Dev <ana@example.org>\n", sha="2" * 40)
            + commit("three\n", sha="3" * 40)
        )
        self.assertEqual([line.split()[0] for line in check_dco.unsigned(log)], ["1" * 12, "3" * 12])


if __name__ == "__main__":
    unittest.main()

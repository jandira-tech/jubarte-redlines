#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Signatures in the flattened API listing (scripts/api_snapshot.py).

A public function whose generic bounds change is a source-breaking change,
so its `.api.txt` line has to change with them (Codex review on #247).
"""

from __future__ import annotations

import gzip
import importlib.util
import tempfile
import unittest
from pathlib import Path
from unittest import mock

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("api_snapshot", HERE / "api_snapshot.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)


def trait(path: str) -> dict:
    return {"trait_bound": {"trait": {"path": path, "args": None}, "generic_params": [], "modifier": "none"}}


def function(params: list, where: list) -> dict:
    return {
        "sig": {"inputs": [["x", {"generic": "T"}]], "output": None, "is_c_variadic": False},
        "generics": {"params": params, "where_predicates": where},
        "header": {"is_const": False, "is_unsafe": False, "is_async": False},
    }


T_PARAM = {"name": "T", "kind": {"type": {"bounds": [trait("Read")], "default": None, "is_synthetic": False}}}


class Signatures(unittest.TestCase):
    def test_a_bound_is_part_of_the_signature(self) -> None:
        self.assertEqual(api._sig(function([T_PARAM], [])), "<T: Read>(x: T)")
        loose = {"name": "T", "kind": {"type": {"bounds": [], "default": None, "is_synthetic": False}}}
        self.assertNotEqual(api._sig(function([T_PARAM], [])), api._sig(function([loose], [])))

    def test_where_predicates_lifetimes_and_consts_are_kept(self) -> None:
        params = [
            {"name": "'a", "kind": {"lifetime": {"outlives": ["'b"]}}},
            T_PARAM,
            {"name": "N", "kind": {"const": {"type": {"primitive": "usize"}, "default": None}}},
        ]
        where = [{"bound_predicate": {"type": {"generic": "T"}, "bounds": [trait("Send")], "generic_params": []}}]
        self.assertEqual(
            api._sig(function(params, where)),
            "<'a: 'b, T: Read, const N: usize>(x: T) where T: Send",
        )

    def test_impl_trait_arguments_stay_in_the_argument(self) -> None:
        synthetic = {"name": "impl Read", "kind": {"type": {"bounds": [trait("Read")], "default": None, "is_synthetic": True}}}
        fn = function([synthetic], [])
        fn["sig"]["inputs"] = [["x", {"impl_trait": [trait("Read")]}]]
        self.assertEqual(api._sig(fn), "(x: impl Read)")

    def test_a_function_without_generics_is_unchanged(self) -> None:
        fn = function([], [])
        fn["sig"]["inputs"] = [["x", {"primitive": "u8"}]]
        self.assertEqual(api._sig(fn), "(x: u8)")


class Snapshot(unittest.TestCase):
    def test_the_gzip_is_the_same_bytes_at_any_time(self) -> None:
        # A resumed release rewrites the snapshot; a header timestamp would
        # dirty docs/api and push another build commit (Codex on #247).
        with tempfile.TemporaryDirectory() as tmp:
            first, second = Path(tmp) / "a.json.gz", Path(tmp) / "b.json.gz"
            with mock.patch.object(gzip.time, "time", return_value=1_000_000_000.0):
                api.write_gz(first, b"{}")
            with mock.patch.object(gzip.time, "time", return_value=2_000_000_000.0):
                api.write_gz(second, b"{}")
            self.assertEqual(first.read_bytes(), second.read_bytes())
            self.assertEqual(gzip.decompress(first.read_bytes()), b"{}")


if __name__ == "__main__":
    unittest.main()

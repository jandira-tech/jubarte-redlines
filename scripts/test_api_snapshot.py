#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""The flattened API listing and its drift report (scripts/api_snapshot.py).

A public function whose generic bounds change is a source-breaking change,
so its `.api.txt` line has to change with them (Codex review on #247).

The drift a releaser signs off at step 6 has to be readable: the 0.11.2 one
ran to 700 lines, nearly all of them blanket and auto-trait impls rustdoc
writes for every type, with the public surface lost among crate-private
items.
"""

from __future__ import annotations

import contextlib
import gzip
import importlib.util
import io
import json
import shutil
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


def item(iid: int, name, kind: str, inner, vis="public", crate: int = 0) -> dict:
    return {"id": iid, "crate_id": crate, "name": name, "visibility": vis, "inner": {kind: inner}}


def fn() -> dict:
    return {
        "sig": {"inputs": [], "output": None, "is_c_variadic": False},
        "generics": {"params": [], "where_predicates": []},
        "header": {"is_const": False, "is_unsafe": False, "is_async": False},
    }


def impl(target: int, target_name: str, trait_path, items: list, blanket=None, synthetic=False) -> dict:
    return {
        "trait": {"path": trait_path, "id": 900, "args": None} if trait_path else None,
        "for": {"resolved_path": {"path": target_name, "id": target, "args": None}},
        "items": items,
        "generics": {"params": [], "where_predicates": []},
        "blanket_impl": blanket,
        "is_synthetic": synthetic,
    }


def struct(fields: list, impls: list) -> dict:
    return {"kind": {"plain": {"fields": fields, "has_stripped_fields": False}}, "impls": impls}


U8 = {"primitive": "u8"}
RESTRICTED = {"restricted": {"parent": 1, "path": "::api"}}


def demo_doc() -> dict:
    """A crate with one public module, one crate-private module whose
    `Shown` is re-exported at the root, and a struct carrying every kind of
    impl rustdoc writes."""
    items = [
        item(0, "demo", "module", {"is_crate": True, "items": [1, 2, 9]}),
        item(1, "api", "module", {"items": [3, 4, 5]}),
        item(2, "hidden", "module", {"items": [8, 18]}, vis="crate"),
        item(3, "Widget", "struct", struct([6, 7], [10, 11, 12, 13, 19])),
        item(4, "make", "function", fn()),
        item(5, "helper", "function", fn(), vis="crate"),
        item(6, "size", "struct_field", U8),
        item(7, "secret", "struct_field", U8, vis="default"),
        item(8, "Shown", "struct", struct([], [])),
        item(9, None, "use", {"source": "hidden::Shown", "name": "Shown", "id": 8, "is_glob": False}),
        item(10, None, "impl", impl(3, "Widget", None, [14, 15]), vis="default"),
        item(11, None, "impl", impl(3, "Widget", "Clone", [16]), vis="default"),
        item(12, None, "impl", impl(3, "Widget", "Into", [17, 20], blanket={"generic": "T"}), vis="default"),
        item(13, None, "impl", impl(3, "Widget", "Send", [], synthetic=True), vis="default"),
        item(14, "new", "function", fn()),
        item(15, "tweak", "function", fn(), vis=RESTRICTED),
        item(16, "clone", "function", fn(), vis="default"),
        item(17, "into", "function", fn(), vis="default", crate=2),
        item(18, "Buried", "struct", struct([], [])),
        item(19, None, "impl", impl(3, "Widget", "StructuralPartialEq", []), vis="default"),
        item(20, "Output", "assoc_type", {"type": {"generic": "T"}}, vis="default", crate=2),
    ]
    paths = {
        "0": ["demo"], "1": ["demo", "api"], "2": ["demo", "hidden"],
        "3": ["demo", "api", "Widget"], "4": ["demo", "api", "make"],
        "5": ["demo", "api", "helper"], "8": ["demo", "hidden", "Shown"],
        "18": ["demo", "hidden", "Buried"],
    }
    return {
        "root": 0,
        "index": {str(i["id"]): i for i in items},
        "paths": {k: {"crate_id": 0, "path": v, "kind": "x"} for k, v in paths.items()},
    }


class Listing(unittest.TestCase):
    def test_the_listing_is_the_public_surface_then_the_crate_private_rest(self) -> None:
        self.assertEqual(api.flatten(demo_doc()), [
            "api\tfunction\tpub\tdemo::api::Widget::new ()",
            "api\tfunction\tpub\tdemo::api::make ()",
            "api\timpl\t\tdemo::api::Widget::(impl Clone)",
            "api\tmodule\tpub\tdemo",
            "api\tmodule\tpub\tdemo::api",
            "api\tstruct\tpub\tdemo::api::Widget",
            "api\tstruct\tpub\tdemo::hidden::Shown",
            "api\tstruct_field\tpub\tdemo::api::Widget::size: u8",
            "api\tuse\tpub\tdemo::Shown = hidden::Shown",
            "internal\tfunction\tpub(crate)\tdemo::api::helper ()",
            "internal\tfunction\tpub(crate::api)\tdemo::api::Widget::tweak ()",
            "internal\tmodule\tpub(crate)\tdemo::hidden",
            "internal\tstruct\tpub\tdemo::hidden::Buried",
            "internal\tstruct_field\t\tdemo::api::Widget::secret: u8",
        ])

    def test_blanket_and_auto_trait_impls_are_not_listed(self) -> None:
        # rustdoc writes `impl Into<U> for Widget`, `impl Send for Widget` and
        # a dozen more for every type; their shared items (`into`, `Output`)
        # were listed under whichever type came first, a name that flipped
        # between releases.
        text = "\n".join(api.flatten(demo_doc()))
        for noise in ("Into", "Send", "into", "Output", "StructuralPartialEq"):
            self.assertNotIn(noise, text)

    def test_a_trait_impl_is_one_line(self) -> None:
        # `impl Clone` says it all: `clone`, `fmt`, `eq` add a line per impl.
        text = "\n".join(api.flatten(demo_doc()))
        self.assertIn("(impl Clone)", text)
        self.assertNotIn("clone", text)

    def test_the_listing_does_not_depend_on_the_order_of_the_index(self) -> None:
        doc = demo_doc()
        backwards = dict(doc, index=dict(reversed(list(doc["index"].items()))))
        self.assertEqual(api.flatten(doc), api.flatten(backwards))

    def test_a_private_field_of_a_public_struct_is_not_public(self) -> None:
        by_path = {e.path: e for e in api.entries(demo_doc())}
        self.assertEqual(by_path["demo::api::Widget::size"].surface, "api")
        self.assertEqual(by_path["demo::api::Widget::secret"].surface, "internal")

    def test_a_snapshot_is_written_in_this_format(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            built = Path(tmp) / "jubarte.json"
            built.write_text(json.dumps(demo_doc()))
            with mock.patch.object(api, "build_doc_json"):
                with mock.patch.object(api, "DOC_JSON", built):
                    with mock.patch.object(api, "API_DIR", Path(tmp) / "api"):
                        with contextlib.redirect_stdout(io.StringIO()):
                            api.main(["9.9.9"])
            listing = (Path(tmp) / "api" / "jubarte-v9.9.9.api.txt").read_text()
            self.assertEqual(listing.splitlines(), api.flatten(demo_doc()))
            stored = gzip.decompress((Path(tmp) / "api" / "jubarte-v9.9.9.json.gz").read_bytes())
            self.assertEqual(json.loads(stored), demo_doc())


def entry(surface: str, kind: str, vis: str, path: str, detail: str = "") -> "api.Entry":
    return api.Entry(surface, kind, vis, path, detail)


class Drift(unittest.TestCase):
    OLD = [
        entry("api", "module", "pub", "demo"),
        entry("api", "module", "pub", "demo::api"),
        entry("api", "function", "pub", "demo::api::make", " (x: u8)"),
        entry("api", "function", "pub", "demo::api::gone", " ()"),
        entry("internal", "module", "pub(crate)", "demo::hidden"),
        entry("internal", "function", "pub(crate)", "demo::hidden::helper", " ()"),
        entry("internal", "function", "pub(crate)", "demo::hidden::dropped", " ()"),
    ]
    NEW = [
        entry("api", "module", "pub", "demo"),
        entry("api", "module", "pub", "demo::api"),
        entry("api", "function", "pub", "demo::api::make", " (x: u16)"),
        entry("api", "function", "pub", "demo::api::fresh", " ()"),
        entry("internal", "module", "pub(crate)", "demo::hidden"),
        entry("internal", "function", "pub(crate)", "demo::hidden::helper", " (y: u8)"),
        entry("internal", "function", "pub(crate)", "demo::hidden::added", " ()"),
        entry("internal", "constant", "pub(crate)", "demo::hidden::deep::LIMIT", ": u8"),
    ]

    def test_the_public_surface_comes_first_and_in_full(self) -> None:
        # What can break a user (removed, changed) leads; additions follow.
        self.assertEqual(api.render_drift("v1", "v2", self.OLD, self.NEW), "\n".join([
            "PUBLIC API, what a user of the crate can name: 1 added, 1 removed, 1 changed",
            "  - function  demo::api::gone ()",
            "  ~ function  demo::api::make",
            "        was  pub (x: u8)",
            "        now  pub (x: u16)",
            "  + function  demo::api::fresh ()",
            "",
            "CRATE-PRIVATE, by module: 2 added, 1 removed, 1 changed",
            "    demo::hidden  +2 -1 ~1",
            "  every line: scripts/api_snapshot.py --drift v1 v2 --private",
            "",
        ]))

    def test_private_drift_is_listed_on_request(self) -> None:
        text = api.render_drift("v1", "v2", self.OLD, self.NEW, private=True)
        self.assertIn("  + constant  demo::hidden::deep::LIMIT: u8\n", text)
        self.assertIn("  - function  demo::hidden::dropped ()\n", text)
        self.assertIn("  ~ function  demo::hidden::helper\n", text)
        self.assertLess(text.index("PUBLIC API"), text.index("CRATE-PRIVATE"))
        self.assertNotIn("every line:", text)

    def test_an_item_that_becomes_public_is_public_drift(self) -> None:
        # Its own line can stay the same: a `pub fn` whose module was opened.
        old = [entry("internal", "function", "pub", "demo::f", " ()")]
        new = [entry("api", "function", "pub", "demo::f", " ()")]
        text = api.render_drift("v1", "v2", old, new)
        self.assertIn(
            "  ~ function  demo::f\n        was  internal: pub ()\n        now  api: pub ()\n", text
        )
        self.assertIn("CRATE-PRIVATE, by module: 0 added, 0 removed, 0 changed\n", text)

    def test_an_item_listed_twice_is_added_and_removed_not_changed(self) -> None:
        # Two cfg variants of one function share a kind and a path.
        old = [entry("api", "function", "pub", "demo::f", " ()"),
               entry("api", "function", "pub", "demo::f", " (x: u8)")]
        new = [entry("api", "function", "pub", "demo::f", " ()")]
        self.assertEqual(api.drift(old, new), [(old[1], None)])

    def test_the_trait_impls_a_type_gains_are_one_line(self) -> None:
        # A new struct with eight derives was nine lines of the 0.11.0 drift.
        new = [
            entry("api", "struct", "pub", "demo::W"),
            entry("api", "impl", "", "demo::W::(impl Clone)"),
            entry("api", "impl", "", "demo::W::(impl Debug)"),
            entry("api", "impl", "", "demo::W::(impl From<u8>)"),
            entry("api", "struct_field", "pub", "demo::W::size", ": u8"),
        ]
        old = [entry("api", "impl", "", "demo::V::(impl Copy)")]
        self.assertEqual(api.render_drift("v1", "v2", old, new).splitlines()[:5], [
            "PUBLIC API, what a user of the crate can name: 5 added, 1 removed, 0 changed",
            "  - impl  demo::V: Copy",
            "  + struct  demo::W",
            "  + impl  demo::W: Clone, Debug, From<u8>",
            "  + struct_field  demo::W::size: u8",
        ])

    def test_no_drift_says_so(self) -> None:
        text = api.render_drift("v1", "v2", self.OLD, self.OLD)
        self.assertIn("PUBLIC API, what a user of the crate can name: 0 added, 0 removed, 0 changed\n", text)
        self.assertNotIn("every line:", text)


class CommittedSnapshots(unittest.TestCase):
    """The two newest release snapshots under docs/api/, as step 6 reads them."""

    PREV, NEW = "v0.11.0", "v0.11.2"

    @staticmethod
    def drift(*extra: str) -> str:
        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            api.main(["--drift", CommittedSnapshots.PREV, CommittedSnapshots.NEW, *extra])
        return out.getvalue()

    @classmethod
    def setUpClass(cls) -> None:
        cls.report = cls.drift()
        cls.every_line = cls.drift("--private")

    def test_the_drift_between_two_releases_is_short_and_free_of_noise(self) -> None:
        self.assertTrue(self.report.startswith("PUBLIC API"), self.report[:80])
        self.assertLess(len(self.report.splitlines()), 120)
        for noise in ("impl Any", "Borrow", "CloneToUninit", "Freeze", "Into<U>", "TryFrom<U>",
                      "RefUnwindSafe", "impl Same", "ToOwned", "Unpin", "Equivalent<K>"):
            self.assertNotIn(noise, self.every_line, noise)

    def test_crate_private_items_stay_out_of_the_public_section(self) -> None:
        public = self.report[:self.report.index("CRATE-PRIVATE")]
        self.assertIn("  + constant  jubarte::comparer::WORD_LEVEL_KEPT_RATIO: f64\n", public)
        self.assertNotIn("pub(crate", public)
        # `pub` fields of a struct in a crate-private module are not public.
        self.assertNotIn("jubarte::convert::pdf::", public)
        self.assertIn("  + struct_field  jubarte::convert::pdf::CommentTint::h: f32\n", self.every_line)

    def test_the_drift_is_the_same_on_every_run(self) -> None:
        self.assertEqual(self.drift("--private"), self.every_line)

    def test_both_sides_are_rebuilt_from_the_rustdoc_json(self) -> None:
        # v0.11.2's committed .api.txt is in the old three-column format: a
        # diff of the two text files would be a wall of removals once.
        with tempfile.TemporaryDirectory() as tmp:
            for label in (self.PREV, self.NEW):
                shutil.copy(str(api.API_DIR / f"jubarte-{label}.json.gz"), tmp)
            with mock.patch.object(api, "API_DIR", Path(tmp)):
                self.assertEqual(self.drift(), self.report)

    def test_a_missing_snapshot_is_named(self) -> None:
        with self.assertRaises(SystemExit) as raised:
            api.main(["--drift", "v0.0.0", self.NEW])
        self.assertIn("jubarte-v0.0.0.json.gz", str(raised.exception))


if __name__ == "__main__":
    unittest.main()

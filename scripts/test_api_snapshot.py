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


def _doc(items: list, paths: dict) -> dict:
    return {
        "root": items[0]["id"],
        "index": {str(i["id"]): i for i in items},
        "paths": {str(k): {"crate_id": 0, "path": v, "kind": "x"} for k, v in paths.items()},
    }


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


class DocBuild(unittest.TestCase):
    """mbx 1.24 refuses `cargo doc --output-format json` before cargo runs
    ("metadata probing failed"), so step 6 died on any machine with a
    nightly: the rustdoc flags go through RUSTDOCFLAGS, never cargo's
    command line (#1769)."""

    def runs(self, toolchains: str) -> list[tuple[list[str], dict]]:
        calls = []

        def run(argv, **kw):
            calls.append((list(argv), kw.get("env") or {}))
            return mock.Mock(stdout=toolchains, returncode=0)

        with mock.patch.object(api.subprocess, "run", side_effect=run):
            api.build_doc_json()
        return [(argv, env) for argv, env in calls if argv[0] == "cargo"]

    def test_cargo_never_sees_a_rustdoc_flag(self) -> None:
        for toolchains in ("stable-aarch64-apple-darwin (default)\nnightly-aarch64-apple-darwin\n",
                           "stable-aarch64-apple-darwin (default)\n"):
            (argv, env), = self.runs(toolchains)
            for flag in ("--output-format", "-Z", "unstable-options", "--document-private-items"):
                self.assertNotIn(flag, argv, toolchains)
            self.assertIn("--output-format json", env["RUSTDOCFLAGS"])
            self.assertIn("--document-private-items", env["RUSTDOCFLAGS"])

    def test_a_nightly_documents_with_nightly_rustdoc(self) -> None:
        (argv, env), = self.runs("nightly-aarch64-apple-darwin\n")
        self.assertEqual(argv[:2], ["cargo", "+nightly"])
        self.assertNotIn("RUSTC_BOOTSTRAP", env)

    def test_without_a_nightly_the_pinned_stable_is_unlocked(self) -> None:
        (argv, env), = self.runs("stable-aarch64-apple-darwin (default)\n")
        self.assertNotIn("+nightly", argv)
        self.assertEqual(env["RUSTC_BOOTSTRAP"], "1")


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


def _run_added(old: Path, new: Path) -> tuple[int, str, str]:
    out, err = io.StringIO(), io.StringIO()
    code = 0
    try:
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            api.main(["--added", str(old), str(new)])
    except SystemExit as exc:
        code = exc.code if isinstance(exc.code, int) else 1
    return code, out.getvalue(), err.getvalue()


class AddedPaths(unittest.TestCase):
    """#1758: public item paths present in NEW and absent in OLD. A method
    body is not a path, and neither is a crate-private item."""

    KEEP = "api\tfunction\tpub\tjubarte::keep ()\n"
    ADDED = KEEP + "api\tfunction\tpub\tjubarte::something ()\n"

    def test_a_new_public_free_function_is_named_and_refused(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            old, new = Path(tmp) / "old.api.txt", Path(tmp) / "new.api.txt"
            old.write_text(self.KEEP)
            new.write_text(self.ADDED + "internal\tfunction\tpub(crate)\tjubarte::secret ()\n")
            code, out, err = _run_added(old, new)
        self.assertEqual((code, err), (1, ""), (out, err))
        self.assertEqual(out, "value jubarte::something\n")

    def test_equal_listings_pass(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "same.api.txt"
            path.write_text(self.KEEP)
            code, out, err = _run_added(path, path)
        self.assertEqual((code, out, err), (0, "", ""))

    def test_a_changed_signature_is_the_same_path(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            old, new = Path(tmp) / "old.api.txt", Path(tmp) / "new.api.txt"
            old.write_text(self.KEEP)
            new.write_text("api\tfunction\tpub\tjubarte::keep (x: u8)\n")
            code, out, _err = _run_added(old, new)
        self.assertEqual((code, out), (0, ""))

    def test_listing_paths_match_the_rustdoc_public_surface(self) -> None:
        # `(impl Clone)` contains a space. A parser that stops at the first
        # space reports that path as added when the other side is rustdoc JSON.
        with tempfile.TemporaryDirectory() as tmp:
            listing, doc = Path(tmp) / "demo.api.txt", Path(tmp) / "demo.json"
            listing.write_text("\n".join(api.flatten(demo_doc())) + "\n")
            doc.write_text(json.dumps(demo_doc()))
            self.assertEqual(_run_added(listing, doc)[:2], (0, ""))
            self.assertEqual(_run_added(doc, listing)[:2], (0, ""))

    def _line_spacing(self, impls: list[tuple[int, dict]]) -> dict:
        items = [
            item(0, "jubarte", "module", {"is_crate": True, "items": [1]}),
            item(1, "edit", "module", {"items": [2]}),
            item(2, "LineSpacing", "struct", struct([], [i for i, _ in impls])),
        ]
        for iid, inner in impls:
            items.append(item(iid, None, "impl", inner, vis="default"))
        return _doc(items, {
            0: ["jubarte"], 1: ["jubarte", "edit"], 2: ["jubarte", "edit", "LineSpacing"],
        })

    def test_renaming_an_impl_parameter_is_not_an_added_path(self) -> None:
        # #1758: `impl<'de> Deserialize<'de>` and `impl<'input> Deserialize<'input>`
        # are the same impl. So are `From<T>` and `From<U>`. A new trait impl
        # is still an addition. Parameter names are canonicalized positionally
        # (`'0`, `T0`); trait impls stay in the gate because cargo-semver-checks
        # does not fail a patch for adding one.
        def de(name: str) -> dict:
            return {
                "trait": {"path": "Deserialize", "id": 900, "args": {
                    "angle_bracketed": {"args": [{"lifetime": name}], "constraints": []},
                }},
                "for": {"resolved_path": {"path": "LineSpacing", "id": 2, "args": None}},
                "items": [],
                "generics": {"params": [{"name": name, "kind": {"lifetime": {"outlives": []}}}],
                             "where_predicates": []},
                "blanket_impl": None, "is_synthetic": False,
            }

        def from_type(name: str) -> dict:
            return {
                "trait": {"path": "From", "id": 901, "args": {
                    "angle_bracketed": {"args": [{"type": {"generic": name}}], "constraints": []},
                }},
                "for": {"resolved_path": {"path": "LineSpacing", "id": 2, "args": None}},
                "items": [],
                "generics": {"params": [{"name": name, "kind": {
                    "type": {"bounds": [], "default": None, "is_synthetic": False},
                }}], "where_predicates": []},
                "blanket_impl": None, "is_synthetic": False,
            }

        def serialize() -> dict:
            return {
                "trait": {"path": "Serialize", "id": 902, "args": None},
                "for": {"resolved_path": {"path": "LineSpacing", "id": 2, "args": None}},
                "items": [],
                "generics": {"params": [], "where_predicates": []},
                "blanket_impl": None, "is_synthetic": False,
            }

        old = self._line_spacing([(3, de("'de")), (4, from_type("T"))])
        renamed = self._line_spacing([(3, de("'input")), (4, from_type("U"))])
        added = self._line_spacing([(3, de("'input")), (4, from_type("U")), (5, serialize())])
        with tempfile.TemporaryDirectory() as tmp:
            old_path, renamed_path, added_path = (Path(tmp) / n for n in ("old.json", "renamed.json", "added.json"))
            old_path.write_text(json.dumps(old))
            renamed_path.write_text(json.dumps(renamed))
            added_path.write_text(json.dumps(added))
            same, same_out, same_err = _run_added(old_path, renamed_path)
            code, out, err = _run_added(old_path, added_path)
        self.assertEqual((same, same_out, same_err), (0, "", ""))
        self.assertEqual(err, "", out)
        self.assertEqual((code, out), (
            1, "impl jubarte::edit::LineSpacing::(impl Serialize)\n",
        ))

    def test_a_new_glob_reexport_adds_the_names_it_makes_public(self) -> None:
        # #1758: `pub use a::*` then `pub use b::*`. Both use-lines are
        # `jubarte::*`; `b::second` was already public at its own path.
        # The glob makes `jubarte::second` nameable. A crate-private item
        # in `b` is not.
        def surface(use_ids: list[int]) -> dict:
            return _doc([
                item(0, "jubarte", "module", {"is_crate": True, "items": [1, 2, *use_ids]}),
                item(1, "a", "module", {"items": [4]}),
                item(2, "b", "module", {"items": [5, 6]}),
                item(3, None, "use", {"source": "a::*", "name": "*", "id": 1, "is_glob": True}),
                item(4, "first", "function", fn()),
                item(5, "second", "function", fn()),
                item(6, "buried", "function", fn(), vis="crate"),
                item(8, None, "use", {"source": "b::*", "name": "*", "id": 2, "is_glob": True}),
            ], {
                0: ["jubarte"], 1: ["jubarte", "a"], 2: ["jubarte", "b"],
                4: ["jubarte", "a", "first"], 5: ["jubarte", "b", "second"],
                6: ["jubarte", "b", "buried"],
            })

        with tempfile.TemporaryDirectory() as tmp:
            old, new = Path(tmp) / "old.json", Path(tmp) / "new.json"
            old.write_text(json.dumps(surface([3])))
            new.write_text(json.dumps(surface([3, 8])))
            code, out, err = _run_added(old, new)
        self.assertEqual(err, "", out)
        self.assertEqual((code, out), (1, "value jubarte::second\n"))

    def test_a_glob_of_an_enum_adds_its_variants(self) -> None:
        # `use` items with is_glob point at a module or an enum. Variants
        # carry no visibility of their own; they are as public as the enum.
        def surface(items_at_root: list[int]) -> dict:
            return _doc([
                item(0, "jubarte", "module", {"is_crate": True, "items": items_at_root}),
                item(1, "E", "enum", {"variants": [2, 3], "impls": []}),
                item(2, "A", "variant", {"kind": "plain"}, vis="default"),
                item(3, "B", "variant", {"kind": "plain"}, vis="default"),
                item(4, None, "use", {"source": "E::*", "name": "*", "id": 1, "is_glob": True}),
            ], {0: ["jubarte"], 1: ["jubarte", "E"]})

        with tempfile.TemporaryDirectory() as tmp:
            old, new = Path(tmp) / "old.json", Path(tmp) / "new.json"
            old.write_text(json.dumps(surface([1])))
            new.write_text(json.dumps(surface([1, 4])))
            code, out, err = _run_added(old, new)
        self.assertEqual(err, "", out)
        self.assertEqual((code, out), (1, "type jubarte::A\ntype jubarte::B\n"))

    def test_a_macro_is_not_the_function_of_the_same_path(self) -> None:
        # #1758: `fn something` and `something!` share a path and not a
        # namespace. cargo-semver-checks 0.51.0 has no macro_added lint, so
        # the gate has to name the macro. Identity is the Rust namespace
        # (type, value, macro) plus the path.
        paths = {0: ["jubarte"], 1: ["jubarte", "something"], 2: ["jubarte", "something"]}
        old = _doc([
            item(0, "jubarte", "module", {"is_crate": True, "items": [1]}),
            item(1, "something", "function", fn()),
        ], paths)
        new = _doc([
            item(0, "jubarte", "module", {"is_crate": True, "items": [1, 2]}),
            item(1, "something", "function", fn()),
            item(2, "something", "macro", "macro_rules! something { () => {}; }"),
        ], paths)
        with tempfile.TemporaryDirectory() as tmp:
            old_path, new_path = Path(tmp) / "old.json", Path(tmp) / "new.json"
            old_path.write_text(json.dumps(old))
            new_path.write_text(json.dumps(new))
            code, out, err = _run_added(old_path, new_path)
        self.assertEqual(err, "", out)
        self.assertEqual((code, out), (1, "macro jubarte::something\n"))

    def test_rustdoc_json_names_an_added_free_function(self) -> None:
        old_doc = demo_doc()
        new_doc = demo_doc()
        new_doc["index"]["1"]["inner"]["module"]["items"].append(21)
        new_doc["index"]["21"] = item(21, "something", "function", fn())
        new_doc["paths"]["21"] = {
            "crate_id": 0, "path": ["demo", "api", "something"], "kind": "function",
        }
        with tempfile.TemporaryDirectory() as tmp:
            old, new = Path(tmp) / "old.json", Path(tmp) / "new.json.gz"
            old.write_text(json.dumps(old_doc))
            with gzip.open(new, "wt") as gz:
                json.dump(new_doc, gz)
            code, out, err = _run_added(old, new)
        self.assertEqual((code, err), (1, ""), out)
        self.assertEqual(out, "value demo::api::something\n")

    def test_the_shapes_a_listing_line_can_take(self) -> None:
        lines = [
            "api\tfunction\tpub\tjubarte::admission::AdmissionError::code const (self: &Self) -> &'static str",
            "api\timpl\t\tjubarte::admission::AdmissionError::(impl Clone)",
            "api\ttype_alias\tpub\tjubarte::audit::LayoutPass = &'a mut dyn FnMut(&[u8])",
            "api\tassoc_type\tpub\tjubarte::cli::Cli::(impl Deref)::Target = CompareArgs",
            "api\tstruct_field\tpub\tjubarte::append::AppendError::Admission::document: &'static str",
            "api\tuse\tpub\tjubarte::WmlDocument = wml_document::WmlDocument",
            "internal\tfunction\tpub(crate)\tjubarte::secret ()",
        ]
        with tempfile.TemporaryDirectory() as tmp:
            old, new = Path(tmp) / "old.api.txt", Path(tmp) / "new.api.txt"
            old.write_text(lines[0] + "\n")
            new.write_text("\n".join(lines) + "\n")
            code, out, err = _run_added(old, new)
        self.assertEqual(err, "")
        self.assertEqual(code, 1)
        self.assertEqual(out.splitlines(), [
            "field jubarte::append::AppendError::Admission::document",
            "impl jubarte::admission::AdmissionError::(impl Clone)",
            "type jubarte::audit::LayoutPass",
            "type jubarte::cli::Cli::(impl Deref)::Target",
            "use jubarte::WmlDocument",
        ])

    def test_emit_writes_a_listing_outside_docs_api(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            built = Path(tmp) / "jubarte.json"
            built.write_text(json.dumps(demo_doc()))
            out = Path(tmp) / "head.api.txt"
            with mock.patch.object(api, "build_doc_json"):
                with mock.patch.object(api, "DOC_JSON", built):
                    with mock.patch.object(api, "API_DIR", Path(tmp) / "api"):
                        with contextlib.redirect_stdout(io.StringIO()):
                            api.main(["--emit", str(out)])
            self.assertEqual(out.read_text().splitlines(), api.flatten(demo_doc()))
            self.assertFalse((Path(tmp) / "api").exists())


if __name__ == "__main__":
    unittest.main()

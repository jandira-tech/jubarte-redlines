#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Integration checks for the temporary zip 9 oracle dependency patch."""

import re
import tomllib
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FORK = "https://github.com/jandira-tech/ooxmlsdk"
REV = "d7c3ae016da8ef661d9a04e12c47003d8fe51eff"


class OracleDependency(unittest.TestCase):
    def test_patch_uses_pinned_fork(self) -> None:
        manifest = tomllib.loads((ROOT / "Cargo.toml").read_text())
        self.assertEqual(
            manifest["patch"]["crates-io"]["ooxmlsdk"],
            {"git": FORK, "rev": REV},
        )
        self.assertEqual(manifest["dev-dependencies"]["ooxmlsdk"]["version"], "0.13.2")

    def test_oracle_and_derive_resolve_from_same_revision(self) -> None:
        lock = tomllib.loads((ROOT / "Cargo.lock").read_text())
        packages = {p["name"]: p for p in lock["package"]}
        for name in ("ooxmlsdk", "ooxmlsdk-derive"):
            with self.subTest(package=name):
                self.assertEqual(packages[name].get("source"), f"git+{FORK}?rev={REV}#{REV}")
        for filename in (
            "Cargo.lock",
            "jubarte-python/Cargo.lock",
            "jubarte-wasm/Cargo.lock",
            "jubarte-rust-inproc/Cargo.lock",
        ):
            with self.subTest(lock=filename):
                binding_lock = tomllib.loads((ROOT / filename).read_text())
                self.assertEqual(
                    {p["version"] for p in binding_lock["package"] if p["name"] == "zip"},
                    {"9.0.5"},
                )

    def test_fork_is_explicitly_allowed(self) -> None:
        policy = tomllib.loads((ROOT / "deny.toml").read_text())
        self.assertIn(FORK, policy["sources"].get("allow-git", []))

    def test_vendored_oracle_is_removed(self) -> None:
        self.assertFalse((ROOT / "vendor" / "ooxmlsdk").exists())
        self.assertNotIn("vendor/ooxmlsdk", (ROOT / "Cargo.toml").read_text())
        self.assertNotIn("vendor/", (ROOT / "REUSE.toml").read_text())


class SecurityAdvisoryRecord(unittest.TestCase):
    """SECURITY.md must name every dependency advisory this tree has answered.

    The daily cargo-deny watch reads only RustSec, so a repository advisory
    (GHSA-m665-v7pm-7grj lives on zip-rs/zip2 alone) never turns it red. This
    check fails while the advisory is missing from SECURITY.md, whatever the
    watch says.
    """

    def test_zip_cloaking_advisory_is_recorded(self) -> None:
        security = (ROOT / "SECURITY.md").read_text()
        self.assertIn("GHSA-m665-v7pm-7grj", security)
        self.assertIn("9.0.5", security)

    def test_graph_drift_gate_can_fail_the_build(self) -> None:
        script = (ROOT / "scripts/security-check.sh").read_text()
        self.assertIn("check advisories bans", script)
        policy = tomllib.loads((ROOT / "deny.toml").read_text())
        self.assertEqual(policy["bans"]["multiple-versions"], "deny")

    def test_graph_covers_the_wasm_target(self) -> None:
        # A duplicate that only the browser build pulls in must reach the
        # bans check too.
        policy = tomllib.loads((ROOT / "deny.toml").read_text())
        triples = {target["triple"] for target in policy["graph"]["targets"]}
        self.assertIn("wasm32-unknown-unknown", triples)

    def test_advisory_inventory_records_range_and_withdrawal(self) -> None:
        security = (ROOT / "SECURITY.md").read_text()
        self.assertIn("RUSTSEC-2025-0168", security)
        self.assertIn(">=1.3, <2.3", security)
        self.assertIn("RUSTSEC-2022-0048", security)
        self.assertIn("withdrawn", security)

    def test_deny_ignores_name_the_locked_crate_that_reaches_them(self) -> None:
        policy = tomllib.loads((ROOT / "deny.toml").read_text())
        security = (ROOT / "SECURITY.md").read_text()
        self.assertGreaterEqual(len(policy["advisories"]["ignore"]), 1)
        self.assertEqual(
            advisory_inventory_problems(policy, security, audited_locked_crates(), advisory_db_crate),
            [],
        )

    def test_inventory_refuses_a_crate_the_advisory_is_not_on(self) -> None:
        policy = {"advisories": {"ignore": [{"id": "RUSTSEC-1-1", "reason": "serde is locked"}]}}
        security = "**RUSTSEC-1-1** — `serde` is a direct dependency.\n"
        problems = advisory_inventory_problems(policy, security, {"serde", "zip"}, lambda _: "zip")
        self.assertTrue(any("zip" in problem for problem in problems), problems)

    def test_inventory_accepts_a_crate_only_a_binding_locks(self) -> None:
        policy = {"advisories": {"ignore": [{"id": "RUSTSEC-1-2", "reason": "pyo3 in the binding"}]}}
        security = "**RUSTSEC-1-2** — `pyo3` is locked by jubarte-python.\n"
        self.assertEqual(advisory_inventory_problems(policy, security, {"pyo3"}, lambda _: None), [])

    def test_inventory_needs_every_named_crate_in_the_paragraph(self) -> None:
        policy = {"advisories": {"ignore": [{"id": "RUSTSEC-1-3", "reason": "ttf-parser direct and via rustybuzz"}]}}
        security = "**RUSTSEC-1-3** — `ttf-parser` is a direct dependency.\n\nrustybuzz elsewhere.\n"
        problems = advisory_inventory_problems(policy, security, {"ttf-parser", "rustybuzz"}, lambda _: None)
        self.assertTrue(any("rustybuzz" in problem for problem in problems), problems)


def audited_manifests() -> list[str]:
    """The manifests scripts/security-check.sh runs cargo-deny on."""
    script = (ROOT / "scripts/security-check.sh").read_text()
    match = re.search(r"for manifest in ([^;]+);", script)
    if match is None:
        raise AssertionError("security-check.sh no longer lists its manifests")
    return match.group(1).split()


def audited_locked_crates() -> set[str]:
    """Every crate name locked by a workspace the security check audits."""
    names: set[str] = set()
    for manifest in audited_manifests():
        lock = ROOT / Path(manifest).with_name("Cargo.lock")
        if not lock.exists():
            continue
        names |= {package["name"] for package in tomllib.loads(lock.read_text())["package"]}
    return names


def advisory_db_crate(advisory: str) -> str | None:
    """The crate a RustSec advisory is filed under, from cargo-deny's local
    database, or None where no database has been fetched."""
    for db in (Path.home() / ".cargo" / "advisory-dbs").glob("*/crates"):
        for path in db.glob(f"*/{advisory}.md"):
            return path.parent.name
    return None


def advisory_inventory_problems(policy, security, locked, crate_of) -> list[str]:
    """Why an ignored advisory is not mapped to the crate that reaches it.

    Each ignore's reason names locked crates; the advisory's paragraph in
    SECURITY.md (from its id to the next blank line) names every one of
    them; and where the advisory database is at hand, the crate it is filed
    under is one of them.
    """
    problems = []
    for item in policy["advisories"]["ignore"]:
        advisory = item["id"]
        reached = sorted(
            name for name in locked if re.search(rf"(?<![\w-]){re.escape(name)}(?![\w-])", item["reason"])
        )
        if not reached:
            problems.append(f"{advisory} reason names no locked crate")
            continue
        at = security.find(advisory)
        if at < 0:
            problems.append(f"{advisory} is not in SECURITY.md")
            continue
        paragraph = security[at:].split("\n\n", 1)[0]
        problems.extend(f"{advisory} paragraph lacks {crate}" for crate in reached if crate not in paragraph)
        filed = crate_of(advisory)
        if filed is not None and filed not in reached:
            problems.append(f"{advisory} is filed under {filed}, which the ignore does not name")
    return problems


if __name__ == "__main__":
    unittest.main()

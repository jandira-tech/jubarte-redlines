# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
"""Tests for split_equivalence.py, the T7 move-only gate (plan AC2)."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import split_equivalence as se

OLD = """\
//! The module.
use crate::x::Y;

// ---- section banner ----

/// Adds.
pub fn add(a: u32, b: u32) -> u32 {
    a + b // keep
}

fn helper(v: &[u32]) -> (u32,) {
    (v.iter().sum(),)
}

const LIMIT: usize = 3;

impl Thing {
    fn go(&self) {}
}

static DATA: &[u8] = include_bytes!("../tests/data/a.bin");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds() {
        assert_eq!(add(1, 2), 3);
    }
}
"""

# The same items, moved, re-indented, reflowed by rustfmt, narrowed to
# pub(super), with a new SPDX header, new uses and a new mod layout.
NEW_MOD = """\
// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only
//! The module.
mod math;
use math::*;

/// Adds.
pub fn add(
    a: u32,
    b: u32,
) -> u32 {
    a + b // keep
}

impl Thing {
    fn go(&self) {}
}

#[cfg(test)]
#[path = "tests/tests.rs"]
mod tests;
"""

NEW_MATH = """\
// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only
use super::*;

pub(super) fn helper(v: &[u32]) -> (u32,) {
    (v.iter().sum(),)
}

pub(super) const LIMIT: usize = 3;

pub(super) static DATA: &[u8] = include_bytes!("../../tests/data/a.bin");
"""

NEW_TESTS = """\
use super::*;

#[test]
fn adds() {
    assert_eq!(add(1, 2), 3,);
}
"""


def tree(**files: str) -> dict[str, str]:
    return dict(files)


OLD_TREE = tree(**{"m.rs": OLD})
NEW_TREE = tree(**{"m/mod.rs": NEW_MOD, "m/math.rs": NEW_MATH, "m/tests/tests.rs": NEW_TESTS})


class PureMove(unittest.TestCase):
    def test_a_pure_move_has_no_residue(self) -> None:
        r = se.compare(OLD_TREE, NEW_TREE)
        self.assertEqual(r.residue, [], se.render(r))
        self.assertEqual(r.matched, 6)

    def test_visibility_only_changes_are_counted_not_failed(self) -> None:
        r = se.compare(OLD_TREE, NEW_TREE)
        self.assertEqual(r.visibility_changed, 3)


class Residue(unittest.TestCase):
    def residue_of(self, path: str, old: str, new: str) -> list[se.Unmatched]:
        changed = dict(NEW_TREE)
        changed[path] = changed[path].replace(old, new)
        self.assertNotEqual(changed[path], NEW_TREE[path], "the probe edit did not apply")
        return se.compare(OLD_TREE, changed).residue

    def test_a_changed_body_is_residue(self) -> None:
        res = self.residue_of("m/math.rs", "(v.iter().sum(),)", "(v.iter().sum::<u32>() + 1,)")
        self.assertEqual(sorted(u.side for u in res), ["new", "old"])
        self.assertTrue(all(u.key == "fn helper" for u in res))

    def test_dropping_a_one_tuple_comma_is_residue(self) -> None:
        res = self.residue_of("m/math.rs", "(v.iter().sum(),)\n", "(v.iter().sum())\n")
        self.assertEqual({u.key for u in res}, {"fn helper"})

    def test_a_changed_doc_comment_is_residue(self) -> None:
        res = self.residue_of("m/mod.rs", "/// Adds.", "/// Sums.")
        self.assertEqual({u.key for u in res}, {"fn add"})

    def test_a_changed_body_comment_is_residue(self) -> None:
        res = self.residue_of("m/mod.rs", "// keep", "// kept")
        self.assertEqual({u.key for u in res}, {"fn add"})

    def test_a_lost_test_is_residue(self) -> None:
        res = self.residue_of("m/tests/tests.rs", "#[test]\nfn adds() {\n    assert_eq!(add(1, 2), 3,);\n}\n", "")
        self.assertEqual([(u.side, u.key) for u in res], [("old", "fn adds")])

    def test_a_duplicated_item_is_residue(self) -> None:
        res = self.residue_of("m/math.rs", "pub(super) const LIMIT", "const LIMIT: usize = 3;\npub(super) const LIMIT")
        self.assertEqual([(u.side, u.key) for u in res], [("new", "const LIMIT")])

    def test_an_include_path_to_another_file_is_residue(self) -> None:
        res = self.residue_of("m/math.rs", "../../tests/data/a.bin", "../tests/data/a.bin")
        self.assertEqual({u.key for u in res}, {"static DATA"})

    def test_a_reindented_multiline_string_is_residue(self) -> None:
        old = {"m.rs": 'mod t {\n    fn s() -> &\'static str {\n        "a\n        b"\n    }\n}\n'}
        new = {"m/mod.rs": "mod t;\n", "m/t.rs": 'fn s() -> &\'static str {\n    "a\n    b"\n}\n'}
        res = se.compare(old, new).residue
        self.assertEqual({u.key for u in res}, {"fn s"})

    def test_a_changed_attribute_is_residue(self) -> None:
        res = self.residue_of("m/tests/tests.rs", "#[test]", "#[test]\n#[ignore]")
        self.assertEqual({u.key for u in res}, {"fn adds"})


def one(old_body: str, new_body: str) -> se.Result:
    return se.compare({"m.rs": f"fn f() {{\n{old_body}\n}}\n"}, {"m/a.rs": f"fn f() {{\n{new_body}\n}}\n"})


class RustfmtReflow(unittest.TestCase):
    """The rewrites rustfmt makes when a moved test loses an indent level."""

    def test_a_closure_loses_the_braces_of_its_one_expression_body(self) -> None:
        self.assertEqual(one("let h = |t: &str| { g(t).h() };", "let h = |t: &str| g(t).h();").residue, [])

    def test_a_match_arm_loses_its_braces_and_gains_a_comma(self) -> None:
        old = 'let v = match t { "a" => { "x" } "b" => "y", _ => { "z" } };'
        new = 'let v = match t { "a" => "x", "b" => "y", _ => "z" };'
        self.assertEqual(one(old, new).residue, [])

    def test_a_one_argument_call_and_parameter_list_drop_their_comma(self) -> None:
        self.assertEqual(one("v.push(S { a: 1 },); g(x,);", "v.push(S { a: 1 }); g(x);").residue, [])
        sig = se.compare({"m.rs": "fn f(a: &S,) -> u8 { 0 }"}, {"m/a.rs": "fn f(a: &S) -> u8 { 0 }"})
        self.assertEqual(sig.residue, [])

    def test_a_one_element_tuple_keeps_its_comma(self) -> None:
        self.assertNotEqual(one("let t = (a,);", "let t = (a);").residue, [])
        self.assertNotEqual(one("let (a,) = t;", "let (a) = t;").residue, [])
        ret = se.compare({"m.rs": "fn f() -> (u8,) { (0,) }"}, {"m/a.rs": "fn f() -> (u8) { (0,) }"})
        self.assertNotEqual(ret.residue, [])
        arm = one("match t { _ => (a,) }", "match t { _ => (a) }")
        self.assertNotEqual(arm.residue, [])

    def test_a_block_with_a_statement_stays_strict(self) -> None:
        self.assertNotEqual(one("let h = |t| { g(t); h(t) };", "let h = |t| h(t);").residue, [])
        self.assertNotEqual(one("let h = |t| { let u = t; u };", "let h = |t| t;").residue, [])

    def test_a_changed_closure_body_is_residue(self) -> None:
        self.assertNotEqual(one("let h = |t| { g(t) };", "let h = |t| g(u);").residue, [])


class Lexing(unittest.TestCase):
    def test_raw_strings_lifetimes_and_chars(self) -> None:
        toks = se.tokens("fn f<'a>(x: &'a str) -> char { let _ = r#\"}\"#; '}' }")
        self.assertIn('r#"}"#', toks)
        self.assertIn("'}'", toks)
        self.assertIn("'a", toks)
        self.assertEqual(toks[-1], "}")

    def test_block_comments_nest(self) -> None:
        toks = se.tokens("/* a /* b */ c */ fn f() {}")
        self.assertEqual(toks[0], "/* a /* b */ c */")

    def test_byte_and_c_strings(self) -> None:
        toks = se.tokens('const A: &[u8] = b"x\\"y"; const B: u8 = b\'}\';')
        self.assertIn('b"x\\"y"', toks)
        self.assertIn("b'}'", toks)


class Items(unittest.TestCase):
    def test_a_struct_literal_const_ends_at_its_semicolon(self) -> None:
        items = se.items_of(se.tokens("const C: S = S { a: 1 };\nfn f() {}\n"), "x.rs")
        self.assertEqual([i.key for i in items], ["const C", "fn f"])

    def test_macro_items_and_impls_are_keyed(self) -> None:
        src = "macro_rules! m { () => {}; }\nimpl<T> Tr for S<T> { fn g() {} }\nthread_local! { static X: u8 = 0; }\n"
        keys = [i.key for i in se.items_of(se.tokens(src), "x.rs")]
        self.assertEqual(keys, ["macro_rules m", "impl < T > Tr for S < T >", "macro thread_local"])


def residue(old: dict[str, str], new: dict[str, str]) -> list[se.Unmatched]:
    return se.compare(old, new).residue


class ReviewFindings(unittest.TestCase):
    """False negatives the MiniMax-M3 review of #2457 found, each now residue."""

    def test_pub_inside_a_macro_matcher_is_compared(self) -> None:
        old = "macro_rules! kw {\n    (pub $x:ident) => { 1 };\n    ($x:ident) => { 2 };\n}\n"
        new = "macro_rules! kw {\n    ($x:ident) => { 1 };\n    (pub $x:ident) => { 2 };\n}\n"
        self.assertNotEqual(residue({"m.rs": old}, {"m/a.rs": new}), [])

    def test_a_user_macro_invocation_keeps_its_trailing_comma(self) -> None:
        self.assertNotEqual(one("let v = choose!(1,);", "let v = choose!(1);").residue, [])
        old = "macro_rules! exact { [foo,] => { 1 } }\n"
        new = "macro_rules! exact { [foo] => { 1 } }\n"
        self.assertNotEqual(residue({"m.rs": old}, {"m/a.rs": new}), [])

    def test_a_std_expression_macro_may_reflow_its_trailing_comma(self) -> None:
        self.assertEqual(one("assert_eq!(a, b,);", "assert_eq!(a, b);").residue, [])
        self.assertEqual(one('panic!("{}", x,);', 'panic!("{}", x);').residue, [])

    def test_closure_braces_inside_a_user_macro_are_compared(self) -> None:
        self.assertNotEqual(one("let v = choose!(|x| { foo });", "let v = choose!(|x| foo);").residue, [])

    def test_closure_braces_inside_a_std_macro_may_reflow(self) -> None:
        self.assertEqual(
            one("assert!(v.iter().any(|g| { g.ok() }));", "assert!(v.iter().any(|g| g.ok()));").residue, []
        )

    def test_a_one_element_tuple_after_any_keyword_keeps_its_comma(self) -> None:
        self.assertNotEqual(one("for (x,) in xs { g(x); }", "for (x) in xs { g(x); }").residue, [])
        self.assertNotEqual(one("let v = t as (u8,);", "let v = t as (u8);").residue, [])
        wh_old = {"m.rs": "fn s() -> u8 where (u8,): W { 0 }\n"}
        wh_new = {"m/a.rs": "fn s() -> u8 where (u8): W { 0 }\n"}
        self.assertNotEqual(residue(wh_old, wh_new), [])

    def test_a_comparison_against_a_one_element_tuple_keeps_its_comma(self) -> None:
        self.assertNotEqual(one("let c = a > (b,);", "let c = a > (b);").residue, [])

    def test_generic_fn_parameters_and_turbofish_calls_may_reflow(self) -> None:
        self.assertEqual(one("let v = parse::<u8>(s,);", "let v = parse::<u8>(s);").residue, [])
        old = {"m.rs": "fn blank<'de, D: Into<u8>>(de: D,) -> u8 { 0 }\n"}
        new = {"m/a.rs": "fn blank<'de, D: Into<u8>>(de: D) -> u8 { 0 }\n"}
        self.assertEqual(residue(old, new), [])

    def test_a_comparison_after_generics_keeps_its_tuple_comma(self) -> None:
        self.assertNotEqual(one("let c = x < y && z > (w,);", "let c = x < y && z > (w);").residue, [])

    def test_same_named_items_cannot_swap_bodies_across_modules(self) -> None:
        old = {"m/a.rs": "fn value() -> u8 { 1 }\n", "m/b.rs": "fn value() -> u8 { 2 }\n"}
        new = {"m/a.rs": "fn value() -> u8 { 2 }\n", "m/b.rs": "fn value() -> u8 { 1 }\n"}
        self.assertNotEqual(residue(old, new), [])

    def test_same_named_test_helpers_still_match_by_module(self) -> None:
        old = {"m.rs": "mod a_tests {\n    fn exercise() {}\n}\nmod b_tests {\n    fn exercise() { g(); }\n}\n"}
        new = {
            "m/mod.rs": '#[path = "tests/a_tests.rs"]\nmod a_tests;\n#[path = "tests/b_tests.rs"]\nmod b_tests;\n',
            "m/tests/a_tests.rs": "fn exercise() {}\n",
            "m/tests/b_tests.rs": "fn exercise() { g(); }\n",
        }
        self.assertEqual(residue(old, new), [])

    def test_a_renamed_module_is_residue(self) -> None:
        res = residue({"m.rs": "mod old_name {\n    fn f() {}\n}\n"}, {"m.rs": "mod new_name {\n    fn f() {}\n}\n"})
        self.assertEqual([u.key for u in res], ["mod old_name"])

    def test_dropping_cfg_test_from_an_inline_module_is_residue(self) -> None:
        old = {"m.rs": "#[cfg(test)]\nmod tests {\n    fn f() {}\n}\nfn g() {}\n"}
        new = {"m.rs": "mod tests {\n    fn f() {}\n}\nfn g() {}\n"}
        self.assertEqual([u.key for u in residue(old, new)], ["fn f", "fn f"])

    def test_a_cfg_on_a_new_sibling_declaration_is_residue(self) -> None:
        old = {"m.rs": "fn f() {}\n"}
        new = {"m/mod.rs": '#[cfg(feature = "x")]\nmod a;\n', "m/a.rs": "fn f() {}\n"}
        self.assertNotEqual(residue(old, new), [])

    def test_a_test_module_moved_out_of_line_keeps_its_cfg_through_the_declaration(self) -> None:
        old = {"m.rs": "#[cfg(test)]\nmod t {\n    fn f() {}\n}\n"}
        new = {"m/mod.rs": '#[cfg(test)]\n#[path = "tests/t.rs"]\nmod t;\n', "m/tests/t.rs": "fn f() {}\n"}
        self.assertEqual(residue(old, new), [])
        lost = {"m/mod.rs": '#[path = "tests/t.rs"]\nmod t;\n', "m/tests/t.rs": "fn f() {}\n"}
        self.assertNotEqual(residue(old, lost), [])

    def test_a_declaration_inside_a_cfg_inline_module_carries_its_cfg(self) -> None:
        old = {"m.rs": "#[cfg(test)]\nmod t {\n    mod x {\n        fn f() {}\n    }\n}\n"}
        new = {"m/mod.rs": "#[cfg(test)]\nmod t {\n    mod x;\n}\n", "m/t/x.rs": "fn f() {}\n"}
        self.assertEqual(residue(old, new), [])
        lost = {"m/mod.rs": "mod t {\n    mod x;\n}\n", "m/t/x.rs": "fn f() {}\n"}
        self.assertNotEqual(residue(old, lost), [])

    def test_an_inner_cfg_attribute_counts_as_the_module_context(self) -> None:
        old = {"m.rs": "#[cfg(test)]\nmod t {\n    fn f() {}\n}\n"}
        new = {"m/mod.rs": "mod t;\n", "m/t.rs": "#![cfg(test)]\nfn f() {}\n"}
        self.assertEqual(residue(old, new), [])


if __name__ == "__main__":
    unittest.main()

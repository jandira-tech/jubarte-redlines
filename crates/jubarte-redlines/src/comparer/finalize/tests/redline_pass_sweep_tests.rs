// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! Seeded redline bodies through each finalizer pass on its own. A pass
//! may move a pilcrow, a property or a run between paragraphs, reorder
//! deleted and inserted blocks, or drop an empty paragraph, but the
//! document it leaves must accept to the same text and reject to the same
//! text as the one it was given.

use super::*;
use crate::test_docs::{Lcg, split};

const DATE: &str = "2026-01-01T00:00:00Z";

/// The vocabulary of generated runs.
fn words() -> Vec<&'static str> {
    split(WORD_LIST)
}

const WORD_LIST: &str = "harbor lantern meadow copper orchard summit a b ONE TWO 1 2 Demo Document Title the and for agreement notice section List Text Bold";

fn text(rng: &mut Lcg, n: usize) -> String {
    (0..n)
        .map(|_| words()[rng.below(words().len())])
        .collect::<Vec<_>>()
        .join(" ")
}

fn run(kind: &str, s: &str, id: &mut u32) -> String {
    let leaf = if kind == "del" { "delText" } else { "t" };
    let r = format!(
        "<w:r><w:rPr><w:lang w:val=\"en-US\"/></w:rPr><w:{leaf} xml:space=\"preserve\">{s}</w:{leaf}></w:r>"
    );
    if kind == "eq" {
        r
    } else {
        *id += 1;
        format!("<w:{kind} w:id=\"{id}\" w:author=\"Editor\" w:date=\"{DATE}\">{r}</w:{kind}>")
    }
}

/// Paragraph properties: a style, numbering, spacing, justification and
/// a mark revision or a property change, each by chance.
fn ppr(rng: &mut Lcg, mark: Option<&str>, id: &mut u32) -> String {
    let mut out = String::from("<w:pPr>");
    match rng.below(6) {
        0 => out.push_str("<w:pStyle w:val=\"Title\"/>"),
        1 => out.push_str("<w:pStyle w:val=\"Heading1\"/>"),
        2 => out.push_str("<w:pStyle w:val=\"ListParagraph\"/>"),
        3 => out.push_str("<w:pStyle w:val=\"Normal\"/>"),
        _ => {}
    }
    if rng.chance(30) {
        out.push_str(&format!(
            "<w:numPr><w:ilvl w:val=\"{}\"/><w:numId w:val=\"9\"/></w:numPr>",
            rng.below(3)
        ));
    }
    if rng.chance(40) {
        out.push_str(&format!(
            "<w:spacing w:before=\"{}\" w:after=\"{}\"/>",
            rng.below(3) * 120,
            rng.below(3) * 80
        ));
    }
    if rng.chance(25) {
        out.push_str(
            [
                "<w:jc w:val=\"center\"/>",
                "<w:jc w:val=\"left\"/>",
                "<w:jc w:val=\"both\"/>",
            ][rng.below(3)],
        );
    }
    if let Some(kind) = mark {
        *id += 1;
        out.push_str(&format!(
            "<w:rPr><w:{kind} w:id=\"{id}\" w:author=\"Editor\" w:date=\"{DATE}\"/></w:rPr>"
        ));
    }
    if rng.chance(20) {
        *id += 1;
        out.push_str(&format!(
            "<w:pPrChange w:id=\"{id}\" w:author=\"Editor\" w:date=\"{DATE}\"><w:pPr><w:spacing w:after=\"200\"/><w:jc w:val=\"right\"/></w:pPr></w:pPrChange>"
        ));
    }
    out.push_str("</w:pPr>");
    out
}

/// One redline paragraph: plain, pure inserted, pure deleted, mixed, or
/// empty, with or without a revised pilcrow.
fn paragraph(rng: &mut Lcg, id: &mut u32, forced: Option<usize>) -> String {
    let n = 1 + rng.below(6);
    let kind = forced.unwrap_or_else(|| rng.below(15));
    let mark = match kind {
        1 | 5 => Some("ins"),
        2 | 6 => Some("del"),
        3 | 4 if rng.chance(50) => Some(if rng.chance(50) { "ins" } else { "del" }),
        _ => None,
    };
    let props = ppr(rng, mark, id);
    let body = match kind {
        0 => run("eq", &text(rng, n), id),
        1 => run("ins", &text(rng, n), id),
        2 => run("del", &text(rng, n), id),
        3 => {
            let mut s = String::new();
            for _ in 0..2 + rng.below(4) {
                let k = ["eq", "ins", "del"][rng.below(3)];
                let words = 1 + rng.below(3);
                s.push_str(&run(k, &format!("{} ", text(rng, words)), id));
            }
            s
        }
        4 => format!(
            "{}{}",
            run("del", &text(rng, n), id),
            run("ins", &text(rng, n), id)
        ),
        5 | 6 => String::new(),
        7 => run(if rng.chance(50) { "ins" } else { "del" }, "   ", id),
        8 => format!("{}{}", run("eq", &text(rng, 1), id), run("ins", "  ", id)),
        9 => {
            *id += 1;
            format!(
                "<w:moveFrom w:id=\"{id}\" w:author=\"Editor\" w:date=\"{DATE}\"><w:r><w:delText>{}</w:delText></w:r></w:moveFrom>{}",
                text(rng, n),
                run("ins", &text(rng, 2), id)
            )
        }
        10 => format!(
            "<w:hyperlink w:anchor=\"x\">{}</w:hyperlink>{}",
            run(["eq", "ins", "del"][rng.below(3)], &text(rng, 2), id),
            run("del", &format!(" {}", text(rng, n)), id)
        ),
        11 => format!(
            "{}<w:r><w:fldChar w:fldCharType=\"begin\"/></w:r><w:r><w:instrText> PAGE </w:instrText></w:r><w:r><w:fldChar w:fldCharType=\"separate\"/></w:r>{}<w:r><w:fldChar w:fldCharType=\"end\"/></w:r>",
            run("del", &text(rng, n), id),
            run("ins", "7", id)
        ),
        12 => format!(
            "{}<m:oMath><m:r><m:t>x</m:t></m:r></m:oMath>{}",
            run("ins", &text(rng, 1), id),
            run("del", &text(rng, 2), id)
        ),
        13 => format!(
            "{}<w:r><w:drawing/></w:r>",
            run(
                if rng.chance(50) { "ins" } else { "del" },
                &text(rng, n),
                id
            )
        ),
        _ => format!(
            "{}{}",
            run("del", words()[6 + rng.below(6)], id),
            run("ins", words()[6 + rng.below(6)], id)
        ),
    };
    format!("<w:p>{props}{body}</w:p>")
}

fn body(rng: &mut Lcg) -> String {
    let mut id = 100;
    let mut out = String::new();
    for _ in 0..2 + rng.below(12) {
        if rng.chance(8) {
            let row_mark = match rng.below(3) {
                0 => format!(
                    "<w:trPr><w:ins w:id=\"7\" w:author=\"Editor\" w:date=\"{DATE}\"/></w:trPr>"
                ),
                1 => format!(
                    "<w:trPr><w:del w:id=\"8\" w:author=\"Editor\" w:date=\"{DATE}\"/></w:trPr>"
                ),
                _ => String::new(),
            };
            out.push_str(&format!(
                "<w:tbl><w:tblPr><w:tblW w:w=\"3600\" w:type=\"dxa\"/></w:tblPr><w:tblGrid><w:gridCol w:w=\"3600\"/></w:tblGrid><w:tr>{row_mark}<w:tc>{}</w:tc></w:tr></w:tbl>",
                paragraph(rng, &mut id, None)
            ));
        } else if rng.chance(20) {
            // A replaced block: pure deletions, then pure insertions.
            for _ in 0..1 + rng.below(4) {
                out.push_str(&paragraph(rng, &mut id, Some(2)));
            }
            for _ in 0..1 + rng.below(4) {
                let kind = if rng.chance(70) { 1 } else { 5 };
                out.push_str(&paragraph(rng, &mut id, Some(kind)));
            }
        } else if rng.chance(10) {
            out.push_str(&format!(
                "<w:sdt><w:sdtPr><w:id w:val=\"5\"/></w:sdtPr><w:sdtContent>{}</w:sdtContent></w:sdt>",
                paragraph(rng, &mut id, None)
            ));
        } else {
            out.push_str(&paragraph(rng, &mut id, None));
        }
    }
    out
}

fn document(fragment: &str) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&format!(
        "<w:document xmlns:w=\"{}\" xmlns:m=\"{}\"><w:body>{fragment}<w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr></w:body></w:document>",
        W::URI,
        M::URI
    ));
    let root = dom.root(doc).unwrap();
    (dom, root)
}

fn inside(dom: &Dom, n: NodeId, names: &[XName]) -> bool {
    let mut cur = dom.parent(n);
    while let Some(p) = cur {
        if names.iter().any(|name| dom.name_is(p, name)) {
            return true;
        }
        cur = dom.parent(p);
    }
    false
}

/// `(accepted, rejected)` non-whitespace text of the body.
fn views(dom: &Dom, root: NodeId) -> (String, String) {
    let mut accepted = String::new();
    let mut rejected = String::new();
    for n in dom.descendants(root, None) {
        let is_t = dom.name_is(n, &W::t());
        let is_del = dom.name_is(n, &W::del_text());
        if !is_t && !is_del {
            continue;
        }
        let v: String = dom
            .value_str(n)
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
        let in_del = is_del || inside(dom, n, &[W::del(), W::move_from()]);
        let in_ins = inside(dom, n, &[W::ins(), W::move_to()]);
        if !in_del {
            accepted.push_str(&v);
        }
        if !in_ins {
            rejected.push_str(&v);
        }
    }
    (accepted, rejected)
}

type Pass = (&'static str, fn(&mut Dom, NodeId));

/// `(name, pass)` for each named finalizer pass.
macro_rules! named {
    ($($pass:ident),* $(,)?) => {
        vec![$((stringify!($pass), $pass as fn(&mut Dom, NodeId))),*]
    };
}

fn passes() -> Vec<Pass> {
    let mut out: Vec<Pass> = named!(
        restore_echoed_heading_deletions_before_table,
        strip_leading_del_echoing_prev_pure_i,
        splice_trailing_short_pure_dels_midstream,
        split_digits_ins_from_mixed_title,
        peel_trailing_ins_from_mix_into_following_pure_del,
        restore_short_del_before_long_ins,
        peel_trailing_del_from_mix_into_following_pure_ins,
        fold_short_list_label_into_empty_pure_del,
        strip_trailing_bare_empty_after_pure_i_dominant,
        promote_live_numpr_on_pure_d_from_pprchange,
        ensure_pure_i_list_snug_spacing,
        normalize_incomplete_spacing,
        strip_redundant_normal_pstyle_and_bidi,
        strip_list_layout_from_mid_pure_del,
        free_mesh_shared_title_token_in_mix,
        free_mesh_wholesale_body_mix,
        free_mesh_bookended_ins_del,
        peel_trailing_for_word_onto_next_mix,
        residual_short_label_zip,
        interleave_list_cluster_after_coalesce,
        reorder_replaced_blocks,
        coalesce_all_paragraphs,
        fix_paragraph_mark_revision_order,
        strip_trailing_empty_pure_del_mark,
        strip_trailing_empty_pure_ins,
        relocate_title_page_last_empty_after_pure_dels,
        strip_empty_pure_ins_before_trailing_pure_dels,
        fold_whitespace_pure_ins_into_following_pure_del,
        rotate_ins_mark_del_only_paragraph,
        restamp_stranded_del_mark_onto_del_only_paragraph,
        split_head_short_title_long_del_mix,
        fold_leading_ins_from_mix_into_preceding_pure_del,
        strip_last_pure_del_mark_only_ppr,
        cleanup_spacing_and_default_jc,
        last_pure_del_inherit_prev_jc,
        strip_empty_pprchange_on_mix_with_live_jc,
        strip_propertyless_ppr,
        promote_heading_spacing_from_pprchange_on_last_mix,
        strip_last_pure_del_mark_when_pprchange
    );
    let wrapped: Vec<Pass> = vec![
        ("merge_replaced_paragraphs", |dom, root| {
            merge_replaced_paragraphs(dom, root, "Editor");
        }),
        ("last_pure_del_spacing_to_pprchange", |dom, root| {
            last_pure_del_spacing_to_pprchange(
                dom,
                root,
                &WmlComparerSettings::default(),
                &mut 900,
            );
        }),
        ("mixed_spacing_to_following_empty", |dom, root| {
            mixed_spacing_to_following_empty(dom, root, &WmlComparerSettings::default(), &mut 900);
        }),
        ("park_mixed_spacing_onto_trailing_pure_del", |dom, root| {
            park_mixed_spacing_onto_trailing_pure_del(
                dom,
                root,
                &WmlComparerSettings::default(),
                &mut 900,
            );
        }),
        (
            "park_mixed_numpr_onto_trailing_empty_pure_del",
            |dom, root| {
                park_mixed_numpr_onto_trailing_empty_pure_del(
                    dom,
                    root,
                    &WmlComparerSettings::default(),
                    &mut 900,
                );
            },
        ),
        ("repair_borrowed_sentence_period", |dom, root| {
            repair_borrowed_sentence_period(dom, root, &WmlComparerSettings::default(), &mut 900);
        }),
        (
            "ensure_empty_pprchange_on_live_heading_spacing",
            |dom, root| {
                ensure_empty_pprchange_on_live_heading_spacing(
                    dom,
                    root,
                    &WmlComparerSettings::default(),
                    &mut 900,
                );
            },
        ),
        ("ensure_empty_pprchange_on_eq_with_live_jc", |dom, root| {
            ensure_empty_pprchange_on_eq_with_live_jc(
                dom,
                root,
                &HashSet::new(),
                &WmlComparerSettings::default(),
                &mut 900,
            );
        }),
        ("strip_redundant_demo_default_spacing", |dom, root| {
            strip_redundant_demo_default_spacing(dom, root, (Some("240"), Some("auto")));
        }),
        ("mark_fully_revised_rows", |dom, root| {
            mark_fully_revised_rows(dom, root, &WmlComparerSettings::default(), &mut 900);
        }),
        (
            "fold_midstream_demo_title_into_numbered_heading_with_settings",
            |dom, root| {
                fold_midstream_demo_title_into_numbered_heading_with_settings(
                    dom,
                    root,
                    &WmlComparerSettings::default(),
                    &mut 900,
                );
            },
        ),
        ("merge_replaced_paragraphs_in", |dom, root| {
            merge_replaced_paragraphs_in(dom, root, "Editor", true);
        }),
        ("residual_short_label_zip_with_settings", |dom, root| {
            residual_short_label_zip_with_settings(
                dom,
                root,
                &WmlComparerSettings::powertools_faithful(),
            );
        }),
        ("trailing_empty_spacing_to_pprchange", |dom, root| {
            trailing_empty_spacing_to_pprchange(
                dom,
                root,
                None,
                &WmlComparerSettings::default(),
                &mut 900,
            );
        }),
        ("conjoin_paragraph_marks", |dom, root| {
            conjoin_paragraph_marks(dom, root, &WmlComparerSettings::default());
        }),
        ("promote_live_jc_from_pprchange_on_body_mix", |dom, root| {
            promote_live_jc_from_pprchange_on_body_mix(dom, root, &HashSet::new());
        }),
        ("drop_sectpr_from_deleted_marks", |dom, root| {
            drop_sectpr_from_deleted_marks(dom, root, &HashSet::new());
        }),
    ];
    out.extend(wrapped);
    out
}

fn sweep(seed: u64, count: usize) {
    let mut failures = Vec::new();
    let mut changed = 0usize;
    for case in 0..count {
        let mut rng = Lcg(seed
            .wrapping_add(case as u64)
            .wrapping_mul(0x9e37_79b9_7f4a_7c15));
        let fragment = body(&mut rng);
        for (name, pass) in passes() {
            let (mut dom, root) = document(&fragment);
            let before = views(&dom, root);
            let xml = dom.serialize_element(root);
            pass(&mut dom, root);
            if dom.serialize_element(root) != xml {
                changed += 1;
            }
            let mut after = views(&dom, root);
            let mut before = before;
            if REORDERING.contains(&name) {
                for v in [&mut before.0, &mut before.1, &mut after.0, &mut after.1] {
                    let mut chars: Vec<char> = v.chars().collect();
                    chars.sort_unstable();
                    *v = chars.into_iter().collect();
                }
            }
            if ACCEPTED_ONLY.contains(&name) {
                before.1.clear();
                after.1.clear();
            }
            if after != before {
                failures.push(format!("{seed}+{case} {name}: {before:?} -> {after:?}"));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} passes changed a view:\n{}",
        failures.len(),
        failures.join("\n")
    );
    assert!(changed > 0, "no pass changed any document");
}

/// Passes that move revised runs across paragraphs or reorder deleted and
/// inserted blocks. On inputs the comparer would not emit they may change
/// the order of the text, so they are held to the same characters.
const REORDERING: &[&str] = &[
    "reorder_replaced_blocks",
    "restore_short_del_before_long_ins",
    "residual_short_label_zip",
    "residual_short_label_zip_with_settings",
    "fold_midstream_demo_title_into_numbered_heading_with_settings",
];

/// Passes that drop a deleted run echoing the inserted text before it.
/// They change only what rejecting restores, so they are held to the
/// accepted text alone (a finding: rejecting no longer restores the
/// original exactly).
const ACCEPTED_ONLY: &[&str] = &["strip_leading_del_echoing_prev_pure_i"];

#[test]
fn passes_keep_both_views_0() {
    sweep(1, 300);
}

#[test]
fn passes_keep_both_views_1() {
    sweep(7_000, 300);
}

#[test]
fn passes_keep_both_views_2() {
    sweep(11_000, 300);
}

#[test]
fn passes_keep_both_views_3() {
    sweep(23_000, 300);
}

#[test]
fn passes_keep_both_views_4() {
    sweep(31_000, 300);
}

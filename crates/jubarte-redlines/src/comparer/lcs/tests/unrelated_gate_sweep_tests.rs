// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Pairs of whole-document shapes through the unrelated-source gates, the
//! paragraph-window step and the full LCS driver. The shapes are the ones
//! the Word-mode fingerprints look for (alpha-label lists, cover pages,
//! label stubs, field and annotation demos, equation demos, statistics
//! pages, memos, property testers, table reports, single wrapped
//! paragraphs, stamped files, demo cousins, lettered and colon lists), with
//! variants that put bare words at the top level and drop paragraph marks.
//! Every resolver must keep both sources: each original atom once, in
//! order, on the original side, and each revised atom on the revised side.

use super::*;
use crate::namespaces::{M, R};
use crate::test_docs::{Lcg, Shapes, para, plain, split, words};
use CorrelationStatus::{Deleted, Equal, Inserted};

fn source(dom: &mut Dom, fragment: &str, settings: &WmlComparerSettings) -> Vec<ComparisonUnit> {
    let doc = dom.parse_xdocument(&format!(
        "<w:document xmlns:w=\"{}\" xmlns:m=\"{}\" xmlns:r=\"{}\"><w:body>{fragment}<w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr></w:body></w:document>",
        W::URI,
        M::URI,
        R::URI
    ));
    let root = dom.root(doc).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    super::super::preprocess::add_sha1_hash_to_block_level_content(
        dom,
        body,
        settings,
        &super::super::preprocess::null_rel_resolver,
    );
    let atoms = super::super::atomize::create_comparison_unit_atom_list(dom, body, settings);
    super::super::units::get_comparison_unit_list(dom, &atoms, settings)
}

/// `(node, element name, text)` of every atom, in source order.
fn atoms(dom: &Dom, units: &[ComparisonUnit]) -> Vec<(NodeId, String, String)> {
    units
        .iter()
        .flat_map(ComparisonUnit::descendant_atoms)
        .map(|a| {
            (
                a.content_element,
                dom.name(a.content_element)
                    .map(|n| n.local_name().to_string())
                    .unwrap_or_default(),
                dom.value_str(a.content_element).into_owned(),
            )
        })
        .collect()
}

fn side(out: &[CorrelatedSequence], left: bool) -> Vec<ComparisonUnit> {
    out.iter()
        .flat_map(|s| {
            if left {
                s.com_units_1.clone().unwrap_or_default()
            } else {
                s.com_units_2.clone().unwrap_or_default()
            }
        })
        .collect()
}

/// Both sources survive a resolver: same atoms in the same order, and
/// every sequence is resolved with the side its status implies.
fn conserve(
    dom: &Dom,
    out: &[CorrelatedSequence],
    left: &[(NodeId, String, String)],
    right: &[(NodeId, String, String)],
    context: &str,
) -> Result<(), String> {
    for (is_left, expected) in [(true, left), (false, right)] {
        let actual = atoms(dom, &side(out, is_left));
        // The story's closing pilcrow may be absorbed by the closing
        // carrier, as Word does; every other atom must stay.
        let expected = match expected.split_last() {
            Some((last, rest)) if last.1 == "pPr" && actual.len() == rest.len() => rest,
            _ => expected,
        };
        if let Some(i) =
            (0..actual.len().max(expected.len())).find(|&i| actual.get(i) != expected.get(i))
        {
            return Err(format!(
                "{context}: {} side differs at {i} of {}/{}: {:?} vs {:?}",
                if is_left { "original" } else { "revised" },
                actual.len(),
                expected.len(),
                &actual[i.saturating_sub(2)..(i + 3).min(actual.len())],
                &expected[i.saturating_sub(2)..(i + 3).min(expected.len())]
            ));
        }
    }
    for s in out {
        match s.correlation_status {
            Deleted => assert!(
                s.com_units_2.as_deref().unwrap_or_default().is_empty(),
                "{context}"
            ),
            Inserted => assert!(
                s.com_units_1.as_deref().unwrap_or_default().is_empty(),
                "{context}"
            ),
            Equal => {}
            other => return Err(format!("{context}: unresolved {other:?}")),
        }
    }
    Ok(())
}

/// The visible text of atoms, ignoring the paragraph marks and payloads.
fn text(atoms: &[(NodeId, String, String)]) -> String {
    atoms
        .iter()
        .filter(|a| a.1 == "t" || a.1 == "delText")
        .map(|a| a.2.as_str())
        .collect()
}

fn table(rows: &[Vec<String>]) -> String {
    let mut out = String::from("<w:tbl><w:tblPr><w:tblStyle w:val=\"TableGrid\"/></w:tblPr>");
    for row in rows {
        out.push_str("<w:tr>");
        for cell in row {
            out.push_str(&format!("<w:tc>{}</w:tc>", plain(cell)));
        }
        out.push_str("</w:tr>");
    }
    out.push_str("</w:tbl>");
    out
}

/// A paragraph whose only run carries a VML text box with one paragraph.
fn textbox(text: &str) -> String {
    format!(
        "<w:p><w:r><w:pict><v:shape xmlns:v=\"urn:schemas-microsoft-com:vml\" style=\"width:90pt;height:30pt\"><v:textbox><w:txbxContent>{}</w:txbxContent></v:textbox></v:shape></w:pict></w:r></w:p>",
        plain(text)
    )
}

const POOL: &str = "harbor lantern meadow copper orchard summit glacier willow saffron \
    bronze quartz harvest violet cobalt silver mariner agreement client \
    payment notice section party term report the and for with to of a is \
    Document Tester Demo List Table Style Text Bold";

const SHAPES: usize = 32;

/// One document of shape `shape` over `pool`.
fn shape_doc(rng: &mut Lcg, pool: &[&str], shape: usize) -> Vec<String> {
    let shapes = Shapes {
        pool,
        filler_span: 10,
    };
    let pick = |rng: &mut Lcg| shapes.pick(rng);
    let filler = |rng: &mut Lcg, n: usize| shapes.filler(rng, n);
    let mut out: Vec<String> = Vec::new();
    match shape {
        0 => {
            for label in ["ONE", "a", "TWO", "b"] {
                out.push(para(label, "", Some(u8::from(label.len() == 1)), ""));
            }
        }
        1 => {
            let n = 5 + rng.below(14);
            for i in 0..n {
                let label = match i % 4 {
                    0 => ["ONE", "TWO", "THREE", "FOUR", "FIVE"][(i / 4) % 5].to_string(),
                    1 => ["a", "b", "c", "d"][(i / 4) % 4].to_string(),
                    2 => ["ii", "iii", "iv", "v"][(i / 4) % 4].to_string(),
                    _ => format!("{}", i + 1),
                };
                out.push(para(&label, "", Some((i % 3) as u8), ""));
            }
        }
        2 => {
            out.push(para(&format!("{} agreement", pick(rng)), "Title", None, ""));
            out.push(plain(&format!("Prepared by {}", pick(rng))));
            out.push(plain("March 10, 2040"));
            out.push(plain(&format!("To {}", pick(rng))));
            out.push(plain(&format!("From {}", pick(rng))));
            if rng.chance(50) {
                out.push(plain("[Client Name]"));
            }
        }
        3 => {
            let n = 4 + rng.below(8);
            for i in 0..n {
                out.push(plain(["a", "x", "b", "c", "Ab", "XYZ", "q"][i % 7]));
            }
        }
        4 => {
            out.push(para("Fields test", "Title", None, ""));
            out.push(plain("html input type text"));
            let n = rng.below(4);
            out.extend(filler(rng, n));
        }
        5 => {
            out.push(plain(&format!(
                "Oftentimes {} authors suggest amendments",
                pick(rng)
            )));
            out.push(plain("Please leave a comment on this paragraph"));
            let n = rng.below(3);
            out.extend(filler(rng, n));
        }
        6 => {
            out.push(para("Math borderBox demo", "Title", None, ""));
            for _ in 0..2 + rng.below(6) {
                out.push(format!(
                    "<w:p><w:r><w:t xml:space=\"preserve\">{} </w:t></w:r><m:oMath><m:borderBox><m:e><m:r><m:t>x</m:t></m:r></m:e></m:borderBox></m:oMath></w:p>",
                    pick(rng)
                ));
            }
        }
        7 => {
            out.push(plain("Num words 20"));
            out.push(plain("Num chars 120"));
            out.push(plain("Num pages 3"));
            out.push(plain("test"));
            out.push(plain("page 3"));
        }
        8 => {
            out.extend(shapes.memo(rng, 2, 30));
        }
        9 => {
            out.push(para("OOXML w:b property tester", "Title", None, ""));
            out.push(para("Bold on", "", None, "<w:b/>"));
            out.push(para("Bold off", "", None, "<w:b w:val=\"0\"/>"));
            if rng.chance(50) {
                out.push(table(&[
                    vec!["w:sz".into(), "24".into()],
                    vec!["w:color".into(), "red".into()],
                ]));
            }
        }
        10 => {
            out.push(plain(&format!("{} report", pick(rng))));
            for t in 0..1 + rng.below(6) {
                out.push(table(&[
                    vec![format!("{} {t}", pick(rng)), pick(rng)],
                    vec!["1".into(), "2".into()],
                ]));
            }
        }
        11 => {
            let n = 25 + rng.below(40);
            out.extend(filler(rng, n));
            for t in 0..1 + rng.below(5) {
                out.push(table(&[vec![
                    format!("{} total {t}", pick(rng)),
                    "9".into(),
                ]]));
                out.extend(filler(rng, 2));
            }
        }
        12 => {
            let rows = 1 + rng.below(4);
            out.push(table(
                &(0..rows)
                    .map(|r| (0..3).map(|c| format!("{}{r}{c}", pick(rng))).collect())
                    .collect::<Vec<_>>(),
            ));
        }
        13 => {
            let n = 18 + rng.below(20);
            out.push(plain(&words(rng, pool, n)));
        }
        14 => {
            out.extend(shapes.essay(rng, 1, 25));
        }
        15 => {
            out.extend(shapes.demo(rng));
        }
        16 => {
            out.extend(shapes.list_demo(rng));
        }
        17 => {
            out.extend(shapes.lettered(rng));
        }
        18 => {
            out.push(para(&format!("{} Tester", pick(rng)), "Title", None, ""));
            let n = 1 + rng.below(5);
            out.extend(filler(rng, n));
            if rng.chance(50) {
                out.push(table(&[vec!["k".into(), "v".into()]]));
            }
        }
        19 => {
            out.push(para(&format!("{} summary", pick(rng)), "Title", None, ""));
            for i in 0..2 + rng.below(6) {
                out.push(plain(&format!("{}: {} {i}", pick(rng), pick(rng))));
            }
        }
        20 => {
            out.push(para(
                &format!("{} heading", pick(rng)),
                "Heading1",
                None,
                "",
            ));
            for i in 0..4 + rng.below(12) {
                out.push(para(
                    &format!("{} {}", pick(rng), i + 1),
                    "ListParagraph",
                    Some((i % 3) as u8),
                    "",
                ));
            }
        }
        21 => {
            out.push(plain(""));
            out.push(plain(""));
            out.push("<w:p><w:r><w:drawing><wp:inline xmlns:wp=\"http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing\"><wp:extent cx=\"10\" cy=\"10\"/><wp:docPr id=\"1\" name=\"P\"/></wp:inline></w:drawing></w:r></w:p>".to_string());
        }
        22 => {
            out.push(para(&format!("{} Document", pick(rng)), "Title", None, ""));
            let n = 1 + rng.below(4);
            out.extend(filler(rng, n));
        }
        23 => {
            out.push(plain("....."));
            out.push(plain("—"));
            out.push(plain(&pick(rng)));
        }
        24 => {
            out.push(para("Tab Tests", "Title", None, ""));
            out.push(plain("CENTER TAB"));
            out.push(plain("First Second End"));
            out.push("<w:p><w:r><w:tab/><w:t>right</w:t></w:r></w:p>".to_string());
        }
        25 => {
            for i in 0..4 + rng.below(30) {
                out.push(para(&format!("{} {}", pick(rng), i), "", Some(0), ""));
            }
        }
        26 => {
            out.push(para(
                &format!("{} Table Style Demo", pick(rng)),
                "Title",
                None,
                "",
            ));
            out.push(table(&[
                vec!["Name".into(), "Role".into()],
                vec![pick(rng), pick(rng)],
            ]));
            out.push(plain(&format!("This table shows {}", pick(rng))));
        }
        27 => {
            out.push(para(
                &format!("Part {}", rng.below(9)),
                "Heading1",
                None,
                "",
            ));
            out.push(plain(&words(rng, pool, 6)));
            out.push("<w:p><w:pPr><w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr></w:pPr></w:p>".to_string());
            out.push(para(
                &format!("Part {}", rng.below(9)),
                "Heading1",
                None,
                "",
            ));
            out.push(plain(&words(rng, pool, 6)));
        }
        28 => {
            out.push(para("SD-2672 plain 3x3", "Title", None, ""));
            out.push(plain(""));
            out.push(table(&[
                vec!["1".into(), "2".into(), "3".into()],
                vec!["4".into(), "5".into(), "6".into()],
                vec!["7".into(), "8".into(), "9".into()],
            ]));
        }
        29 => {
            out.push(para(
                &format!("{} Text Box Demo", pick(rng)),
                "Title",
                None,
                "",
            ));
            for _ in 0..1 + rng.below(3) {
                out.push(textbox(&words(rng, pool, 4)));
                out.push(plain(&words(rng, pool, 5)));
            }
        }
        30 => {
            out.push(textbox(&pick(rng)));
            out.push(table(&[vec![pick(rng), pick(rng)]]));
            out.push(textbox(&words(rng, pool, 3)));
        }
        _ => {
            out.push(plain(&format!("{}", rng.below(100))));
            out.push(plain(&words(rng, pool, 2)));
        }
    }
    if rng.chance(25) {
        let n = 1 + rng.below(4);
        out.extend(filler(rng, n));
    }
    if rng.chance(20) {
        out.insert(0, plain(""));
    }
    out
}

/// Small edits of a block list: delete, insert, swap or reword one block.
fn edit(rng: &mut Lcg, doc: &[String], pool: &[&str]) -> Vec<String> {
    let mut out = doc.to_vec();
    for _ in 0..1 + rng.below(3) {
        let at = rng.below(out.len().max(1));
        match rng.below(4) {
            0 if out.len() > 1 => {
                out.remove(at);
            }
            1 => {
                let k = 2 + rng.below(8);
                out.insert(at, plain(&words(rng, pool, k)));
            }
            2 if out.len() > 1 => {
                let other = rng.below(out.len());
                out.swap(at, other);
            }
            _ => {
                let k = 2 + rng.below(8);
                let last = out.len() - 1;
                out[at.min(last)] = plain(&words(rng, pool, k));
            }
        }
    }
    out
}

/// Bare words at the top level: the first paragraph with words is
/// replaced by its own contents, as a story without paragraph groups.
fn flatten_first(cu: Vec<ComparisonUnit>) -> Vec<ComparisonUnit> {
    let Some(i) = cu.iter().position(|u| {
        as_group(u).is_some_and(|g| {
            g.group_type == ComparisonUnitGroupType::Paragraph && g.contents.len() > 1
        })
    }) else {
        return cu;
    };
    let mut out = cu;
    let g = as_group(&out[i]).unwrap().contents.clone();
    out.splice(i..=i, g);
    out
}

fn presets() -> Vec<(&'static str, WmlComparerSettings)> {
    vec![
        ("word", WmlComparerSettings::default()),
        (
            "faithful",
            WmlComparerSettings {
                merge_replaced_paragraphs: false,
                detail_threshold: 0.0,
                ..WmlComparerSettings::default()
            },
        ),
        (
            "coarse",
            WmlComparerSettings {
                detail_threshold: 0.15,
                ..WmlComparerSettings::default()
            },
        ),
    ]
}

/// One pair through the unrelated-source gates, the paragraph window and
/// the full driver, in every preset and with bare-word variants.
fn run_pair(a: &str, b: &str, context: &str, failures: &mut Vec<String>) -> usize {
    let mut replaced = 0;
    for (name, settings) in presets() {
        for variant in 0..3 {
            let mut dom = Dom::new();
            let mut cu1 = source(&mut dom, a, &settings);
            let mut cu2 = source(&mut dom, b, &settings);
            if variant == 1 {
                cu1 = flatten_first(cu1);
            } else if variant == 2 {
                cu2 = flatten_first(cu2);
            }
            let ctx = format!("{context} [{name}/{variant}]");
            let left = atoms(&dom, &cu1);
            let right = atoms(&dom, &cu2);
            if let Some(out) =
                detect_unrelated_sources_word_mode_inner(&mut dom, &cu1, &cu2, &settings)
            {
                let case = format!("unrelated {ctx}");
                match conserve(&dom, &out, &left, &right, &case) {
                    Err(e) => failures.push(e),
                    Ok(()) if KNOWN_DEFECTS.contains(&case.as_str()) => failures.push(format!(
                        "ratchet: {case} is listed in KNOWN_DEFECTS but now keeps both sources; remove it"
                    )),
                    Ok(()) => {}
                }
                replaced += 1;
            }
            let out = lcs(&mut dom, cu1, cu2, &settings);
            assert!(
                out.iter()
                    .all(|s| s.correlation_status != CorrelationStatus::Unknown),
                "lcs {ctx}: unresolved sequence"
            );
            let l = text(&atoms(&dom, &side(&out, true)));
            let r = text(&atoms(&dom, &side(&out, false)));
            assert!(
                l.len() >= text(&left).len().min(1) && r.len() >= text(&right).len().min(1),
                "lcs {ctx}: a non-empty source lost all its text"
            );
        }
    }
    replaced
}

/// `count` seeded pairs. With `same_pool` both sides draw their words
/// from one small pool, so the gates meet fingerprinted shapes whose
/// vocabularies overlap and must decline them.
fn sweep(seed: u64, count: usize, same_pool: bool) {
    let pool = split(POOL);
    let mut replaced = 0;
    let mut failures = Vec::new();
    for case in 0..count {
        let mut rng = Lcg(seed
            .wrapping_add(case as u64)
            .wrapping_mul(0x9e37_79b9_7f4a_7c15));
        let start = rng.below(pool.len() - 10);
        let width = if same_pool {
            3 + rng.below(4)
        } else {
            4 + rng.below(16)
        };
        let left_pool = &pool[start..(start + width).min(pool.len())];
        let shift = if same_pool { 0 } else { rng.below(10) };
        let right_start = (start + shift).min(pool.len() - 4);
        let right_pool = &pool[right_start..(right_start + width).min(pool.len())];
        let sa = case % SHAPES;
        let sb = (case / SHAPES + case * 7) % SHAPES;
        let a = shape_doc(&mut rng, left_pool, sa);
        let b = if rng.chance(25) {
            edit(&mut rng, &a, right_pool)
        } else {
            shape_doc(&mut rng, right_pool, sb)
        };
        replaced += run_pair(
            &a.concat(),
            &b.concat(),
            &format!("{seed}+{case} shapes {sa}->{sb}"),
            &mut failures,
        );
    }
    // Unrelated shapes do meet the whole-replacement gates.
    assert!(replaced > 0, "no pair took a whole-replacement gate");
    check(&failures);
}

#[test]
fn shape_pairs_keep_both_sources_0() {
    sweep(1, 240, false);
}

#[test]
fn shape_pairs_keep_both_sources_1() {
    sweep(5_000, 240, false);
}

#[test]
fn shape_pairs_keep_both_sources_2() {
    sweep(9_000, 240, false);
}

#[test]
fn shape_pairs_keep_both_sources_3() {
    sweep(13_000, 240, false);
}

/// Every shape against every other, without random edits.
#[test]
fn every_shape_against_every_shape_keeps_both_sources() {
    let pool = split(POOL);
    let mut replaced = 0;
    let mut failures = Vec::new();
    for sa in 0..SHAPES {
        for sb in 0..SHAPES {
            let mut rng = Lcg((sa * 97 + sb) as u64);
            let a = shape_doc(&mut rng, &pool[..16], sa);
            let b = shape_doc(&mut rng, &pool[8..28], sb);
            replaced += run_pair(
                &a.concat(),
                &b.concat(),
                &format!("shapes {sa}->{sb}"),
                &mut failures,
            );
        }
    }
    assert!(replaced > 0);
    check(&failures);
}

/// KNOWN DEFECTS, not expected behaviour: whole replacements that lose or
/// reorder a source atom. The list is a ratchet: a new case fails the
/// sweep, and so does a listed case that keeps both sources (remove its
/// entry), so it can only shrink.
const KNOWN_DEFECTS: &[&str] = &[
    "unrelated 21000+239 shapes 15->16 [coarse/0]",
    "unrelated 21000+239 shapes 15->16 [faithful/0]",
    "unrelated 21000+239 shapes 15->16 [word/0]",
    "unrelated 25000+198 shapes 6->16 [coarse/0]",
    "unrelated 25000+198 shapes 6->16 [faithful/0]",
    "unrelated 25000+198 shapes 6->16 [word/0]",
    "unrelated 1+189 shapes 29->16 [coarse/0]",
    "unrelated 1+189 shapes 29->16 [faithful/0]",
    "unrelated 1+189 shapes 29->16 [word/0]",
    "unrelated 13000+112 shapes 16->19 [coarse/2]",
    "unrelated 13000+112 shapes 16->19 [faithful/2]",
    "unrelated 13000+112 shapes 16->19 [word/2]",
    "unrelated 13000+189 shapes 29->16 [coarse/0]",
    "unrelated 13000+189 shapes 29->16 [faithful/0]",
    "unrelated 13000+189 shapes 29->16 [word/0]",
    "unrelated shapes 15->16 [coarse/0]",
    "unrelated shapes 15->16 [faithful/0]",
    "unrelated shapes 15->16 [word/0]",
    "unrelated shapes 16->29 [coarse/0]",
    "unrelated shapes 16->29 [faithful/0]",
    "unrelated shapes 16->29 [word/0]",
];

fn check(failures: &[String]) {
    let unknown: Vec<_> = failures
        .iter()
        .filter(|f| {
            !KNOWN_DEFECTS
                .iter()
                .any(|k| f.starts_with(&format!("{k}:")))
        })
        .collect();
    assert!(
        unknown.is_empty(),
        "{} of {} failures are new:\n{}",
        unknown.len(),
        failures.len(),
        unknown
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn shared_vocabulary_shape_pairs_keep_both_sources_0() {
    sweep(17_000, 300, true);
}

#[test]
fn shared_vocabulary_shape_pairs_keep_both_sources_1() {
    sweep(21_000, 300, true);
}

#[test]
fn shared_vocabulary_shape_pairs_keep_both_sources_2() {
    sweep(25_000, 300, true);
}

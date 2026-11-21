// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;
struct PolicyCase {
    name: String,
    inserted: Vec<(String, bool)>,
    deleted: Vec<(String, bool)>,
    fold: bool,
}
const DATE: &str = "2026-01-02T03:04:05Z";
fn paragraph(kind: &str, id: u32, text: &str, numbered: bool, history: bool) -> String {
    let num = if numbered {
        "<w:numPr><w:ilvl w:val='0'/><w:numId w:val='7'/></w:numPr>"
    } else {
        ""
    };
    let saved = if history {
        format!(
            "<w:pPrChange w:id='{}' w:author='Earlier layout owner' w:date='2025-01-01T00:00:00Z'><w:pPr><w:spacing w:after='60'/></w:pPr></w:pPrChange>",
            id + 100
        )
    } else {
        String::new()
    };
    let tag = if kind == "del" { "delText" } else { "t" };
    format!(
        "<w:p><w:pPr><w:pStyle w:val='BodyText'/>{num}<w:spacing w:after='80'/><w:rPr><w:{kind} w:id='{}' w:author='Authored mark owner' w:date='{DATE}'/><w:i/></w:rPr>{saved}</w:pPr><w:{kind} w:id='{id}' w:author='Authored {kind} text owner' w:date='{DATE}'><w:r><w:rPr><w:b/><w:color w:val='123456'/></w:rPr><w:{tag}>{text}</w:{tag}></w:r></w:{kind}></w:p>",
        id + 50
    )
}
#[test]
fn document_policy_reaches_numbering_relatedness_and_demo_intro_boundaries() {
    let mut cases = Vec::new();
    let long_old = ["Established legacy charter obligations remain enforceable"; 10].join(" ");
    // M428: without either boundary numbering owner, this is a large,
    // unrelated asymmetric replacement; with both, Word's uniform-list
    // exception retains a MIX. Neither alternative earlier gate qualifies.
    for first_old_numbered in [false, true] {
        for last_new_numbered in [false, true] {
            cases.push(PolicyCase {
                name: format!(
                    "uniform-list/old-num={first_old_numbered}/new-num={last_new_numbered}"
                ),
                inserted: vec![
                    ("Zebra".into(), false),
                    ("Zebra".into(), false),
                    ("Zebra".into(), last_new_numbered),
                ],
                deleted: vec![
                    ("Num One".into(), first_old_numbered),
                    (long_old.clone(), false),
                    (long_old.clone(), false),
                ],
                fold: first_old_numbered && last_new_numbered,
            });
        }
    }
    // M413: a title-page boundary itself remains unrelated. An earlier
    // revised paragraph matching the old clause is independently meaningful
    // source relatedness and prevents treating the entire gap as wholesale.
    for earlier_clause_related in [false, true] {
        let old_clause = "Original obligations bind parties permanently";
        cases.push(PolicyCase {
            name: format!("cover/earlier-clause-related={earlier_clause_related}"),
            inserted: vec![
                (
                    if earlier_clause_related {
                        old_clause
                    } else {
                        "Revised editorial summary"
                    }
                    .into(),
                    false,
                ),
                ("New appendix overview".into(), false),
                ("Updated approval workflow".into(), false),
                ("Prepared revised edition".into(), false),
            ],
            deleted: vec![
                (old_clause.into(), false),
                ("Former execution requirements".into(), false),
            ],
            fold: earlier_clause_related,
        });
    }
    // M336: Word explicitly folds a short item into its demo introduction.
    // The unrelated ordinary clause control does not have that exception.
    for (name, first_old, fold) in [
        (
            "demonstrates",
            "This report demonstrates original sampling procedures",
            true,
        ),
        (
            "document-prefix",
            "This document records original sampling procedures",
            true,
        ),
        (
            "ordinary-clause",
            "Original clause covers sampling procedures completely",
            false,
        ),
    ] {
        cases.push(PolicyCase {
            name: format!("intro/{name}"),
            inserted: vec![
                ("Alpha".into(), false),
                ("Beta".into(), false),
                ("Zebra".into(), false),
            ],
            deleted: vec![
                (first_old.into(), false),
                (long_old.clone(), false),
                (long_old.clone(), false),
            ],
            fold,
        });
    }
    for case in cases {
        for history in [false, true] {
            for in_cell in [false, true] {
                let mut paragraphs = String::new();
                for (index, (text, numbered)) in case.inserted.iter().enumerate() {
                    paragraphs.push_str(&paragraph(
                        "ins",
                        index as u32 + 1,
                        text,
                        *numbered,
                        history,
                    ));
                }
                for (index, (text, numbered)) in case.deleted.iter().enumerate() {
                    paragraphs.push_str(&paragraph(
                        "del",
                        index as u32 + 20,
                        text,
                        *numbered,
                        history,
                    ));
                }
                let content = if in_cell {
                    format!(
                        "<w:tbl><w:tblPr><w:tblW w:w='1800' w:type='dxa'/></w:tblPr><w:tblGrid><w:gridCol w:w='1800'/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w='1800' w:type='dxa'/></w:tcPr>{paragraphs}</w:tc></w:tr></w:tbl>"
                    )
                } else {
                    paragraphs
                };
                let source = format!(
                    "<w:document xmlns:w='{}'><w:body>{content}<w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr></w:body></w:document>",
                    W::URI
                );
                let mut dom = Dom::new();
                let document = dom.parse_xdocument(&source);
                let root = dom.root(document).expect("document");
                let body = dom.element(root, &W::body()).expect("body");
                let container = if in_cell {
                    dom.descendants(body, Some(&W::tc()))[0]
                } else {
                    body
                };
                let p = dom.elements(container, Some(&W::p()));
                let (inss, dels) = p.split_at(case.inserted.len());
                let before = dom.serialize_element(root);
                let nodes = dom.node_count();
                let coordinate = format!("{}/history={history}/cell={in_cell}", case.name);
                for _ in 0..2 {
                    assert_eq!(
                        should_fold_multi_del_at_document_scale(
                            &dom,
                            container,
                            *inss.last().expect("new boundary"),
                            dels[0],
                            inss,
                            dels
                        ),
                        case.fold,
                        "{coordinate}: recorded Word policy"
                    );
                    assert_eq!(
                        dom.serialize_element(root),
                        before,
                        "{coordinate}: entire authored source remains unchanged"
                    );
                    assert_eq!(dom.node_count(), nodes, "{coordinate}: read-only policy");
                }
            }
        }
    }
}

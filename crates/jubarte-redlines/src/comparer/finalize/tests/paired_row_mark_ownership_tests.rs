// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;

fn canonical(dom: &Dom, node: NodeId) -> String {
    let mut attrs = dom
        .attributes_iter(node)
        .filter(|(n, _)| !dom.is_namespace_declaration(n))
        .map(|(n, v)| {
            (
                n.namespace_name().to_string(),
                n.local_name().to_string(),
                v,
            )
        })
        .collect::<Vec<_>>();
    attrs.sort();
    let children = dom
        .nodes(node)
        .into_iter()
        .map(|child| {
            let encoded = canonical(dom, child);
            format!("{}:{encoded}", encoded.len())
        })
        .collect::<String>();
    let header = format!("{:?}:{attrs:?}:{:?}", dom.name(node), dom.text_value(node));
    // Length-prefixed child encodings preserve the complete tree without
    // recursively Debug-escaping every nested property's representation.
    format!("{}:{header}{}:{children}", header.len(), children.len())
}

fn project(dom: &mut Dom, root: NodeId, accept: bool) -> (String, Vec<usize>, String) {
    let copy = dom.clone_subtree(root);
    let projected = if accept {
        crate::revision_processor::accept_revisions_document(dom, copy)
    } else {
        crate::revision_processor::reject_revisions_document(dom, copy)
    };
    let rows = dom
        .descendants(projected, Some(&W::tr()))
        .into_iter()
        .map(|r| dom.elements(r, Some(&W::tc())).len())
        .collect();
    let text = dom
        .descendants(projected, Some(&W::t()))
        .into_iter()
        .map(|n| dom.value(n))
        .collect();
    (canonical(dom, projected), rows, text)
}

#[test]
fn paired_rows_with_surviving_marks_keep_empty_revised_cells_and_full_original_history() {
    for kind in ["del", "ins"] {
        let leaf = if kind == "del" { "delText" } else { "t" };
        let paired_props = "<w:trPr><w:cantSplit/><w:trPrChange w:id='7'><w:trPr><w:trHeight w:val='500'/></w:trPr></w:trPrChange></w:trPr>";
        let cell = |marked: bool| {
            let mark = if marked {
                format!("<w:{kind} w:id='2'/>")
            } else {
                String::new()
            };
            format!(
                "<w:tc><w:tcPr><w:tcW w:w='2000' w:type='dxa'/><w:tcPrChange w:id='9'><w:tcPr><w:tcW w:w='1800' w:type='dxa'/></w:tcPr></w:tcPrChange></w:tcPr><w:p><w:pPr><w:spacing w:after='60'/><w:rPr>{mark}</w:rPr></w:pPr><w:{kind} w:id='3'><w:r><w:rPr><w:b/></w:rPr><w:{leaf}>original</w:{leaf}></w:r></w:{kind}></w:p></w:tc>"
            )
        };
        for all_owned in [false, true] {
            let cells = (0..4)
                .map(|i| cell(all_owned || i == 3))
                .collect::<String>();
            let mut dom = Dom::new();
            let doc = dom.parse_xdocument(&format!("<w:document xmlns:w='{}'><w:body><w:tbl><w:tblPr><w:tblW w:w='8000' w:type='dxa'/></w:tblPr><w:tblGrid><w:gridCol w:w='2000'/><w:gridCol w:w='2000'/><w:gridCol w:w='2000'/><w:gridCol w:w='2000'/></w:tblGrid><w:tr>{paired_props}{cells}</w:tr></w:tbl><w:sectPr/></w:body></w:document>", W::URI));
            let root = dom.root(doc).unwrap();
            let before_accept = project(&mut dom, root, true);
            let before_reject = project(&mut dom, root, false);
            mark_fully_revised_rows(&mut dom, root, &WmlComparerSettings::default(), &mut 10);
            let row = dom.descendants(root, Some(&W::tr()))[0];
            let properties = dom.element(row, &W::tr_pr()).unwrap();
            assert_eq!(dom.element(properties, &W::name(kind)).is_some(), all_owned);
            if !all_owned {
                assert_eq!(project(&mut dom, root, true), before_accept);
                assert_eq!(project(&mut dom, root, false), before_reject);
                let (accepted, counts, _) = project(&mut dom, root, kind == "del");
                assert_eq!(counts, vec![4]);
                assert!(!accepted.contains("original"));
                let (_, counts, text) = project(&mut dom, root, kind != "del");
                assert_eq!(counts, vec![4]);
                assert_eq!(text, "original".repeat(4));
            }
        }
    }
}
#[test]
fn unchanged_empty_pilcrows_keep_row_ownership_without_row_property_history() {
    for kind in ["ins", "del"] {
        for nested_control in [false, true] {
            for formatted_empty in [false, true] {
                let inert_run = if formatted_empty {
                    "<w:r><w:rPr><w:b/><w14:textFill><w14:solidFill><w14:srgbClr w14:val='112233'/></w14:solidFill></w14:textFill></w:rPr></w:r>"
                } else {
                    ""
                };
                let live =
                    format!("<w:p><w:pPr><w:spacing w:after='240'/></w:pPr>{inert_run}</w:p>");
                let live = if nested_control {
                    format!(
                        "<w:sdt><w:sdtPr><w:id w:val='42'/><w:tag w:val='Clause'/></w:sdtPr><w:sdtContent>{live}</w:sdtContent></w:sdt>"
                    )
                } else {
                    live
                };
                let owned = format!(
                    "<w:p><w:pPr><w:spacing w:after='120'/><w:rPr><w:{kind} w:id='8' w:author='Editor'/></w:rPr></w:pPr></w:p>"
                );
                let mut dom = Dom::new();
                let doc = dom.parse_xdocument(&format!("<w:document xmlns:w='{}' xmlns:w14='http://schemas.microsoft.com/office/word/2010/wordml'><w:body><w:tbl><w:tblPr><w:tblW w:w='3600' w:type='dxa'/></w:tblPr><w:tblGrid><w:gridCol w:w='1800'/><w:gridCol w:w='1800'/></w:tblGrid><w:tr><w:trPr><w:cantSplit/></w:trPr><w:tc><w:tcPr><w:tcW w:w='1800' w:type='dxa'/><w:shd w:fill='ABCDEF'/></w:tcPr>{owned}{live}</w:tc><w:tc><w:tcPr><w:tcW w:w='1800' w:type='dxa'/></w:tcPr>{owned}</w:tc></w:tr></w:tbl><w:sectPr/></w:body></w:document>", W::URI));
                let root = dom.root(doc).unwrap();
                let before = canonical(&dom, root);
                let original = project(&mut dom, root, false);
                let revised = project(&mut dom, root, true);
                let mut next_id = 90;
                mark_fully_revised_rows(
                    &mut dom,
                    root,
                    &WmlComparerSettings::default(),
                    &mut next_id,
                );
                assert_eq!(
                    canonical(&dom, root),
                    before,
                    "{kind}, nested={nested_control}, formatted={formatted_empty}"
                );
                assert_eq!(next_id, 90);
                assert_eq!(project(&mut dom, root, false), original);
                assert_eq!(project(&mut dom, root, true), revised);
                assert_eq!(original.1, vec![2]);
                assert_eq!(revised.1, vec![2]);
            }
        }
    }
}

#[test]
fn new_empty_rows_with_owned_pilcrows_still_receive_their_row_lifetime() {
    for kind in ["ins", "del"] {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&format!("<w:document xmlns:w='{}'><w:body><w:tbl><w:tr><w:tc><w:p><w:pPr><w:rPr><w:{kind} w:id='8'/></w:rPr></w:pPr></w:p></w:tc></w:tr></w:tbl></w:body></w:document>", W::URI));
        let root = dom.root(doc).unwrap();
        let mut next_id = 90;
        mark_fully_revised_rows(
            &mut dom,
            root,
            &WmlComparerSettings::default(),
            &mut next_id,
        );
        let row = dom.descendants(root, Some(&W::tr()))[0];
        let row_props = dom.element(row, &W::tr_pr()).unwrap();
        assert!(dom.element(row_props, &W::name(kind)).is_some());
        assert_eq!(next_id, 91);
        let survives = kind == "ins";
        assert_eq!(project(&mut dom, root, survives).1, vec![1]);
        assert!(project(&mut dom, root, !survives).1.is_empty());
    }
}

#[test]
fn public_empty_table_comparison_preserves_complete_authored_rows_and_pilcrows() {
    fn source_table(dom: &mut Dom, root: NodeId, accept: bool) -> String {
        let copy = dom.clone_subtree(root);
        let projected = if accept {
            crate::revision_processor::accept_revisions_document(dom, copy)
        } else {
            crate::revision_processor::reject_revisions_document(dom, copy)
        };
        remove_powertools_scratch_markup(dom, projected);
        for local in ["rPr", "pPr", "trPr"] {
            for properties in dom.descendants(projected, Some(&W::name(local))) {
                if dom
                    .attributes(properties)
                    .iter()
                    .all(|(name, _)| dom.is_namespace_declaration(name))
                    && dom.nodes(properties).is_empty()
                {
                    dom.remove(properties);
                }
            }
        }
        let tables = dom.descendants(projected, Some(&W::tbl()));
        assert_eq!(tables.len(), 1, "both authored sources own this same table");
        canonical(dom, tables[0])
    }
    for (a_count, b_count) in [(1, 2), (2, 1), (3, 8), (8, 3)] {
        for whitespace in [false, true] {
            for word in [false, true] {
                let settings = if word {
                    WmlComparerSettings::default()
                } else {
                    WmlComparerSettings::powertools_faithful()
                };
                let mut dom = Dom::new();
                let roots = [a_count, b_count].map(|count| {
                        let empty = format!("<w:p><w:pPr><w:pStyle w:val='BodyText'/><w:spacing w:before='120' w:after='80'/></w:pPr>{}</w:p>", if whitespace { "<w:r><w:rPr><w:b/></w:rPr><w:t xml:space='preserve'>   </w:t></w:r>" } else { "" });
                        let doc = dom.parse_xdocument(&format!("<w:document xmlns:w='{}'><w:body><w:tbl><w:tblPr><w:tblW w:w='3600' w:type='dxa'/><w:tblLook w:val='04A0' w:firstRow='1' w:lastRow='0' w:firstColumn='1' w:lastColumn='0' w:noHBand='0' w:noVBand='1'/></w:tblPr><w:tblGrid><w:gridCol w:w='3600'/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w='3600' w:type='dxa'/><w:shd w:fill='ABCDEF'/></w:tcPr>{}<w:p/></w:tc></w:tr></w:tbl><w:sectPr/></w:body></w:document>", W::URI, empty.repeat(count)));
                        dom.root(doc).unwrap()
                    });
                let original = source_table(&mut dom, roots[0], false);
                let revised = source_table(&mut dom, roots[1], true);
                let bodies = roots.map(|root| dom.element(root, &W::body()).unwrap());
                let output = crate::comparer::compare_bodies_faithful(
                    &mut dom, roots[0], roots[1], bodies[0], bodies[1], &settings,
                );
                assert_eq!(
                    source_table(&mut dom, output, false),
                    original,
                    "original={a_count}, revised={b_count}, whitespace={whitespace}, word={word}"
                );
                assert_eq!(
                    source_table(&mut dom, output, true),
                    revised,
                    "original={a_count}, revised={b_count}, whitespace={whitespace}, word={word}"
                );
            }
        }
    }
}

#[test]
fn explicit_row_lifetime_is_not_overridden_by_a_surviving_cell_mark() {
    for kind in ["ins", "del"] {
        let leaf = if kind == "del" { "delText" } else { "t" };
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&format!("<w:document xmlns:w='{}'><w:body><w:tbl><w:tr><w:trPr><w:{kind} w:id='1'/><w:trPrChange w:id='2'><w:trPr><w:cantSplit/></w:trPr></w:trPrChange></w:trPr><w:tc><w:p><w:{kind} w:id='3'><w:r><w:{leaf}>owned</w:{leaf}></w:r></w:{kind}></w:p></w:tc></w:tr></w:tbl></w:body></w:document>", W::URI));
        let root = dom.root(doc).unwrap();
        let before = canonical(&dom, root);
        mark_fully_revised_rows(&mut dom, root, &WmlComparerSettings::default(), &mut 10);
        assert_eq!(canonical(&dom, root), before);
    }
}

#[test]
fn explicit_row_lifetime_preserves_opposite_content_revisions_and_property_history() {
    for (row_kind, content_kind) in [("ins", "del"), ("del", "ins")] {
        let leaf = if content_kind == "del" {
            "delText"
        } else {
            "t"
        };
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&format!(
                "<w:document xmlns:w='{}'><w:body><w:tbl><w:tblPr><w:tblW w:w='2000' w:type='dxa'/></w:tblPr><w:tblGrid><w:gridCol w:w='2000'/></w:tblGrid><w:tr><w:trPr><w:cantSplit/><w:{row_kind} w:id='1' w:author='Row owner' w:date='2026-01-02T03:04:05Z'/><w:trPrChange w:id='2' w:author='Property owner' w:date='2026-01-02T03:04:05Z'><w:trPr><w:trHeight w:val='500'/></w:trPr></w:trPrChange></w:trPr><w:tc><w:tcPr><w:tcW w:w='2000' w:type='dxa'/><w:shd w:fill='123456'/><w:tcPrChange w:id='4' w:author='Property owner' w:date='2026-01-02T03:04:05Z'><w:tcPr><w:tcW w:w='1800' w:type='dxa'/></w:tcPr></w:tcPrChange></w:tcPr><w:p><w:pPr><w:spacing w:after='120'/><w:rPr><w:b/></w:rPr></w:pPr><w:{content_kind} w:id='3' w:author='Text owner' w:date='2026-01-02T03:04:05Z'><w:r><w:rPr><w:i/></w:rPr><w:{leaf}>owned</w:{leaf}></w:r></w:{content_kind}></w:p></w:tc></w:tr></w:tbl></w:body></w:document>",
                W::URI
            ));
        let root = dom.root(doc).unwrap();
        let before = canonical(&dom, root);
        let before_accept = project(&mut dom, root, true);
        let before_reject = project(&mut dom, root, false);
        let mut next_id = 10;
        mark_fully_revised_rows(
            &mut dom,
            root,
            &WmlComparerSettings::default(),
            &mut next_id,
        );
        assert_eq!(
            canonical(&dom, root),
            before,
            "row={row_kind}, content={content_kind}"
        );
        assert_eq!(
            next_id, 10,
            "explicit row owner consumes no new revision ID"
        );
        assert_eq!(project(&mut dom, root, true), before_accept);
        assert_eq!(project(&mut dom, root, false), before_reject);
        let row = dom.descendants(root, Some(&W::tr()))[0];
        let properties = dom.element(row, &W::tr_pr()).unwrap();
        assert!(dom.element(properties, &W::name(row_kind)).is_some());
        assert!(dom.element(properties, &W::name(content_kind)).is_none());
    }
}

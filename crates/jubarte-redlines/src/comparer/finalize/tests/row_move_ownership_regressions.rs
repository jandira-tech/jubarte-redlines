// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::mark_fully_revised_rows;
use crate::comparer::WmlComparerSettings;
use crate::namespaces::W;
use crate::revision_processor::{accept_revisions_document, reject_revisions_document};
use crate::xmllinq::Dom;

fn moved_row(direction: &str, mark: Option<&str>) -> String {
    let properties = mark.map_or_else(String::new, |mark| format!(
            "<w:pPr><w:rPr><w:{mark} w:id=\"9\" w:author=\"test\" w:date=\"1970-01-01T00:00:00Z\"/></w:rPr></w:pPr>"
        ));
    format!(
        "<w:tr><w:tc><w:p>{properties}<w:{direction}RangeStart w:id=\"1\" w:name=\"move1\"/><w:{direction} w:id=\"2\" w:author=\"test\" w:date=\"1970-01-01T00:00:00Z\"><w:r><w:t>alpha</w:t></w:r></w:{direction}><w:{direction}RangeEnd w:id=\"1\"/></w:p></w:tc></w:tr>"
    )
}

#[test]
fn moving_cell_content_without_paragraph_ownership_keeps_existing_rows() {
    for opposite_marks in [false, true] {
        let rows = moved_row("moveFrom", opposite_marks.then_some("ins"))
            + &moved_row("moveTo", opposite_marks.then_some("del"));
        let mut dom = Dom::new();
        let document = dom.parse_xdocument(&format!(
                "<w:document xmlns:w=\"{}\"><w:body><w:tbl><w:tblGrid><w:gridCol w:w=\"2000\"/></w:tblGrid>{rows}</w:tbl><w:sectPr/></w:body></w:document>", W::URI
            ));
        let root = dom.root(document).unwrap();
        mark_fully_revised_rows(&mut dom, root, &WmlComparerSettings::default(), &mut 10);
        assert!(dom.descendants(root, Some(&W::tr_pr())).is_empty());
        if !opposite_marks {
            for accept in [false, true] {
                let copy = dom.clone_subtree(root);
                let projected = if accept {
                    accept_revisions_document(&mut dom, copy)
                } else {
                    reject_revisions_document(&mut dom, copy)
                };
                let rows = dom.descendants(projected, Some(&W::tr()));
                assert_eq!(rows.len(), 2);
                let text = rows
                    .iter()
                    .map(|&row| {
                        dom.descendants(row, Some(&W::t()))
                            .into_iter()
                            .map(|text| dom.value(text))
                            .collect::<String>()
                    })
                    .collect::<Vec<_>>();
                assert_eq!(
                    text,
                    if accept {
                        vec!["", "alpha"]
                    } else {
                        vec!["alpha", ""]
                    }
                );
            }
        }
    }
}

#[test]
fn moved_content_with_matching_paragraph_mark_tracks_row_lifecycle() {
    for (direction, mark) in [("moveFrom", "del"), ("moveTo", "ins")] {
        let mut dom = Dom::new();
        let document = dom.parse_xdocument(&format!(
            "<w:document xmlns:w=\"{}\"><w:body><w:tbl>{}</w:tbl><w:sectPr/></w:body></w:document>",
            W::URI,
            moved_row(direction, Some(mark))
        ));
        let root = dom.root(document).unwrap();
        mark_fully_revised_rows(&mut dom, root, &WmlComparerSettings::default(), &mut 10);
        let row = dom.descendants(root, Some(&W::tr()))[0];
        let properties = dom.element(row, &W::tr_pr()).unwrap();
        assert!(dom.element(properties, &W::name(mark)).is_some());
    }
}

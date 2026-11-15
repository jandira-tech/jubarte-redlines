// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Pair 02 package path: table-bookmark-end vs table-vmerge-colspan from batch_to_fix.
use jubarte::document_comparer::compare_documents;
use std::io::{Cursor, Read};

fn read_part(docx: &[u8], name: &str) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name(name).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

#[test]
fn pair02_batch_docx_first_table_mixes_cells() {
    let a = std::fs::read(
        "tests/corpus/batch_to_fix/pairs/02_table_bookmark_end_table_vmerge_colspan/base.docx",
    )
    .expect("base");
    let b = std::fs::read(
        "tests/corpus/batch_to_fix/pairs/02_table_bookmark_end_table_vmerge_colspan/next.docx",
    )
    .expect("next");
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = read_part(&out, "word/document.xml");
    use jubarte::namespaces::W;
    use jubarte::xmllinq::Dom;
    let mut dom = Dom::new();
    let document = dom.parse_xdocument(&doc);
    let root = dom.root(document).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    let table = dom.elements(body, Some(&W::tbl()))[0];
    let row = dom.elements(table, Some(&W::tr()))[0];
    let cell = dom.elements(row, Some(&W::tc()))[0];
    let inserted: String = dom
        .descendants(cell, Some(&W::ins()))
        .into_iter()
        .flat_map(|revision| dom.descendants(revision, Some(&W::t())))
        .map(|text| dom.value(text))
        .collect();
    let deleted: String = dom
        .descendants(cell, Some(&W::del()))
        .into_iter()
        .flat_map(|revision| dom.descendants(revision, Some(&W::del_text())))
        .map(|text| dom.value(text))
        .collect();
    assert_eq!(
        inserted, "AAA",
        "Word inserts AAA inside the first physical cell"
    );
    assert_eq!(
        deleted, "R1C1",
        "Word deletes R1C1 in that same physical cell"
    );
    assert!(!doc.contains("WordTableMeshContext"));
}

fn geometry(package: &[u8]) -> Vec<Vec<Vec<(String, String, String)>>> {
    use jubarte::namespaces::W;
    use jubarte::xmllinq::Dom;
    let mut dom = Dom::new();
    let document = dom.parse_xdocument(&read_part(package, "word/document.xml"));
    let root = dom.root(document).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    dom.elements(body, Some(&W::tbl()))
        .into_iter()
        .map(|table| {
            dom.elements(table, Some(&W::tr()))
                .into_iter()
                .map(|row| {
                    dom.elements(row, Some(&W::tc()))
                        .into_iter()
                        .map(|cell| {
                            let properties = dom.element(cell, &W::tc_pr());
                            let span = properties
                                .and_then(|properties| dom.element(properties, &W::grid_span()))
                                .and_then(|span| dom.attribute(span, &W::val()))
                                .unwrap_or("1")
                                .to_string();
                            let merge = properties
                                .and_then(|properties| dom.element(properties, &W::name("vMerge")))
                                .map(|merge| dom.attribute(merge, &W::val()).unwrap_or("continue"))
                                .unwrap_or("")
                                .to_string();
                            let text = dom
                                .descendants(cell, Some(&W::t()))
                                .into_iter()
                                .map(|text| dom.value(text))
                                .collect();
                            (span, merge, text)
                        })
                        .collect()
                })
                .collect()
        })
        .collect()
}

#[test]
fn pair02_word_cell_mesh_retains_phantoms_and_faithful_restores_authored_partitions() {
    use jubarte::comparer::WmlComparerSettings;
    use jubarte::document_comparer::{
        accept_revisions, compare_documents_with_settings, reject_revisions,
    };
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/corpus/batch_to_fix/pairs/02_table_bookmark_end_table_vmerge_colspan");
    let a = std::fs::read(source.join("base.docx")).unwrap();
    let b = std::fs::read(source.join("next.docx")).unwrap();
    let original = geometry(&accept_revisions(&a).unwrap());
    let revised = geometry(&accept_revisions(&b).unwrap());
    for settings in [
        WmlComparerSettings::default(),
        WmlComparerSettings::powertools_faithful(),
    ] {
        let compared = compare_documents_with_settings(&a, &b, &settings).unwrap();
        let accepted = geometry(&accept_revisions(&compared).unwrap());
        let rejected = geometry(&reject_revisions(&compared).unwrap());
        assert_eq!(
            rejected, original,
            "original table text and authored spans remain recoverable"
        );
        if settings.merge_replaced_paragraphs {
            // Saved pair02 Word keeps three physical cells in each paired row,
            // including empty cells on acceptance; only extra rows are inserted.
            assert_eq!(
                accepted[0].iter().map(Vec::len).collect::<Vec<_>>(),
                [3, 3, 3, 2, 2]
            );
            let cell_texts: Vec<Vec<&str>> = accepted[0]
                .iter()
                .map(|row| row.iter().map(|(_, _, text)| text.as_str()).collect())
                .collect();
            assert_eq!(
                cell_texts,
                vec![
                    vec!["AAA", "", ""],
                    vec!["BBB", "CCC", ""],
                    vec!["", "DDD", ""],
                    vec!["EEE", "FFF"],
                    vec!["GGG", ""]
                ]
            );
        } else {
            assert_eq!(
                accepted, revised,
                "faithful mode recovers every authored cell and span"
            );
        }
    }
}

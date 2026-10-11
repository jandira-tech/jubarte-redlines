// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M337 — both-table short next with ≥2 tables (pirates×table_left): full LCS
//! recovers Word's shape, the new title inserted whole, then the old log
//! deleted, then the new body (IDDD…IIII). Wholesale pure-I/D was ~41.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

fn body_para_classes(xml: &str) -> Vec<char> {
    let mut out = Vec::new();
    let mut rest = xml;
    if let Some(i) = rest.find("<w:body") {
        rest = &rest[i..];
    }
    if let Some(i) = rest.find("</w:body>") {
        rest = &rest[..i];
    }
    while let Some(start) = rest.find("<w:p") {
        let after = &rest[start..];
        let end_rel = after
            .find("</w:p>")
            .map(|j| j + "</w:p>".len())
            .or_else(|| after.find("/>").map(|j| j + 2));
        let Some(end_rel) = end_rel else { break };
        let p = &after[..end_rel];
        rest = &after[end_rel..];
        let has_ins = p.contains("<w:ins");
        let has_del = p.contains("<w:del");
        out.push(match (has_ins, has_del) {
            (true, true) => 'M',
            (true, false) => 'I',
            (false, true) => 'D',
            (false, false) => 'E',
        });
    }
    out
}

#[test]
fn pirates_x_table_left_not_wholesale_pure_id() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src =
        root.join("tests/corpus/neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__sd_2766_pirates_tracked_changes_3285d875.docx");
    let b = src.join("super_editor__sd_1494_table_left_indent_11bb24c7.docx");
    if !a.exists() || !b.exists() {
        eprintln!("skip: fixtures missing");
        return;
    }
    let out = compare_documents_with_settings(
        &std::fs::read(&a).unwrap(),
        &std::fs::read(&b).unwrap(),
        &WmlComparerSettings {
            author_for_revisions: "Redline".into(),
            merge_replaced_paragraphs: true,
            ..WmlComparerSettings::default()
        },
    )
    .expect("compare");
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut xml = String::new();
    f.read_to_string(&mut xml).unwrap();
    let cls = body_para_classes(&xml);
    let seq: String = cls.iter().collect();
    let n_m = cls.iter().filter(|&&c| c == 'M').count();
    let n_i = cls.iter().filter(|&&c| c == 'I').count();
    let n_d = cls.iter().filter(|&&c| c == 'D').count();
    // Wholesale pure-I/D is I-block then D-block (I≥14 D≥25 MIX=0). Word
    // inserts the new title whole before the old one's deletion, then the
    // new body after the old log: I, 28 D, 9 I, no mixed paragraph. Fusing
    // the title into "A Simple Captain's Log" (MDDD…) is no Word shape.
    assert!(
        n_m == 0 && n_i <= 12 && n_d >= 20,
        "Word shape IDDD…IIII; got MIX={n_m} I={n_i} D={n_d} seq={seq}"
    );
    assert!(
        seq.starts_with("ID"),
        "expected the title inserted whole, then Ds; got {seq}"
    );
}

#[test]
fn hyperlink_x_rtl_still_pure_id() {
    // Single-table next stays on M313 pure-I/D (MIX=0).
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src =
        root.join("tests/corpus/neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__superdoc_hyperlink_cases_1dde9cd3.docx");
    let b = src.join("behavior__sd_2672_rtl_table_63bd9d10.docx");
    if !a.exists() || !b.exists() {
        eprintln!("skip: fixtures missing");
        return;
    }
    let out = compare_documents_with_settings(
        &std::fs::read(&a).unwrap(),
        &std::fs::read(&b).unwrap(),
        &WmlComparerSettings {
            author_for_revisions: "Redline".into(),
            merge_replaced_paragraphs: true,
            ..WmlComparerSettings::default()
        },
    )
    .expect("compare");
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut xml = String::new();
    f.read_to_string(&mut xml).unwrap();
    let cls = body_para_classes(&xml);
    let n_m = cls.iter().filter(|&&c| c == 'M').count();
    assert_eq!(
        n_m, 0,
        "single-table next stays pure-I/D MIX=0; got MIX={n_m}"
    );
}

/// Capture physical source cells, their live spans, and their paragraph text.
/// Revision projection uses the public processor so row lifetimes count too.
fn table_geometry(docx: &[u8]) -> Vec<Vec<Vec<(String, String)>>> {
    use jubarte::namespaces::W;
    use jubarte::opc::PartFs;
    use jubarte::xmllinq::Dom;
    let package = PartFs::open(docx).unwrap();
    let main = package.main_document_part().unwrap();
    let mut dom = Dom::new();
    let document = dom.parse_xdocument(&package.part_string(&main).unwrap());
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
                            let span = dom
                                .element(cell, &W::tc_pr())
                                .and_then(|properties| dom.element(properties, &W::grid_span()))
                                .and_then(|span| dom.attribute(span, &W::val()))
                                .unwrap_or("1")
                                .to_string();
                            let text = dom
                                .descendants(cell, Some(&W::t()))
                                .into_iter()
                                .map(|text| dom.value(text))
                                .collect();
                            (span, text)
                        })
                        .collect()
                })
                .collect()
        })
        .collect()
}

#[test]
fn m337_word_retains_observed_extra_empty_cell_while_faithful_restores_source_cells() {
    use jubarte::document_comparer::{accept_revisions, reject_revisions};
    use jubarte::opc::PartFs;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let source =
        root.join("tests/corpus/neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = source.join("super_editor__sd_2766_pirates_tracked_changes_3285d875.docx");
    let b = source.join("super_editor__sd_1494_table_left_indent_11bb24c7.docx");
    if !a.exists() || !b.exists() {
        eprintln!("skip: fixtures missing");
        return;
    }
    let a = std::fs::read(a).unwrap();
    let b = std::fs::read(b).unwrap();
    let original = table_geometry(&accept_revisions(&a).unwrap());
    let revised = table_geometry(&accept_revisions(&b).unwrap());
    let counts = |tables: &[Vec<Vec<(String, String)>>]| -> Vec<Vec<usize>> {
        tables
            .iter()
            .map(|table| table.iter().map(Vec::len).collect())
            .collect()
    };
    assert_eq!(counts(&original), vec![vec![4, 4, 4, 4, 4]]);
    assert_eq!(counts(&revised), vec![vec![3, 3], vec![3, 3]]);
    for settings in [
        WmlComparerSettings::default(),
        WmlComparerSettings::powertools_faithful(),
    ] {
        let output = compare_documents_with_settings(&a, &b, &settings).unwrap();
        let package = PartFs::open(&output).unwrap();
        let xml = package
            .part_string(&package.main_document_part().unwrap())
            .unwrap();
        assert!(
            !xml.contains("WordTableMeshContext"),
            "private context must be removed from saved output"
        );
        assert_eq!(
            table_geometry(&reject_revisions(&output).unwrap()),
            original
        );
        let accepted = table_geometry(&accept_revisions(&output).unwrap());
        if settings.merge_replaced_paragraphs {
            // Actual Word artifact 737a91c72f keeps four empty cells in the
            // first table. That quirk is confined to the M337 Word route.
            assert_eq!(counts(&accepted), vec![vec![4, 4], vec![3, 3]]);
            assert!(
                accepted[0]
                    .iter()
                    .flatten()
                    .all(|(_, text)| text.is_empty())
            );
            assert_eq!(accepted[1], revised[1]);
            let classes = body_para_classes(&xml);
            assert_eq!(classes.iter().filter(|&&class| class == 'M').count(), 0);
            assert!(classes.iter().filter(|&&class| class == 'I').count() <= 12);
            assert!(classes.iter().collect::<String>().starts_with("ID"));
        } else {
            assert_eq!(
                accepted, revised,
                "faithful mode restores every authored cell"
            );
        }
    }
}

#[test]
fn observed_word_table_mesh_families_keep_their_shape_and_faithful_source_geometry() {
    use jubarte::document_comparer::{accept_revisions, reject_revisions};
    use jubarte::opc::PartFs;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cases = [
        (
            "word_redlines_superdoc",
            "super_editor__sd_2766_pirates_tracked_changes_3285d875.docx",
            "behavior__sd_2343_table_border_widths_b5148e83.docx",
            vec![
                vec![4],
                vec![2],
                vec![2],
                vec![2, 2],
                vec![2],
                vec![2],
                vec![2],
            ],
            vec![vec![4, 4, 4, 4, 4]],
        ),
        (
            "word_based",
            "eigenpal_docx_editor_suggesting_mixed_edits.docx",
            "employee_directory_table_2.docx",
            vec![vec![3, 3, 3, 3]],
            vec![
                vec![3],
                vec![1],
                vec![4, 4, 4, 4, 4, 4, 4, 4],
                vec![1],
                vec![1],
                vec![2],
            ],
        ),
        (
            "word_redlines_superdoc",
            "super_editor__ooxml_rFonts_rstyle_linked_combos_dem_213298de.docx",
            "behavior__sd_2672_rtl_table_63bd9d10.docx",
            vec![vec![3, 3]],
            vec![vec![3, 3, 2]],
        ),
    ];
    let counts = |tables: &[Vec<Vec<(String, String)>>]| -> Vec<Vec<usize>> {
        tables
            .iter()
            .map(|table| table.iter().map(Vec::len).collect())
            .collect()
    };
    for (folder, a, b, word_accepted, word_rejected) in cases {
        let source = root
            .join("tests/corpus/neurotic_docx_bench/corpus")
            .join(folder)
            .join("docx_source");
        if !source.join(a).exists() || !source.join(b).exists() {
            eprintln!("skip: {a} × {b} fixtures missing");
            continue;
        }
        let original = std::fs::read(source.join(a)).unwrap();
        let revised = std::fs::read(source.join(b)).unwrap();
        let a_geometry = table_geometry(&accept_revisions(&original).unwrap());
        let b_geometry = table_geometry(&accept_revisions(&revised).unwrap());
        for settings in [
            WmlComparerSettings::default(),
            WmlComparerSettings::powertools_faithful(),
        ] {
            let output = compare_documents_with_settings(&original, &revised, &settings).unwrap();
            let package = PartFs::open(&output).unwrap();
            let xml = package
                .part_string(&package.main_document_part().unwrap())
                .unwrap();
            assert!(!xml.contains("WordTableMeshContext"));
            let accepted = table_geometry(&accept_revisions(&output).unwrap());
            let rejected = table_geometry(&reject_revisions(&output).unwrap());
            if settings.merge_replaced_paragraphs {
                assert_eq!(
                    counts(&accepted),
                    word_accepted,
                    "{a} × {b}: observed Word acceptance shape"
                );
                assert_eq!(
                    counts(&rejected),
                    word_rejected,
                    "{a} × {b}: observed Word rejection shape"
                );
                assert!(
                    body_para_classes(&xml).contains(&'M'),
                    "{a} × {b}: Word keeps mixed cell text"
                );
            } else {
                assert_eq!(
                    accepted, b_geometry,
                    "{a} × {b}: authored revised cells and text"
                );
                assert_eq!(
                    rejected, a_geometry,
                    "{a} × {b}: authored original cells and text"
                );
            }
        }
    }
}

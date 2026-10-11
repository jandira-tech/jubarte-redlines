// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Public source ownership oracles for the established M394/M424 Word splices.
mod common;

use common::docx::part_string;
use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::{
    accept_revisions, compare_documents_with_settings, reject_revisions,
};
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

type Payload = (
    String,
    Vec<(String, Option<String>, Option<String>)>,
    Vec<Vec<usize>>,
);

fn payload(bytes: &[u8]) -> Payload {
    let xml = part_string(bytes, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    let text = dom
        .descendants(root, Some(&W::t()))
        .into_iter()
        .map(|n| dom.value(n))
        .collect();
    let controls = dom
        .descendants(root, None)
        .into_iter()
        .filter(|&n| {
            ["tab", "br", "cr", "noBreakHyphen", "softHyphen"]
                .iter()
                .any(|name| dom.name_is(n, &W::name(name)))
        })
        .map(|n| {
            (
                dom.name(n).unwrap().local_name().to_string(),
                dom.attribute(n, &W::name("type")).map(str::to_string),
                dom.attribute(n, &W::name("clear")).map(str::to_string),
            )
        })
        .collect();
    let geometry = dom
        .descendants(root, Some(&W::tbl()))
        .into_iter()
        .map(|table| {
            dom.elements(table, Some(&W::name("tr")))
                .into_iter()
                .map(|row| dom.elements(row, Some(&W::name("tc"))).len())
                .collect()
        })
        .collect();
    (text, controls, geometry)
}

#[test]
fn established_word_midsplices_recover_every_source_payload_in_order() {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/corpus/neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    for (original, revised) in [
        (
            "evals__employment_offer_4cf5a872.docx",
            "evals__lease_agreement_7081191d.docx",
        ),
        (
            "behavior__sd_2517_localized_heading_styles_39c2e4a1.docx",
            "behavior__sd_2672_gridbefore_vmerge_7c895dff.docx",
        ),
    ] {
        let a_path = directory.join(original);
        let b_path = directory.join(revised);
        if !a_path.exists() || !b_path.exists() {
            eprintln!("skip: fixtures missing {original}/{revised}");
            continue;
        }
        let a = std::fs::read(a_path).unwrap();
        let b = std::fs::read(b_path).unwrap();
        let compared = compare_documents_with_settings(
            &a,
            &b,
            &WmlComparerSettings {
                merge_replaced_paragraphs: true,
                ..WmlComparerSettings::default()
            },
        )
        .unwrap();
        assert_eq!(
            payload(&reject_revisions(&compared).unwrap()),
            payload(&a),
            "original ownership: {original}"
        );
        assert_eq!(
            payload(&accept_revisions(&compared).unwrap()),
            payload(&b),
            "revised ownership: {revised}"
        );
    }
}

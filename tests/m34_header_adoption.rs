// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M-HDR — per-slot header/footer adoption (word-alignment mode).
//!
//! GT evidence pair page-numbering-examples (A) vs potpourritest (B): A has
//! only footer1 (default); B has headers 1–3 + footers 1–3 (even/default/
//! first). Word's redline package carries SIX header/footer parts: A's footer
//! slot is content-diffed, and B's parts fill every (kind, w:type) slot A
//! lacks. The old rule ("adopt B's set only when A has NO refs at all")
//! dropped all of B's parts here. The new rule is a UNION per (kind, type)
//! slot: keep A's existing refs/parts untouched; adopt B's refs for slots the
//! output's FINAL sectPr lacks. "Absent" is judged on the final sectPr's
//! EXPLICIT refs only (matching the existing function's behavior) — OOXML
//! slot inheritance from earlier sections is deliberately not modeled here.

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;
use jubarte::namespaces::{R, W};
use jubarte::opc::PartFs;
use jubarte::xmllinq::Dom;

const R_URI: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

fn word_mode() -> WmlComparerSettings {
    WmlComparerSettings {
        author_for_revisions: "Redline".into(),
        date_time_for_revisions: "2020-01-01T00:00:00Z".into(),
        detail_threshold: 0.0,
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    }
}

fn hf_part(kind: &str, text: &str) -> Vec<u8> {
    let root = if kind == "header" { "w:hdr" } else { "w:ftr" };
    format!(
        "<{root} xmlns:w=\"{w}\"><w:p><w:r><w:t>{text}</w:t></w:r></w:p></{root}>",
        w = W::URI
    )
    .into_bytes()
}

/// doc A: body + a single default footer (footer1.xml).
fn doc_a(base: &[u8]) -> Vec<u8> {
    let mut p = PartFs::open(base).unwrap();
    p.set_part("word/footer1.xml", hf_part("footer", "FooterAlpha"));
    p.add_content_type_override(
        "/word/footer1.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml",
    );
    let rid = p.add_document_relationship(
        "word/document.xml",
        &format!("{R_URI}/footer"),
        "footer1.xml",
    );
    p.set_part(
        "word/document.xml",
        format!(
            "<w:document xmlns:w=\"{w}\" xmlns:r=\"{r}\"><w:body>\
             <w:p><w:r><w:t>shared body text</w:t></w:r></w:p>\
             <w:sectPr><w:footerReference w:type=\"default\" r:id=\"{rid}\"/>\
             <w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr>\
             </w:body></w:document>",
            w = W::URI,
            r = R_URI,
        )
        .into_bytes(),
    );
    p.to_zip().unwrap()
}

/// doc B: body + headers 1–3 and footers 1–3 covering even/default/first.
fn doc_b(base: &[u8]) -> Vec<u8> {
    let mut p = PartFs::open(base).unwrap();
    let slots = [
        ("header", "even", "header1.xml", "HdrEvenBravo"),
        ("header", "default", "header2.xml", "HdrDefaultBravo"),
        ("header", "first", "header3.xml", "HdrFirstBravo"),
        ("footer", "even", "footer1.xml", "FtrEvenBravo"),
        ("footer", "default", "footer2.xml", "FtrDefaultBravo"),
        ("footer", "first", "footer3.xml", "FtrFirstBravo"),
    ];
    let mut refs = String::new();
    for (kind, ty, file, text) in slots {
        p.set_part(&format!("word/{file}"), hf_part(kind, text));
        p.add_content_type_override(
            &format!("/word/{file}"),
            &format!("application/vnd.openxmlformats-officedocument.wordprocessingml.{kind}+xml"),
        );
        let rid =
            p.add_document_relationship("word/document.xml", &format!("{R_URI}/{kind}"), file);
        refs.push_str(&format!(
            "<w:{kind}Reference w:type=\"{ty}\" r:id=\"{rid}\"/>"
        ));
    }
    p.set_part(
        "word/document.xml",
        format!(
            "<w:document xmlns:w=\"{w}\" xmlns:r=\"{r}\"><w:body>\
             <w:p><w:r><w:t>shared body text</w:t></w:r></w:p>\
             <w:sectPr>{refs}<w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr>\
             </w:body></w:document>",
            w = W::URI,
            r = R_URI,
        )
        .into_bytes(),
    );
    p.to_zip().unwrap()
}

/// The union rule: A's footer-default slot survives (content-diffable), and
/// B fills the five missing slots; every ref resolves to a real part.
#[test]
fn per_slot_union_adopts_bs_missing_header_footer_slots() {
    let base = std::fs::read("tests/fixtures/redline/original.docx").unwrap();
    let out = compare_documents_with_settings(&doc_a(&base), &doc_b(&base), &word_mode()).unwrap();
    let pkg = PartFs::open(&out).unwrap();

    let dx = pkg.part_string("word/document.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&dx);
    let root = dom.root(doc).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    let sect = dom.element(body, &W::name("sectPr")).expect("final sectPr");

    let href = W::name("headerReference");
    let fref = W::name("footerReference");
    let rels = pkg.read_rels_for("word/document.xml").unwrap();

    // collect (kind, type) → resolved part text of every ref in the sectPr,
    // asserting each rId resolves to an existing part along the way
    let mut slot_text = std::collections::HashMap::new();
    for e in dom.elements(sect, None) {
        let Some(name) = dom.name(e) else { continue };
        let kind = if name == href {
            "header"
        } else if name == fref {
            "footer"
        } else {
            continue;
        };
        let ty = dom
            .attribute(e, &W::name("type"))
            .unwrap_or("default")
            .to_string();
        let rid = dom
            .attribute(e, &R::name("id"))
            .unwrap_or_else(|| panic!("{kind}/{ty} ref missing r:id"));
        let rel =
            rels.items.iter().find(|r| r.id == rid).unwrap_or_else(|| {
                panic!("rId {rid} ({kind}/{ty}) missing from document.xml.rels")
            });
        let part = pkg.resolve_rel_target("word/document.xml", &rel.target);
        let text = pkg
            .part_string(&part)
            .unwrap_or_else(|| panic!("{kind}/{ty} target part {part} missing from package"));
        slot_text.insert((kind.to_string(), ty), text);
    }

    // all six (kind, type) slots referenced from the final sectPr
    for (kind, ty) in [
        ("header", "even"),
        ("header", "default"),
        ("header", "first"),
        ("footer", "even"),
        ("footer", "default"),
        ("footer", "first"),
    ] {
        assert!(
            slot_text.contains_key(&(kind.to_string(), ty.to_string())),
            "final sectPr must reference {kind}/{ty}; got slots: {:?}",
            slot_text.keys().collect::<Vec<_>>()
        );
    }

    // A's footer-default slot is KEPT (A's content, not replaced by B's part)
    let fd = &slot_text[&("footer".to_string(), "default".to_string())];
    assert!(
        fd.contains("FooterAlpha"),
        "footer/default must keep doc A's (diffed) footer, got: {fd}"
    );

    // the five missing slots carry B's content
    for (kind, ty, text) in [
        ("header", "even", "HdrEvenBravo"),
        ("header", "default", "HdrDefaultBravo"),
        ("header", "first", "HdrFirstBravo"),
        ("footer", "even", "FtrEvenBravo"),
        ("footer", "first", "FtrFirstBravo"),
    ] {
        let got = &slot_text[&(kind.to_string(), ty.to_string())];
        assert!(
            got.contains(text),
            "{kind}/{ty} must carry doc B's part ({text}), got: {got}"
        );
    }

    // and doc A's original footer part bytes were not clobbered by B's
    // same-named footer1.xml (B's footer/even) — B must land elsewhere
    let a_footer = pkg.part_string("word/footer1.xml").unwrap();
    assert!(
        a_footer.contains("FooterAlpha"),
        "A's word/footer1.xml must not be clobbered: {a_footer}"
    );
}

/// M-PAG mechanism 1 (GT-verified on sd-2517 vs sectpr-headerref): for every
/// (kind, type) slot doc A populates, the redline RETAINS A's part content —
/// Word keeps A's footer paragraphs per slot. B's parts fill only slots A
/// lacks. The prior M4.H.x content-diff degenerated when B's matched part is
/// a single run-less paragraph: the "diff" came out as B's wholesale content
/// with no revision markup, blanking all 19 of A's footers (−3 rendered
/// pages: the 3-line footer's body-shrink physics).
#[test]
fn matched_footer_slots_retain_doc_as_content() {
    let a_path = std::path::Path::new(
        "tests/corpus/_fixtures/original_fixtures/sd-2517-localized-heading-styles.docx",
    );
    let b_path =
        std::path::Path::new("tests/corpus/_fixtures/original_fixtures/sectpr-headerref.docx");
    if !a_path.is_file() || !b_path.is_file() {
        eprintln!("SKIP: _fixtures/original_fixtures corpus not present");
        return;
    }
    let a = std::fs::read(a_path).unwrap();
    let b = std::fs::read(b_path).unwrap();
    let out = compare_documents_with_settings(&a, &b, &word_mode()).unwrap();
    let pkg = PartFs::open(&out).unwrap();

    // B's single footer is one empty paragraph; capture its inner content
    // (between the root tags) as the degenerate signature.
    let b_pkg = PartFs::open(&b).unwrap();
    let b_footer = b_pkg.part_string("word/footer1.xml").unwrap();
    let inner = |x: &str| -> String {
        let start = x
            .find("<w:ftr")
            .and_then(|i| x[i..].find('>').map(|j| i + j + 1));
        let end = x.rfind("</w:ftr>");
        match (start, end) {
            (Some(s), Some(e)) if s <= e => x[s..e].split_whitespace().collect(),
            _ => String::new(),
        }
    };
    let b_inner = inner(&b_footer);

    let footers: Vec<String> = pkg
        .parts()
        .into_iter()
        .filter(|p| p.starts_with("word/footer") && p.ends_with(".xml"))
        .collect();
    assert!(!footers.is_empty(), "output must carry footer parts");

    let mut with_a_text = 0usize;
    for part in &footers {
        let x = pkg.part_string(part).unwrap();
        assert_ne!(
            inner(&x),
            b_inner,
            "{part} must not be B's wholesale empty-paragraph footer — A populates this slot"
        );
        if x.contains("Smith Family Trust") {
            with_a_text += 1;
        }
    }
    assert!(
        with_a_text > 0,
        "at least one footer part must retain A's 'Smith Family Trust' content; \
         {} footer parts, none had it",
        footers.len()
    );
}

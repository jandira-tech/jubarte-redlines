// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Parity-surface regression for the AST report
//! (`jubarte_family/LOSSLESS_LINQ_PARITY_REPORT.md`).
//!
//! These tests drive the **shipped** jubarte-rs entry points the report
//! classifies as **present** for linqtoxml + lossless redline counterparts.
//! They are structural/API proofs, not full golden redline parity.

use jubarte::comparer::{self, WmlComparerSettings};
use jubarte::comparison_log::{ComparisonLog, ComparisonLogCode};
use jubarte::document_comparer;
use jubarte::markup_simplifier;
use jubarte::namespaces::W;
use jubarte::opc::PartFs;
use jubarte::revision_processor;
use jubarte::strict_translation;
use jubarte::unid;
use jubarte::util::{self, group_adjacent};
use jubarte::wml_document::WmlDocument;
use jubarte::xmllinq::{Dom, XName, XNamespace};

const ORIGINAL: &[u8] = include_bytes!("fixtures/redline/original.docx");
const MODIFIED: &[u8] = include_bytes!("fixtures/redline/modified.docx");

/// Report claim: XNamespace / XName / Dom / parse / serialize / hash-serialize are present.
#[test]
fn linq_surface_parse_mutate_serialize_hash() {
    let ns = XNamespace::get("http://example.com/ns");
    assert_eq!(ns.namespace_name(), "http://example.com/ns");
    assert_eq!(XNamespace::none().namespace_name(), "");
    assert!(XNamespace::xmlns().namespace_name().contains("xmlns"));
    assert!(XNamespace::xml().namespace_name().contains("XML"));

    let name = XName::get("root", "http://example.com/ns");
    assert_eq!(name.local_name(), "root");
    assert_eq!(name.clark(), "{http://example.com/ns}root");
    let from_clark = XName::from_clark("{http://example.com/ns}root");
    assert_eq!(from_clark, name);

    let xml = r#"<?xml version="1.0" encoding="UTF-8"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Hi</w:t></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = jubarte::xmllinq::parse_xdocument(&mut dom, xml);
    assert!(dom.is_document(doc));
    let root = dom.root(doc).expect("root");
    assert!(dom.is_element(root));
    let root_name = dom.name(root).expect("name");
    assert_eq!(root_name.local_name(), "document");

    let t_name = W::name("t");
    let texts = dom.descendants(root, Some(&t_name));
    assert_eq!(texts.len(), 1);
    assert_eq!(dom.value(texts[0]), "Hi");

    // Mutation surface used by the comparer ports
    let p = dom.descendants(root, Some(&W::name("p")))[0];
    dom.set_attribute_value(p, &W::name("rsidR"), Some("00AB12"));
    assert_eq!(dom.attribute(p, &W::name("rsidR")), Some("00AB12"));
    let cloned = dom.clone_subtree(p);
    assert_eq!(dom.parent(cloned), None);

    let ser = jubarte::xmllinq::serialize_element(&dom, root);
    assert!(ser.contains("<w:document"));
    assert!(ser.contains("Hi"));

    let hash = jubarte::xmllinq::serialize_element_sha1_hex(&dom, root);
    assert_eq!(hash.len(), 40);
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));

    let struct_hash = jubarte::xmllinq::serialize_element_structure_sha1_hex(&dom, root);
    assert_eq!(struct_hash.len(), 40);

    let doc_ser = jubarte::xmllinq::serialize_document(&dom, doc);
    assert!(doc_ser.contains("<?xml") || doc_ser.contains("<w:document"));
}

/// Report claim: BOM-safe parse is present on RS (better-on-RS vs TS inventory).
#[test]
fn linq_bom_strip_on_parse() {
    let with_bom = "\u{feff}<root><a/></root>";
    let mut dom = Dom::new();
    let doc = jubarte::xmllinq::parse_xdocument(&mut dom, with_bom);
    let root = dom.root(doc).expect("root after BOM strip");
    let root_name = dom.name(root).expect("name");
    assert_eq!(root_name.local_name(), "root");
}

/// Report claim: body compare + settings presets are present.
#[test]
fn comparer_body_and_settings_surface() {
    let mut dom = Dom::new();
    let xml = r#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>A</w:t></w:r></w:p></w:body></w:document>"#;
    let doc1 = jubarte::xmllinq::parse_xdocument(&mut dom, xml);
    let doc2 = jubarte::xmllinq::parse_xdocument(
        &mut dom,
        r#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>B</w:t></w:r></w:p></w:body></w:document>"#,
    );
    let root1 = dom.root(doc1).unwrap();
    let root2 = dom.root(doc2).unwrap();
    let body1 = dom.element(root1, &W::name("body")).expect("body1");
    let body2 = dom.element(root2, &W::name("body")).expect("body2");

    let settings = WmlComparerSettings::default();
    let out = comparer::compare_bodies_faithful(&mut dom, root1, root2, body1, body2, &settings);
    assert!(dom.is_element(out));
    let out_name = dom.name(out).expect("out name");
    assert_eq!(out_name.local_name(), "document");

    let _pt = WmlComparerSettings::powertools_faithful();
}

/// Report claim: document_comparer package API is present (compare/accept/reject/get_revisions).
#[test]
fn document_comparer_package_surface() {
    let redline = document_comparer::compare_documents(ORIGINAL, MODIFIED, "Parity Author")
        .expect("compare_documents");
    assert!(!redline.is_empty());
    let pkg = PartFs::open(&redline).expect("open redline");
    let doc_xml = pkg
        .part_string(
            &pkg.main_document_part()
                .unwrap_or_else(|| "word/document.xml".into()),
        )
        .expect("document.xml");
    assert!(
        doc_xml.contains("<w:ins") || doc_xml.contains("<w:del"),
        "expected revision markup"
    );

    let accepted = document_comparer::accept_revisions(&redline).expect("accept");
    assert!(!accepted.is_empty());
    let rejected = document_comparer::reject_revisions(&redline).expect("reject");
    assert!(!rejected.is_empty());

    let settings = WmlComparerSettings::default();
    let revs = document_comparer::get_revisions(&redline, &settings).expect("get_revisions");
    // Fixtures differ — at least the API returns a list (may be non-empty).
    let _ = revs.len();
}

/// Report claim: markup_simplifier is the compare subset (single-char runs + rsid).
#[test]
fn markup_simplifier_compare_subset_surface() {
    let mut dom = Dom::new();
    let r = dom.new_element(W::r());
    let t = dom.new_element(W::t());
    dom.add_text(t, "ab");
    dom.add(r, t);
    let exploded = markup_simplifier::single_character_run_transform(&mut dom, r);
    assert!(exploded.len() >= 2, "one run per character");

    let rsid_el = dom.new_element(W::p());
    dom.set_attribute_value(rsid_el, &W::name("rsidR"), Some("00AA"));
    let cleaned = markup_simplifier::remove_rsid_transform(&mut dom, rsid_el);
    assert!(cleaned.is_some());
}

/// Report claim: unid assign / generate present; deterministic path intentionally absent.
#[test]
fn unid_surface() {
    let a = unid::generate_unid();
    let b = unid::generate_unid();
    assert_eq!(a.len(), 32);
    assert_eq!(b.len(), 32);
    assert_ne!(a, b);

    let mut dom = Dom::new();
    let p = dom.new_element(W::p());
    let r = dom.new_element(W::r());
    dom.add(p, r);
    unid::assign_to_all_elements(&mut dom, p);
    unid::assign_to_self_and_descendants(&mut dom, p);
}

/// Report claim: ComparisonLog severity API present (string codes incomplete).
#[test]
fn comparison_log_surface() {
    let mut log = ComparisonLog::new();
    log.info("i");
    log.warning("w");
    log.error("e");
    assert_eq!(log.error_count(), 1);
    assert_eq!(log.entries.len(), 3);
    assert_eq!(log.entries[0].code, ComparisonLogCode::Info);
}

/// Report claim: WmlDocument + PartFs + strict_to_transitional present.
#[test]
fn wml_opc_strict_surface() {
    let mut wml = WmlDocument::from_bytes(ORIGINAL).expect("from_bytes");
    let _ = wml.main_document_part_name();
    let root = wml.main_document_root().expect("main root");
    assert!(wml.dom().is_element(root));
    assert!(!wml.part_fs().parts().is_empty());

    let pkg = PartFs::open(ORIGINAL).expect("PartFs::open");
    assert!(pkg.main_document_part().is_some() || pkg.part_string("word/document.xml").is_some());

    let transitional = strict_translation::strict_to_transitional_docx(ORIGINAL);
    assert!(!transitional.is_empty());
    PartFs::open(&transitional).expect("strict output still a package");
}

/// Report claims (corrected):
/// - `element_has_tracked_revisions` ≡ PartHasTrackedRevisions on a tree (pub)
/// - `iterate_block_content_elements` ≡ IterateBlockContentElements + AnnotateBlockContentElements
/// - styles accept/reject run via `accept_revisions_package` (see document_comparer::accept_revisions)
#[test]
fn revision_processor_element_surface() {
    let mut dom = Dom::new();
    let xml = r#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:ins w:id="0" w:author="A" w:date="2020-01-01T00:00:00Z"><w:r><w:t>X</w:t></w:r></w:ins></w:p><w:p><w:r><w:t>Y</w:t></w:r></w:p></w:body></w:document>"#;
    let doc = jubarte::xmllinq::parse_xdocument(&mut dom, xml);
    let root = dom.root(doc).unwrap();
    // PartHasTrackedRevisions port (tree form)
    assert!(revision_processor::element_has_tracked_revisions(
        &dom, root
    ));

    let body = dom.element(root, &W::name("body")).expect("body");
    let chain = revision_processor::iterate_block_content_elements(&dom, body);
    assert!(
        chain.len() >= 2,
        "AnnotateBlockContentElements port must chain block content"
    );
    assert!(chain[0].this_block_content_element.is_some());
    assert_eq!(
        chain[0].next_block_content_element,
        chain[1].this_block_content_element
    );

    let accepted = revision_processor::accept_revisions_for_element(&mut dom, root);
    assert!(dom.is_element(accepted));
    assert!(!revision_processor::element_has_tracked_revisions(
        &dom, accepted
    ));
}

/// Package-level accept/reject (includes styles part transforms when styles exist).
#[test]
fn revision_processor_package_styles_path() {
    // document_comparer::accept_revisions / reject_revisions call
    // revision_processor::{accept,reject}_revisions_package, which branch on
    // is_styles → accept/reject_revisions_for_styles_transform.
    let redline =
        document_comparer::compare_documents(ORIGINAL, MODIFIED, "Styles Path").expect("compare");
    let accepted = document_comparer::accept_revisions(&redline).expect("accept package");
    assert!(!accepted.is_empty());
    PartFs::open(&accepted).expect("accepted still a package");
    let rejected = document_comparer::reject_revisions(&redline).expect("reject package");
    assert!(!rejected.is_empty());
    PartFs::open(&rejected).expect("rejected still a package");
}

/// Report claim: util sha1 + group_adjacent present (PtUtil subset).
#[test]
fn util_surface() {
    let h = util::sha1_hex("hello");
    assert_eq!(h.len(), 40);
    let groups = group_adjacent(vec![1, 1, 2, 2, 2, 3], |&x| x);
    assert_eq!(groups.len(), 3);
}

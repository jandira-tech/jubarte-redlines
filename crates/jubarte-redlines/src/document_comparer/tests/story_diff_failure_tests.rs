// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;

#[test]
fn neutralization_preserves_resolved_internal_references_and_source_package() {
    let mut pkg = PartFs::open(include_bytes!(
        "../../../../../tests/fixtures/redline/original.docx"
    ))
    .unwrap();
    let part = "word/header1.xml";
    pkg.set_part("word/media/owned.bin", vec![1, 2, 3]);
    let rid = pkg.add_document_relationship(part, &format!("{}/image", R::URI), "media/owned.bin");
    let valid = format!(
        "<w:hdr xmlns:w='{}' xmlns:r='{}'><w:p r:embed='{rid}'/></w:hdr>",
        W::URI,
        R::URI,
    );
    assert_eq!(
        neutralize_story_relationships(&pkg, part, &valid).unwrap(),
        valid
    );
    let mixed = valid.replace("<w:p ", "<w:p r:link='missing' ");
    let before = pkg.to_zip().unwrap();
    let neutralized = neutralize_story_relationships(&pkg, part, &mixed).unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&neutralized);
    let p = dom.element(dom.root(doc).unwrap(), &W::p()).unwrap();
    assert_eq!(dom.attribute(p, &R::name("embed")), Some(rid.as_str()));
    assert_eq!(dom.attribute(p, &R::name("link")), None);
    assert_eq!(pkg.to_zip().unwrap(), before);
}

#[test]
fn unmarked_story_loss_is_a_typed_refusal() {
    for (a, b, redline) in [("A", "B", "A"), ("A", "B", "B"), ("A", "A", "")] {
        let err = ensure_story_text_tracked("word/header1.xml", a, b, redline, false)
            .expect_err("neither silently keeping A nor replacing it with B is a redline");
        let OpcError::Io(io) = err else {
            panic!("expected a typed refusal");
        };
        let typed = io
            .get_ref()
            .unwrap()
            .downcast_ref::<StoryDiffError>()
            .unwrap();
        assert_eq!(typed.part, "word/header1.xml");
        assert_eq!(typed.reason, "story text changed without revision marks");
    }
    ensure_story_text_tracked("word/header1.xml", "A", "A", "A", false).unwrap();
    ensure_story_text_tracked("word/header1.xml", "A", "B", "AB", true).unwrap();
}

#[test]
fn rootless_story_cannot_be_neutralized_into_a_silent_copy() {
    let pkg = PartFs::open(include_bytes!(
        "../../../../../tests/fixtures/redline/original.docx"
    ))
    .unwrap();
    let err = neutralize_story_relationships(&pkg, "word/footer1.xml", "").unwrap_err();
    assert!(
        err.to_string()
            .contains("STORY_DIFF_FAILED: word/footer1.xml")
    );
}

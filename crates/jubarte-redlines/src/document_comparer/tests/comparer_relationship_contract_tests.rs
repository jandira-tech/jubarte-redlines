// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;
fn package() -> PartFs {
    PartFs::open(include_bytes!(
        "../../../../../tests/fixtures/redline/original.docx"
    ))
    .unwrap()
}
#[test]
fn referenced_relationship_agreement_requires_matching_type_mode_and_payload_or_external_target() {
    for (
        different_type,
        external_a,
        external_b,
        different_target,
        missing_a,
        missing_b,
        different_bytes,
        agrees,
    ) in [
        (false, false, false, false, false, false, false, true),
        (true, false, false, false, false, false, false, false),
        (false, false, true, false, false, false, false, false),
        (false, true, false, false, false, false, false, false),
        (false, true, true, false, false, false, false, true),
        (false, true, true, true, false, false, false, false),
        (false, false, false, true, false, false, false, true),
        (false, false, false, false, true, false, false, false),
        (false, false, false, false, false, true, false, false),
        (false, false, false, false, false, false, true, false),
    ] {
        let mut a = package();
        let mut b = package();
        let target_a = if external_a {
            "https://example.invalid/one"
        } else {
            "media/one.bin"
        };
        let target_b = if external_b {
            if different_target {
                "https://example.invalid/two"
            } else {
                "https://example.invalid/one"
            }
        } else if different_target {
            "media/two.bin"
        } else {
            "media/one.bin"
        };
        let ty_a = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image";
        let ty_b = if different_type {
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/oleObject"
        } else {
            ty_a
        };
        let id_a = if external_a {
            a.add_document_relationship_external("word/document.xml", ty_a, target_a)
        } else {
            a.add_document_relationship("word/document.xml", ty_a, target_a)
        };
        let id_b = if external_b {
            b.add_document_relationship_external("word/document.xml", ty_b, target_b)
        } else {
            b.add_document_relationship("word/document.xml", ty_b, target_b)
        };
        assert_eq!(id_a, id_b);
        if !external_a && !missing_a {
            a.set_part("word/media/one.bin", vec![1, 2, 3]);
        }
        if !external_b && !missing_b {
            b.set_part(
                if different_target {
                    "word/media/two.bin"
                } else {
                    "word/media/one.bin"
                },
                if different_bytes {
                    vec![3, 2, 1]
                } else {
                    vec![1, 2, 3]
                },
            );
        }
        let xml = format!(
            "<a:blip xmlns:a='http://schemas.openxmlformats.org/drawingml/2006/main' xmlns:r='{}' r:embed='{id_a}'/>",
            R::URI
        );
        let before_a = a.to_zip().unwrap();
        let before_b = b.to_zip().unwrap();
        assert_eq!(
            part_rels_agree((&a, "word/document.xml"), (&b, "word/document.xml"), &xml),
            agrees,
            "types={different_type} modes={external_a}/{external_b} targets={different_target} missing={missing_a}/{missing_b} bytes={different_bytes}"
        );
        assert_eq!(a.to_zip().unwrap(), before_a);
        assert_eq!(b.to_zip().unwrap(), before_b);
    }
}
#[test]
fn repeated_hyperlinks_share_one_carried_relationship_and_keep_independent_source_text_and_format()
{
    let mut a = package();
    let b = {
        let mut pkg = package();
        pkg.add_document_relationship_external(
            "word/header.xml",
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink",
            "https://example.invalid/owned",
        );
        pkg
    };
    let id = &b.read_rels_for("word/header.xml").unwrap().items[0].id;
    let xml = format!(
        "<w:hdr xmlns:w='{}' xmlns:r='{}'><w:p><w:hyperlink r:id='{id}'><w:r><w:rPr><w:b/></w:rPr><w:t>First</w:t></w:r></w:hyperlink><w:hyperlink r:id='{id}'><w:r><w:rPr><w:i/></w:rPr><w:t>Second</w:t></w:r></w:hyperlink></w:p></w:hdr>",
        W::URI,
        R::URI
    );
    let before = b.to_zip().unwrap();
    let output =
        carry_revised_part_relationships(&mut a, "word/header.xml", (&b, "word/header.xml"), &xml)
            .unwrap();
    let rels = &a.read_rels_for("word/header.xml").unwrap().items;
    assert_eq!(rels.len(), 1);
    assert_eq!(rels[0].target, "https://example.invalid/owned");
    assert_eq!(rels[0].target_mode.as_deref(), Some("External"));
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&output);
    let root = dom.root(doc).unwrap();
    let links = dom.descendants(root, Some(&W::hyperlink()));
    assert_eq!(links.len(), 2);
    for (i, node) in links.into_iter().enumerate() {
        assert_eq!(dom.attribute(node, &R::id()), Some(rels[0].id.as_str()));
        assert_eq!(dom.value(node), if i == 0 { "First" } else { "Second" });
        let pr = dom.descendants(node, Some(&W::r_pr()))[0];
        assert!(
            dom.element(pr, &W::name(if i == 0 { "b" } else { "i" }))
                .is_some()
        );
    }
    assert_eq!(b.to_zip().unwrap(), before);
}

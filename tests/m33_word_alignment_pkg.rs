//! Word-alignment, package level: Word's Compare presents the REVISED
//! document's headers/footers (evidence: comments_complex-style-attr — the
//! header exists only in doc B yet renders in Word's redline). When doc A has
//! no header/footer references and doc B's final sectPr does, adopt doc B's:
//! copy the parts (+ their rels/media) and reference them from the output.

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;
use jubarte::namespaces::W;
use jubarte::opc::PartFs;

fn word_mode() -> WmlComparerSettings {
    WmlComparerSettings {
        author_for_revisions: "Redline".into(),
        date_time_for_revisions: "2020-01-01T00:00:00Z".into(),
        detail_threshold: 0.0,
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    }
}

#[test]
fn w6_revised_doc_headers_adopted() {
    let base = std::fs::read("tests/fixtures/redline/original.docx").unwrap();

    // doc A: plain body, no headers
    let mut pa = PartFs::open(&base).unwrap();
    pa.set_part(
        "word/document.xml",
        format!(
            "<w:document xmlns:w=\"{w}\"><w:body><w:p><w:r><w:t>shared body text</w:t></w:r></w:p></w:body></w:document>",
            w = W::URI
        )
        .into_bytes(),
    );
    let a = pa.to_zip().unwrap();

    // doc B: same body + a header part referenced from the final sectPr
    let mut pb = PartFs::open(&base).unwrap();
    pb.set_part(
        "word/header1.xml",
        format!(
            "<w:hdr xmlns:w=\"{w}\"><w:p><w:r><w:t>REVISED HEADER TEXT</w:t></w:r></w:p></w:hdr>",
            w = W::URI
        )
        .into_bytes(),
    );
    pb.add_content_type_override(
        "/word/header1.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
    );
    let rid = pb.add_document_relationship(
        "word/document.xml",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
        "header1.xml",
    );
    pb.set_part(
        "word/document.xml",
        format!(
            "<w:document xmlns:w=\"{w}\" xmlns:r=\"{r}\"><w:body>\
             <w:p><w:r><w:t>shared body text</w:t></w:r></w:p>\
             <w:sectPr><w:headerReference w:type=\"default\" r:id=\"{rid}\"/>\
             <w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr>\
             </w:body></w:document>",
            w = W::URI,
            r = "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
        )
        .into_bytes(),
    );
    let b = pb.to_zip().unwrap();

    let out = compare_documents_with_settings(&a, &b, &word_mode()).unwrap();
    let pkg = PartFs::open(&out).unwrap();

    // the header part was copied and contains the revised text
    let hx = pkg
        .part_string("word/header1.xml")
        .expect("header part adopted from revised doc");
    assert!(hx.contains("REVISED HEADER TEXT"), "{hx}");

    // the output document references it via a RESOLVING rel
    let dx = pkg.part_string("word/document.xml").unwrap();
    assert!(
        dx.contains("headerReference"),
        "sectPr references the adopted header: {dx}"
    );
    let rels = pkg.read_rels_for("word/document.xml").unwrap();
    assert!(
        rels.items
            .iter()
            .any(|r| r.rel_type.ends_with("/header") && r.target.contains("header1.xml")),
        "header rel present"
    );
}

/// Adopting doc B's header must not OVERWRITE doc A's unrelated parts on a
/// name collision (PR #75 review): A carries its own `word/header1.xml`
/// (no refs — orphan) and its own `word/media/image1.png`; B's header of the
/// same name references a same-named image with different bytes. Both of
/// A's parts must survive byte-identical, and B's content must arrive under
/// fresh names, reachable through the new relationships.
#[test]
fn w6b_adopted_header_never_clobbers_existing_parts() {
    let base = std::fs::read("tests/fixtures/redline/original.docx").unwrap();

    // doc A: plain body, NO header refs — but orphan parts with colliding names
    let mut pa = PartFs::open(&base).unwrap();
    pa.set_part(
        "word/document.xml",
        format!(
            "<w:document xmlns:w=\"{w}\"><w:body><w:p><w:r><w:t>shared body text</w:t></w:r></w:p></w:body></w:document>",
            w = W::URI
        )
        .into_bytes(),
    );
    pa.set_part(
        "word/header1.xml",
        format!(
            "<w:hdr xmlns:w=\"{w}\"><w:p><w:r><w:t>DOC A ORPHAN HEADER</w:t></w:r></w:p></w:hdr>",
            w = W::URI
        )
        .into_bytes(),
    );
    pa.set_part("word/media/image1.png", b"AAAA-doc-a-image".to_vec());
    // pre-seed the first rename candidate too, so the uniqueness loop has to
    // bump past n == 0 (PR #75 review: the n > 0 branch was uncovered)
    pa.set_part(
        "word/media/redlineB_image1.png",
        b"CCCC-unrelated-preseed".to_vec(),
    );
    let a = pa.to_zip().unwrap();

    // doc B: same body + header1.xml (different content) referencing
    // media/image1.png (different bytes)
    let mut pb = PartFs::open(&base).unwrap();
    pb.set_part(
        "word/header1.xml",
        format!(
            "<w:hdr xmlns:w=\"{w}\" xmlns:r=\"{r}\"><w:p><w:r><w:t>REVISED HEADER TEXT</w:t></w:r></w:p></w:hdr>",
            w = W::URI,
            r = "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
        )
        .into_bytes(),
    );
    pb.set_part("word/media/image1.png", b"BBBB-doc-b-image".to_vec());
    pb.set_part(
        "word/_rels/header1.xml.rels",
        b"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
          <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
          <Relationship Id=\"rIdImg\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/image\" Target=\"media/image1.png\"/>\
          </Relationships>"
            .to_vec(),
    );
    pb.add_content_type_override(
        "/word/header1.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
    );
    let rid = pb.add_document_relationship(
        "word/document.xml",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
        "header1.xml",
    );
    pb.set_part(
        "word/document.xml",
        format!(
            "<w:document xmlns:w=\"{w}\" xmlns:r=\"{r}\"><w:body>\
             <w:p><w:r><w:t>shared body text</w:t></w:r></w:p>\
             <w:sectPr><w:headerReference w:type=\"default\" r:id=\"{rid}\"/>\
             <w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr>\
             </w:body></w:document>",
            w = W::URI,
            r = "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
        )
        .into_bytes(),
    );
    let b = pb.to_zip().unwrap();

    let out = compare_documents_with_settings(&a, &b, &word_mode()).unwrap();
    let pkg = PartFs::open(&out).unwrap();

    // doc A's colliding parts survive UNTOUCHED
    assert_eq!(
        pkg.part_bytes("word/media/image1.png").map(<[u8]>::to_vec),
        Some(b"AAAA-doc-a-image".to_vec()),
        "doc A's image must not be clobbered by B's same-named header image"
    );
    let a_hdr = pkg
        .part_string("word/header1.xml")
        .expect("A's orphan part");
    assert!(
        a_hdr.contains("DOC A ORPHAN HEADER"),
        "doc A's orphan header must not be clobbered: {a_hdr}"
    );

    // B's header arrived under a fresh name, wired through the new rel
    let rels = pkg.read_rels_for("word/document.xml").unwrap();
    let hdr_rel = rels
        .items
        .iter()
        .find(|r| r.rel_type.ends_with("/header"))
        .expect("adopted header rel");
    let hdr_part = pkg.resolve_rel_target("word/document.xml", &hdr_rel.target);
    let hx = pkg.part_string(&hdr_part).expect("adopted header part");
    assert!(
        hx.contains("REVISED HEADER TEXT"),
        "rel points at B's adopted header content: {hx}"
    );
    // …and B's image is reachable from the adopted header's rels
    let hrels = pkg.read_rels_for(&hdr_part).expect("adopted header rels");
    let img_rel = hrels
        .items
        .iter()
        .find(|r| r.rel_type.ends_with("/image"))
        .expect("image rel");
    let img_part = pkg.resolve_rel_target(&hdr_part, &img_rel.target);
    assert_eq!(
        pkg.part_bytes(&img_part).map(<[u8]>::to_vec),
        Some(b"BBBB-doc-b-image".to_vec()),
        "B's header image reachable under its fresh name ({img_part})"
    );
    // both of A's pre-seeded candidates survived, so the loop really bumped
    assert_eq!(
        pkg.part_bytes("word/media/redlineB_image1.png")
            .map(<[u8]>::to_vec),
        Some(b"CCCC-unrelated-preseed".to_vec()),
        "pre-seeded first candidate untouched"
    );
}

/// Word repairs a dangling `w:numId` reference by synthesizing a default
/// decimal multilevel numbering part (evidence: nested-table-rowspan_
/// numbered-list — the revised fixture references numId=2 with NO
/// numbering.xml; Word's redline carries a synthesized part and the list
/// renders numbered, ours carried the dangling ref and the list rendered as
/// plain paragraphs). Word-mode gated.
#[test]
fn w10_dangling_numid_gets_synthesized_numbering_part() {
    let base = std::fs::read("tests/fixtures/redline/original.docx").unwrap();

    // doc A: plain body, NO numbering part
    let mut pa = PartFs::open(&base).unwrap();
    pa.remove_part("word/numbering.xml");
    pa.set_part(
        "word/document.xml",
        format!(
            "<w:document xmlns:w=\"{w}\"><w:body><w:p><w:r><w:t>intro text</w:t></w:r></w:p></w:body></w:document>",
            w = W::URI
        )
        .into_bytes(),
    );
    let a = pa.to_zip().unwrap();

    // doc B: adds a numbered paragraph referencing numId=2 — also NO numbering part
    let mut pb = PartFs::open(&base).unwrap();
    pb.remove_part("word/numbering.xml");
    pb.set_part(
        "word/document.xml",
        format!(
            "<w:document xmlns:w=\"{w}\"><w:body>\
             <w:p><w:r><w:t>intro text</w:t></w:r></w:p>\
             <w:p><w:pPr><w:numPr><w:ilvl w:val=\"0\"/><w:numId w:val=\"2\"/></w:numPr></w:pPr>\
             <w:r><w:t>first item</w:t></w:r></w:p>\
             </w:body></w:document>",
            w = W::URI
        )
        .into_bytes(),
    );
    let b = pb.to_zip().unwrap();

    let out = compare_documents_with_settings(&a, &b, &word_mode()).unwrap();
    let pkg = PartFs::open(&out).unwrap();

    // output still references numId=2 …
    let dx = pkg.part_string("word/document.xml").unwrap();
    assert!(dx.contains("w:numId"), "numPr survives the diff: {dx}");

    // … and a numbering part was synthesized to satisfy it
    let nx = pkg
        .part_string("word/numbering.xml")
        .expect("numbering part synthesized for the dangling numId");
    assert!(
        nx.contains("w:numId=\"2\"") || nx.contains("w:num w:numId=\"2\""),
        "w:num for the dangling id present: {nx}"
    );
    assert!(
        nx.contains("abstractNum"),
        "abstractNum definition present: {nx}"
    );
    assert!(nx.contains("decimal"), "decimal multilevel default: {nx}");

    // package wiring: content type + document rel
    assert_eq!(
        pkg.content_type_for("word/numbering.xml").as_deref(),
        Some("application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml"),
        "content-type override present"
    );
    let rels = pkg.read_rels_for("word/document.xml").unwrap();
    assert!(
        rels.items
            .iter()
            .any(|r| r.rel_type.ends_with("/numbering") && r.target.contains("numbering.xml")),
        "numbering rel present"
    );
}

/// Documents with NO numbering references must not trigger synthesis (the
/// resolvable-reference case is covered by the pipeline carrying the part).
#[test]
fn w10b_no_numbering_references_triggers_no_synthesis() {
    let base = std::fs::read("tests/fixtures/redline/original.docx").unwrap();
    let mut pa = PartFs::open(&base).unwrap();
    pa.remove_part("word/numbering.xml");
    pa.set_part(
        "word/document.xml",
        format!(
            "<w:document xmlns:w=\"{w}\"><w:body><w:p><w:r><w:t>plain only</w:t></w:r></w:p></w:body></w:document>",
            w = W::URI
        )
        .into_bytes(),
    );
    let a = pa.to_zip().unwrap();
    let mut pb = PartFs::open(&base).unwrap();
    pb.remove_part("word/numbering.xml");
    pb.set_part(
        "word/document.xml",
        format!(
            "<w:document xmlns:w=\"{w}\"><w:body><w:p><w:r><w:t>plain only edited</w:t></w:r></w:p></w:body></w:document>",
            w = W::URI
        )
        .into_bytes(),
    );
    let b = pb.to_zip().unwrap();

    let out = compare_documents_with_settings(&a, &b, &word_mode()).unwrap();
    let pkg = PartFs::open(&out).unwrap();
    assert!(
        pkg.part_string("word/numbering.xml").is_none(),
        "no numbering references → no synthesized part"
    );
}

/// Notes parts at NONSTANDARD names (OPC: the rels are authoritative) must
/// still be diffed — hardcoded `word/footnotes.xml` silently skipped them
/// (PR #51 review).
#[test]
fn w19_nonstandard_notes_part_names_are_diffed() {
    let base = std::fs::read("tests/fixtures/redline/original.docx").unwrap();
    let w = jubarte::namespaces::W::URI;
    let mk = |note_text: &str| -> Vec<u8> {
        let mut p = PartFs::open(&base).unwrap();
        p.set_part(
            "word/notes1.xml",
            format!(
                "<w:footnotes xmlns:w=\"{w}\">\
                 <w:footnote w:type=\"separator\" w:id=\"-1\"><w:p><w:r><w:separator/></w:r></w:p></w:footnote>\
                 <w:footnote w:type=\"continuationSeparator\" w:id=\"0\"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:footnote>\
                 <w:footnote w:id=\"2\"><w:p><w:r><w:t>{note_text}</w:t></w:r></w:p></w:footnote>\
                 </w:footnotes>"
            )
            .into_bytes(),
        );
        p.add_content_type_override(
            "/word/notes1.xml",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml",
        );
        p.add_document_relationship(
            "word/document.xml",
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes",
            "notes1.xml",
        );
        p.set_part(
            "word/document.xml",
            format!(
                "<w:document xmlns:w=\"{w}\"><w:body>\
                 <w:p><w:r><w:t>body text</w:t></w:r>\
                 <w:r><w:footnoteReference w:id=\"2\"/></w:r></w:p>\
                 </w:body></w:document>"
            )
            .into_bytes(),
        );
        p.to_zip().unwrap()
    };
    let a = mk("original note wording");
    let b = mk("edited note wording");

    let out = compare_documents_with_settings(&a, &b, &word_mode()).unwrap();
    let pkg = PartFs::open(&out).unwrap();
    let nx = pkg
        .part_string("word/notes1.xml")
        .expect("nonstandard notes part present in output");
    assert!(
        nx.contains("edited") && nx.contains("wording"),
        "the note DIFF reached the nonstandard part (revised text present): {nx}"
    );
}

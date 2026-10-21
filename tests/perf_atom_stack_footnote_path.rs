// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! ATOM-STACK-01 regression: footnote/endnote must stay on the ancestor path.
//!
//! Pre-ATOM-STACK `ancestor_chain` stopped at the footnotes/endnotes *part*
//! (`w:footnotes` / `w:endnotes`), body, hdr, ftr — but NOT at individual
//! `w:footnote` / `w:endnote` definitions. ProcessFootnoteEndnote atomizes
//! those definitions as content roots and produce rebuilds the note wrapper
//! from `ancestor_elements[0]`. Dropping the note from the path made
//! `produce_note_redline` return None → panic on deleted notes
//! (parity ladder CRASH on footnotes_sample×gdocs_comments_export).

use jubarte::comparer::WmlComparerSettings;
use jubarte::comparer::atomize::create_comparison_unit_atom_list;
use jubarte::document_comparer::compare_documents;
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

fn note_root_atoms(
    dom: &mut Dom,
    note_local: &str,
    inner: &str,
) -> Vec<jubarte::comparer::atoms::ComparisonUnitAtom> {
    let xml = format!(
        r#"<?xml version="1.0"?>
        <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
          <w:body>
            <w:{note_local} w:id="1">{inner}</w:{note_local}>
          </w:body>
        </w:document>"#
    );
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).expect("root");
    let body = dom.element(root, &W::body()).expect("body");
    let note = dom
        .elements(body, Some(&W::name(note_local)))
        .into_iter()
        .next()
        .expect("note");
    // ProcessFootnoteEndnote atomizes the definition node, not the body.
    create_comparison_unit_atom_list(dom, note, &WmlComparerSettings::default())
}

fn path_locals(dom: &Dom, atoms: &[jubarte::comparer::atoms::ComparisonUnitAtom]) -> Vec<String> {
    let text = atoms
        .iter()
        .find(|a| dom.name(a.content_element) == Some(W::t()))
        .expect("text atom");
    text.ancestor_elements
        .iter()
        .map(|&n| {
            dom.name(n)
                .map(|nm| nm.local_name().to_string())
                .unwrap_or_else(|| "?".into())
        })
        .collect()
}

#[test]
fn atom_stack_footnote_definition_path_includes_footnote() {
    let mut dom = Dom::new();
    let atoms = note_root_atoms(
        &mut dom,
        "footnote",
        r#"<w:p><w:r><w:t>Ouch.</w:t></w:r></w:p>"#,
    );
    let anc = path_locals(&dom, &atoms);
    assert_eq!(
        anc.first().map(String::as_str),
        Some("footnote"),
        "old ancestor_chain included w:footnote; got {anc:?}"
    );
    assert!(
        anc.ends_with(&["p".into(), "r".into(), "t".into()])
            || anc
                .windows(3)
                .any(|w| w == ["p".to_string(), "r".to_string(), "t".to_string()]),
        "expected …p/r/t leaf path, got {anc:?}"
    );
}

#[test]
fn atom_stack_endnote_definition_path_includes_endnote() {
    let mut dom = Dom::new();
    let atoms = note_root_atoms(
        &mut dom,
        "endnote",
        r#"<w:p><w:r><w:t>Note</w:t></w:r></w:p>"#,
    );
    let anc = path_locals(&dom, &atoms);
    assert_eq!(
        anc.first().map(String::as_str),
        Some("endnote"),
        "old ancestor_chain included w:endnote; got {anc:?}"
    );
}

#[test]
fn atom_stack_body_path_still_excludes_body() {
    let mut dom = Dom::new();
    let xml = r#"<?xml version="1.0"?>
        <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
          <w:body><w:p><w:r><w:t>Hi</w:t></w:r></w:p></w:body>
        </w:document>"#;
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    let atoms = create_comparison_unit_atom_list(&mut dom, body, &WmlComparerSettings::default());
    let anc = path_locals(&dom, &atoms);
    assert_eq!(anc.first().map(String::as_str), Some("p"));
    assert!(!anc.iter().any(|s| s == "body"));
}

/// Real shipped path: deleted footnote content must not panic produce_note_redline.
#[test]
fn deleted_footnote_compare_does_not_panic() {
    // Minimal packages: A has a footnote body ref + definition; B has neither.
    // Exercising ProcessFootnoteEndnote Deleted branch end-to-end.
    fn pkg(document_xml: &str, footnotes_xml: Option<&str>) -> Vec<u8> {
        use std::io::Write;
        let mut buf = Vec::new();
        {
            let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
            let opt = zip::write::FileOptions::<()>::default()
                .compression_method(zip::CompressionMethod::Stored);
            zip.start_file("[Content_Types].xml", opt).unwrap();
            let mut ctypes = String::from(
                r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>"#,
            );
            if footnotes_xml.is_some() {
                ctypes.push_str(
                    r#"
  <Override PartName="/word/footnotes.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml"/>"#,
                );
            }
            ctypes.push_str("\n</Types>");
            zip.write_all(ctypes.as_bytes()).unwrap();

            zip.start_file("_rels/.rels", opt).unwrap();
            zip.write_all(
                br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#,
            )
            .unwrap();

            zip.start_file("word/_rels/document.xml.rels", opt).unwrap();
            let mut rels = String::from(
                r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
            );
            if footnotes_xml.is_some() {
                rels.push_str(
                    r#"
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes" Target="footnotes.xml"/>"#,
                );
            }
            rels.push_str("\n</Relationships>");
            zip.write_all(rels.as_bytes()).unwrap();

            zip.start_file("word/document.xml", opt).unwrap();
            zip.write_all(document_xml.as_bytes()).unwrap();

            if let Some(fnx) = footnotes_xml {
                zip.start_file("word/footnotes.xml", opt).unwrap();
                zip.write_all(fnx.as_bytes()).unwrap();
            }
            zip.finish().unwrap();
        }
        buf
    }

    let a_doc = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p>
      <w:r><w:t>Body</w:t></w:r>
      <w:r><w:footnoteReference w:id="1"/></w:r>
    </w:p>
  </w:body>
</w:document>"#;
    let a_fn = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:footnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:footnote w:type="separator" w:id="-1"><w:p><w:r><w:separator/></w:r></w:p></w:footnote>
  <w:footnote w:type="continuationSeparator" w:id="0"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:footnote>
  <w:footnote w:id="1">
    <w:p><w:r><w:rPr><w:rStyle w:val="FootnoteReference"/></w:rPr><w:footnoteRef/></w:r>
      <w:r><w:t>Ouch.</w:t></w:r></w:p>
  </w:footnote>
</w:footnotes>"#;
    let b_doc = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p><w:r><w:t>Body</w:t></w:r></w:p>
  </w:body>
</w:document>"#;

    let a = pkg(a_doc, Some(a_fn));
    let b = pkg(b_doc, None);
    let out = compare_documents(&a, &b, "Test").expect("deleted footnote must not panic");
    assert!(
        out.len() > 100,
        "expected a real package, got {} bytes",
        out.len()
    );
    // Package must contain delText stream of the deleted note body.
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(&out[..])).unwrap();
    let mut found_note_text = false;
    for i in 0..zip.len() {
        let mut f = zip.by_index(i).unwrap();
        let name = f.name().to_string();
        if !name.ends_with(".xml") {
            continue;
        }
        let mut s = String::new();
        use std::io::Read;
        f.read_to_string(&mut s).unwrap();
        if s.contains("Ouch") {
            found_note_text = true;
            break;
        }
    }
    assert!(
        found_note_text,
        "redline should still carry deleted footnote text 'Ouch'"
    );
}

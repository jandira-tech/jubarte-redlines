// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M22 — feature-gating `mc:AlternateContent` must be RESOLVED (replaced by its
//! `mc:Choice` content), like Word does, while drawing/VML fallbacks are KEPT.
//!
//! We treated every `mc:AlternateContent` as an opaque atom kept verbatim
//! (atomize.rs:256, produce.rs:557). Run-level feature-gating AltContent then got
//! hoisted to invalid block positions → Word "unreadable content" repair
//! (e.g. alternate-content_bookmark_use_cases), and `ooxmlsdk` rejects it with
//! `UnexpectedTag { ty: TableCell, found: AlternateContent }`.
//!
//! Validated against the 100-pair corpus: Word's outputs retain 7 drawing/VML
//! AltContent and 0 text-only ones — i.e. Word resolves text-only, keeps drawing.

use std::io::{Cursor, Read, Write};

use jubarte::document_comparer::compare_documents;

const W_NS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
const MC_NS: &str = "http://schemas.openxmlformats.org/markup-compatibility/2006";
const W14_NS: &str = "http://schemas.microsoft.com/office/word/2010/wordml";

fn build_docx(doc_xml: &str) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
        let opt = zip::write::SimpleFileOptions::default();
        z.start_file("[Content_Types].xml", opt).unwrap();
        z.write_all(br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#).unwrap();
        z.start_file("_rels/.rels", opt).unwrap();
        z.write_all(br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rIdM" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#).unwrap();
        z.start_file("word/document.xml", opt).unwrap();
        z.write_all(doc_xml.as_bytes()).unwrap();
        z.start_file("word/_rels/document.xml.rels", opt).unwrap();
        z.write_all(br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"/>"#).unwrap();
        z.finish().unwrap();
    }
    buf
}

/// Body with a text-only feature-gating AltContent (must resolve) AND a drawing
/// AltContent (must be kept). `marker` distinguishes A from B so there is a diff.
fn doc(marker: &str) -> Vec<u8> {
    let body = format!(
        "<w:document xmlns:w=\"{W_NS}\" xmlns:mc=\"{MC_NS}\" xmlns:w14=\"{W14_NS}\"><w:body>\
           <w:p><w:r><w:t>DIFFWORD_{marker}</w:t></w:r></w:p>\
           <w:p><w:r><w:t xml:space=\"preserve\">before </w:t></w:r>\
             <mc:AlternateContent>\
               <mc:Choice Requires=\"w14\"><w:r><w:t>FEATUREWORD</w:t></w:r></mc:Choice>\
               <mc:Fallback><w:r><w:t>fallbackword</w:t></w:r></mc:Fallback>\
             </mc:AlternateContent>\
             <w:r><w:t xml:space=\"preserve\"> after</w:t></w:r></w:p>\
           <w:p><w:r>\
             <mc:AlternateContent>\
               <mc:Choice Requires=\"wps\"><w:drawing/></mc:Choice>\
               <mc:Fallback><w:pict/></mc:Fallback>\
             </mc:AlternateContent></w:r></w:p>\
           <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr>\
         </w:body></w:document>"
    );
    build_docx(&body)
}

fn read_part(docx: &[u8], name: &str) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name(name).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

#[test]
fn text_only_alternate_content_is_resolved_drawing_is_kept() {
    let out = compare_documents(&doc("A"), &doc("B"), "Test").expect("compare ok");
    let x = read_part(&out, "word/document.xml");

    let n_altcontent = x.matches("<mc:AlternateContent").count();
    assert_eq!(
        n_altcontent, 1,
        "exactly ONE AltContent should survive (the drawing one); the text-only one must resolve. got {n_altcontent}:\n{x}"
    );
    assert!(
        x.contains("FEATUREWORD"),
        "resolved Choice content must remain inline: {x}"
    );
    assert!(
        !x.contains("fallbackword"),
        "the discarded Fallback content must be gone: {x}"
    );
    assert!(
        x.contains("<w:drawing"),
        "the drawing AltContent (Choice) must be kept: {x}"
    );

    // And the output must load via the strict typed SDK (the validity oracle).
    ooxmlsdk::parts::wordprocessing_document::WordprocessingDocument::new(Cursor::new(out))
        .expect("output must be ooxmlsdk-loadable");
}

/// MCE branch selection must honor `@Requires`: the first `mc:Choice` is NOT taken
/// unconditionally. When the first Choice gates on a namespace the document never
/// binds, resolution falls through to a later (understood) Choice or the Fallback —
/// the content Word actually keeps. Only `w14` is declared below; `w99` is not.
fn doc_multichoice(marker: &str) -> Vec<u8> {
    let body = format!(
        "<w:document xmlns:w=\"{W_NS}\" xmlns:mc=\"{MC_NS}\" xmlns:w14=\"{W14_NS}\"><w:body>\
           <w:p><w:r><w:t>DIFFWORD_{marker}</w:t></w:r></w:p>\
           <w:p><w:r>\
             <mc:AlternateContent>\
               <mc:Choice Requires=\"w99\"><w:r><w:t>UNDECLAREDCHOICE</w:t></w:r></mc:Choice>\
               <mc:Choice Requires=\"w14\"><w:r><w:t>SECONDCHOICE</w:t></w:r></mc:Choice>\
               <mc:Fallback><w:r><w:t>fallbackword</w:t></w:r></mc:Fallback>\
             </mc:AlternateContent></w:r></w:p>\
           <w:p><w:r>\
             <mc:AlternateContent>\
               <mc:Choice Requires=\"w99\"><w:r><w:t>UNDECLAREDONLY</w:t></w:r></mc:Choice>\
               <mc:Fallback><w:r><w:t>FALLBACKONLY</w:t></w:r></mc:Fallback>\
             </mc:AlternateContent></w:r></w:p>\
           <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr>\
         </w:body></w:document>"
    );
    build_docx(&body)
}

#[test]
fn alternate_content_respects_requires_branch_selection() {
    let out = compare_documents(&doc_multichoice("A"), &doc_multichoice("B"), "Test")
        .expect("compare ok");
    let x = read_part(&out, "word/document.xml");

    // No AltContent should survive (both are text-only and must resolve).
    assert_eq!(
        x.matches("<mc:AlternateContent").count(),
        0,
        "text-only AltContent must resolve: {x}"
    );
    // First AltContent: undeclared `w99` Choice is skipped, the understood `w14`
    // Choice wins — NOT the first Choice, and NOT the Fallback.
    assert!(
        x.contains("SECONDCHOICE"),
        "understood (w14) Choice must win: {x}"
    );
    assert!(
        !x.contains("UNDECLAREDCHOICE"),
        "first (w99) Choice must be skipped: {x}"
    );
    // Second AltContent: the only Choice gates on undeclared `w99`, so the Fallback
    // is selected.
    assert!(
        x.contains("FALLBACKONLY"),
        "fallback must be selected when no Choice is understood: {x}"
    );
    assert!(
        !x.contains("UNDECLAREDONLY"),
        "undeclared-only Choice must not be selected: {x}"
    );

    ooxmlsdk::parts::wordprocessing_document::WordprocessingDocument::new(Cursor::new(out))
        .expect("output must be ooxmlsdk-loadable");
}

/// DOM-level edge cases for the `@Requires` branch-selection heuristic in
/// `resolve_alternate_content` (the docx tests above only cover the happy path).
/// These call the pass directly so each branch of `choice_requires_understood` /
/// `prefix_in_scope` is pinned.
mod requires_selection {
    use jubarte::comparer::finalize::resolve_alternate_content;
    use jubarte::namespaces::{MC, W};
    use jubarte::xmllinq::{Dom, NodeId, XNamespace};

    const W_NS: &str = super::W_NS;
    const MC_NS: &str = super::MC_NS;
    const W14_NS: &str = super::W14_NS;

    /// `<w:document xmlns:w xmlns:mc xmlns:w14><w:body>…</w:body></w:document>`,
    /// run the build closure on the body, resolve, return the serialized document.
    fn resolved(build: impl FnOnce(&mut Dom, NodeId)) -> String {
        let mut d = Dom::new();
        let root = d.new_element(W::document());
        d.set_attribute_value(root, &XNamespace::xmlns().name("w"), Some(W_NS));
        d.set_attribute_value(root, &XNamespace::xmlns().name("mc"), Some(MC_NS));
        d.set_attribute_value(root, &XNamespace::xmlns().name("w14"), Some(W14_NS));
        let body = d.new_element(W::body());
        d.add(root, body);
        build(&mut d, body);
        resolve_alternate_content(&mut d, root);
        d.serialize_element(root)
    }

    fn add_choice(d: &mut Dom, ac: NodeId, requires: Option<&str>, marker: &str) {
        let ch = d.new_element(MC::name("Choice"));
        if let Some(r) = requires {
            d.set_attribute_value(ch, &XNamespace::none().name("Requires"), Some(r));
        }
        let r = d.new_element(W::r());
        let t = d.new_element(W::t());
        d.add_text(t, marker);
        d.add(r, t);
        d.add(ch, r);
        d.add(ac, ch);
    }

    fn add_fallback(d: &mut Dom, ac: NodeId, marker: &str) {
        let fb = d.new_element(MC::name("Fallback"));
        let r = d.new_element(W::r());
        let t = d.new_element(W::t());
        d.add_text(t, marker);
        d.add(r, t);
        d.add(fb, r);
        d.add(ac, fb);
    }

    /// Every prefix in `@Requires` must be in scope: `"w14 w99"` (w99 undeclared)
    /// is skipped, the later single-`w14` Choice wins — not the first, not fallback.
    #[test]
    fn all_requires_prefixes_must_be_in_scope() {
        let s = resolved(|d, body| {
            let ac = d.new_element(MC::name("AlternateContent"));
            d.add(body, ac);
            add_choice(d, ac, Some("w14 w99"), "MULTIUNDECLARED");
            add_choice(d, ac, Some("w14"), "SINGLEOK");
            add_fallback(d, ac, "FB");
        });
        assert!(
            s.contains("SINGLEOK"),
            "understood single-prefix Choice wins: {s}"
        );
        assert!(
            !s.contains("MULTIUNDECLARED"),
            "Choice with any undeclared prefix is skipped: {s}"
        );
        assert!(
            !s.contains("FB"),
            "fallback not used when a Choice qualifies: {s}"
        );
        assert!(
            !s.contains("AlternateContent"),
            "AltContent resolved away: {s}"
        );
    }

    /// A Choice with no `@Requires` is vacuously understood and selected.
    #[test]
    fn choice_without_requires_is_selected() {
        let s = resolved(|d, body| {
            let ac = d.new_element(MC::name("AlternateContent"));
            d.add(body, ac);
            add_choice(d, ac, None, "NOREQUIRES");
            add_fallback(d, ac, "FB");
        });
        assert!(
            s.contains("NOREQUIRES"),
            "no-@Requires Choice is selected: {s}"
        );
        assert!(!s.contains("FB"), "{s}");
    }

    /// No Choice understood AND no Fallback → the AltContent is left intact rather
    /// than fabricating content (preserve the input; this shape is malformed/rare).
    #[test]
    fn no_understood_choice_and_no_fallback_leaves_altcontent() {
        let s = resolved(|d, body| {
            let ac = d.new_element(MC::name("AlternateContent"));
            d.add(body, ac);
            add_choice(d, ac, Some("w99"), "UNDECLARED");
        });
        assert!(
            s.contains("AlternateContent"),
            "unresolved AltContent is preserved, not dropped: {s}"
        );
        assert!(s.contains("UNDECLARED"), "{s}");
    }

    /// A prefix declared by an `xmlns:` on the Choice itself counts as in scope
    /// (XML namespace scoping — `ancestors_and_self`).
    #[test]
    fn requires_prefix_declared_on_choice_is_in_scope() {
        let mut d = Dom::new();
        let root = d.new_element(W::document());
        d.set_attribute_value(root, &XNamespace::xmlns().name("w"), Some(W_NS));
        d.set_attribute_value(root, &XNamespace::xmlns().name("mc"), Some(MC_NS));
        let body = d.new_element(W::body());
        d.add(root, body);
        let ac = d.new_element(MC::name("AlternateContent"));
        d.add(body, ac);
        let ch = d.new_element(MC::name("Choice"));
        // declare the required prefix locally on the Choice (not on the root)
        d.set_attribute_value(ch, &XNamespace::xmlns().name("wlocal"), Some("urn:x-local"));
        d.set_attribute_value(ch, &XNamespace::none().name("Requires"), Some("wlocal"));
        let r = d.new_element(W::r());
        let t = d.new_element(W::t());
        d.add_text(t, "LOCALOK");
        d.add(r, t);
        d.add(ch, r);
        d.add(ac, ch);
        resolve_alternate_content(&mut d, root);
        let s = d.serialize_element(root);
        assert!(
            s.contains("LOCALOK"),
            "Choice-local xmlns satisfies its own @Requires: {s}"
        );
        assert!(!s.contains("AlternateContent"), "{s}");
    }
}

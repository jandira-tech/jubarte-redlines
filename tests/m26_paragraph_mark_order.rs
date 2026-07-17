// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M26 — paragraph-mark revision markers must obey OOXML child order.
//!
//! When a paragraph mark is inserted/deleted, the marker goes in the paragraph's
//! paraRPr: `<w:pPr>…<w:rPr><w:ins/>…</w:rPr></w:pPr>`. OOXML requires
//! (a) `ins`/`del`/`moveFrom`/`moveTo` to be the FIRST children of the paraRPr
//! (CT_ParaRPr), and (b) the paraRPr to come AFTER the paragraph's content
//! properties (pStyle, …) in the pPr. We appended the marker after existing rPr
//! props (`<w:rPr><w:lang/><w:ins/>`) and/or placed the paraRPr before pStyle —
//! ooxmlsdk tolerates it, but real Word reports "unreadable content" (the cause of
//! the contract-acc / hyperlink / NumberingImplicitNumId / nda corruptions).

use std::io::{Cursor, Read, Write};

use jubarte::document_comparer::compare_documents;
use quick_xml::NsReader;
use quick_xml::events::Event;
use quick_xml::name::{Namespace, ResolveResult};

const W_NS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
const W_NS_BYTES: &[u8] = W_NS.as_bytes();

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

fn read_part(docx: &[u8], name: &str) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name(name).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

/// Direct child element local-names of every `<w:rPr>` / `<w:pPr>` block — used to
/// assert ordering without a full parser.
/// For every `<w:{parent}>` element in `xml`, the ordered local-names of its
/// **direct** child elements — namespace-resolved (matched on the WordprocessingML
/// URI, not the `w:` prefix) and depth-correct (a proper element stack, so a
/// non-self-closing child no longer hides the siblings that follow it). Empty
/// elements (`<w:pStyle .../>`) count as children.
fn direct_children_of(xml: &str, parent: &str) -> Vec<Vec<String>> {
    let mut reader = NsReader::from_str(xml);
    reader.config_mut().trim_text(false);
    let is_w =
        |ns: &ResolveResult| matches!(ns, ResolveResult::Bound(Namespace(n)) if *n == W_NS_BYTES);
    let mut results: Vec<Vec<String>> = Vec::new();
    // One frame per open element: `Some(bucket)` if that element is a target
    // `parent`, else `None`. The frame on top of the stack is the current element.
    let mut stack: Vec<Option<usize>> = Vec::new();
    loop {
        match reader.read_resolved_event() {
            Ok((ns, Event::Start(e))) => {
                let local = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();
                if let Some(Some(bi)) = stack.last() {
                    results[*bi].push(local.clone());
                }
                if is_w(&ns) && local == parent {
                    results.push(Vec::new());
                    let bi = results.len() - 1;
                    stack.push(Some(bi));
                } else {
                    stack.push(None);
                }
            }
            Ok((ns, Event::Empty(e))) => {
                let local = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();
                if let Some(Some(bi)) = stack.last() {
                    results[*bi].push(local.clone());
                }
                if is_w(&ns) && local == parent {
                    results.push(Vec::new()); // an empty `parent` has no children
                }
            }
            Ok((_, Event::End(_))) => {
                stack.pop();
            }
            Ok((_, Event::Eof)) => break,
            Err(e) => panic!("XML parse error: {e}"),
            _ => {}
        }
    }
    results
}

#[test]
fn paragraph_mark_revision_markers_are_ordered() {
    // A: one paragraph. B: that paragraph + an INSERTED paragraph carrying a pStyle
    // and a paraRPr with a w:lang (so the inserted para-mark must slot ins FIRST in
    // the paraRPr, and the paraRPr must follow pStyle).
    let a = build_docx(&format!(
        "<w:document xmlns:w=\"{W_NS}\"><w:body><w:p><w:r><w:t>shared</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr></w:body></w:document>"
    ));
    let b = build_docx(&format!(
        "<w:document xmlns:w=\"{W_NS}\"><w:body><w:p><w:r><w:t>shared</w:t></w:r></w:p>\
         <w:p><w:pPr><w:pStyle w:val=\"Heading1\"/><w:rPr><w:lang w:val=\"en-US\"/></w:rPr></w:pPr><w:r><w:t>NEWPARA</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr></w:body></w:document>"
    ));
    let out = compare_documents(&a, &b, "Test").expect("compare ok");
    let x = read_part(&out, "word/document.xml");

    // Every rPr that carries a revision marker must list it FIRST.
    for dc in direct_children_of(&x, "rPr") {
        if dc
            .iter()
            .any(|c| matches!(c.as_str(), "ins" | "del" | "moveFrom" | "moveTo"))
        {
            assert!(
                matches!(
                    dc.first().map(|s| s.as_str()),
                    Some("ins" | "del" | "moveFrom" | "moveTo")
                ),
                "revision marker must be first in rPr, got {dc:?}"
            );
        }
    }
    // Every pPr that has both pStyle and rPr must order pStyle before rPr.
    for dc in direct_children_of(&x, "pPr") {
        if let (Some(ps), Some(rp)) = (
            dc.iter().position(|c| c == "pStyle"),
            dc.iter().position(|c| c == "rPr"),
        ) {
            assert!(ps < rp, "pStyle must precede rPr in pPr, got {dc:?}");
        }
    }
}

/// Anchored-insertion path: when the paragraph carries a `sectPr` (or `pPrChange`),
/// the revision-bearing paraRPr must be repositioned to sit *before* that anchor
/// (CT_PPr order: content props … rPr, then sectPr, then pPrChange), and the marker
/// must still be hoisted to the front of the rPr. Drives
/// `fix_paragraph_mark_revision_order` directly so the anchored branch is exercised
/// (the end-to-end fixture above only hits the unanchored `dom.add(ppr, rpr)` case).
#[test]
fn paragraph_mark_rpr_is_repositioned_before_sectpr_anchor() {
    use jubarte::comparer::finalize::fix_paragraph_mark_revision_order;
    use jubarte::namespaces::W;
    use jubarte::xmllinq::Dom;

    let mut d = Dom::new();
    let body = d.new_element(W::body());
    let p = d.new_element(W::p());
    let ppr = d.new_element(W::p_pr());

    // content prop first
    let pstyle = d.new_element(W::name("pStyle"));
    d.set_attribute_value(pstyle, &W::name("val"), Some("Heading1"));
    d.add(ppr, pstyle);

    // paraRPr with a non-marker prop placed BEFORE the marker (wrong order on
    // purpose), and the whole rPr placed BEFORE the sectPr below — but we then add
    // the sectPr so the rPr currently precedes it; the marker order is what (a) fixes
    // and the anchor is what (b) must respect.
    let rpr = d.new_element(W::name("rPr"));
    let lang = d.new_element(W::name("lang"));
    d.add(rpr, lang);
    let ins = d.new_element(W::ins());
    d.set_attribute_value(ins, &W::id(), Some("5"));
    d.add(rpr, ins);
    d.add(ppr, rpr);

    // paragraph-local section break — the reposition anchor.
    let sectpr = d.new_element(W::name("sectPr"));
    d.add(ppr, sectpr);

    d.add(p, ppr);
    d.add(body, p);

    fix_paragraph_mark_revision_order(&mut d, body);

    // (a) the ins marker is hoisted to the front of the paraRPr.
    let rpr2 = d.element(ppr, &W::name("rPr")).expect("rPr survives");
    let first = d
        .elements(rpr2, None)
        .first()
        .copied()
        .expect("rPr has children");
    assert_eq!(
        d.name(first).unwrap().local_name(),
        "ins",
        "ins must be first in paraRPr"
    );

    // (b) the paraRPr is repositioned to sit BEFORE the sectPr anchor.
    let kids: Vec<String> = d
        .elements(ppr, None)
        .iter()
        .map(|&c| d.name(c).unwrap().local_name().to_string())
        .collect();
    let rp = kids.iter().position(|s| s == "rPr").expect("rPr present");
    let sp = kids
        .iter()
        .position(|s| s == "sectPr")
        .expect("sectPr present");
    assert!(
        rp < sp,
        "paraRPr must precede the sectPr anchor, got {kids:?}"
    );
}

/// `pPrChange` is the other reposition anchor (same `.or_else` path as `sectPr`):
/// the revision paraRPr must land before it too.
#[test]
fn paragraph_mark_rpr_is_repositioned_before_pprchange_anchor() {
    use jubarte::comparer::finalize::fix_paragraph_mark_revision_order;
    use jubarte::namespaces::W;
    use jubarte::xmllinq::Dom;

    let mut d = Dom::new();
    let body = d.new_element(W::body());
    let p = d.new_element(W::p());
    let ppr = d.new_element(W::p_pr());

    let rpr = d.new_element(W::name("rPr"));
    let lang = d.new_element(W::name("lang"));
    d.add(rpr, lang);
    let ins = d.new_element(W::ins());
    d.set_attribute_value(ins, &W::id(), Some("7"));
    d.add(rpr, ins);
    d.add(ppr, rpr);

    let pprchange = d.new_element(W::name("pPrChange"));
    d.add(ppr, pprchange);

    d.add(p, ppr);
    d.add(body, p);

    fix_paragraph_mark_revision_order(&mut d, body);

    let rpr2 = d.element(ppr, &W::name("rPr")).expect("rPr survives");
    let first = d
        .elements(rpr2, None)
        .first()
        .copied()
        .expect("rPr has children");
    assert_eq!(
        d.name(first).unwrap().local_name(),
        "ins",
        "ins first in paraRPr"
    );

    let kids: Vec<String> = d
        .elements(ppr, None)
        .iter()
        .map(|&c| d.name(c).unwrap().local_name().to_string())
        .collect();
    let rp = kids.iter().position(|s| s == "rPr").expect("rPr present");
    let pc = kids
        .iter()
        .position(|s| s == "pPrChange")
        .expect("pPrChange present");
    assert!(
        rp < pc,
        "paraRPr must precede the pPrChange anchor, got {kids:?}"
    );
}

/// Regression guard for the `direct_children_of` helper itself: a non-self-closing
/// child (`rPr` with its own child) must NOT hide the sibling that follows it
/// (`sectPr`) — the exact bug the old depth-broken string scan had.
#[test]
fn direct_children_of_is_depth_correct() {
    let xml = format!(
        "<w:pPr xmlns:w=\"{W_NS}\"><w:pStyle w:val=\"H\"/><w:rPr><w:b/></w:rPr><w:sectPr/></w:pPr>"
    );
    assert_eq!(
        direct_children_of(&xml, "pPr"),
        vec![vec![
            "pStyle".to_string(),
            "rPr".to_string(),
            "sectPr".to_string()
        ]],
        "sibling after a nested child must still be seen"
    );
    // the nested rPr is reported separately with only its own direct child
    assert_eq!(direct_children_of(&xml, "rPr"), vec![vec!["b".to_string()]]);
}

/// The helper matches on the WordprocessingML namespace URI, not the literal `w:`
/// prefix — a different prefix bound to the same URI still resolves.
#[test]
fn direct_children_of_resolves_by_namespace_not_prefix() {
    let xml = format!("<x:pPr xmlns:x=\"{W_NS}\"><x:pStyle/><x:rPr/></x:pPr>");
    assert_eq!(
        direct_children_of(&xml, "pPr"),
        vec![vec!["pStyle".to_string(), "rPr".to_string()]],
        "non-`w` prefix bound to the W namespace must still match"
    );
}

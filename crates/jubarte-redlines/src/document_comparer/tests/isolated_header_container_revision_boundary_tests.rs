// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;
fn semantic(dom: &Dom, node: NodeId) -> String {
    if !dom.is_element(node) {
        return dom.text_value(node).unwrap_or_default().into();
    }
    let name = dom.name(node).unwrap();
    let mut attrs = dom
        .attributes_iter(node)
        .filter(|(n, _)| {
            n.namespace_name() != "http://www.w3.org/2000/xmlns/" && n.local_name() != "xmlns"
        })
        .collect::<Vec<_>>();
    if attrs.is_empty() && dom.nodes(node).is_empty() && (name == W::r_pr() || name == W::p_pr()) {
        return String::new();
    }
    attrs.sort_by_key(|(n, v)| (n.namespace_name().to_owned(), n.local_name().to_owned(), *v));
    format!(
        "{:?}{attrs:?}[{}]",
        name,
        dom.nodes(node)
            .into_iter()
            .map(|n| semantic(dom, n))
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("|")
    )
}
fn parsed(xml: &str) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).unwrap();
    (dom, root)
}
#[test]
fn pure_header_container_payloads_recover_every_authored_property_and_fallback() {
    let run = "<w:r><w:rPr><w:b/><w:color w:val='123456'/></w:rPr><w:t>owned header payload</w:t><w:tab/><w:br/></w:r>";
    let forms = [
        format!(
            "<w:hyperlink w:anchor='Clause'>{run}</w:hyperlink><w:bookmarkStart w:id='31' w:name='Clause'/><w:bookmarkEnd w:id='31'/>"
        ),
        format!(
            "<w:sdt><w:sdtPr><w:alias w:val='Clause'/><w:tag w:val='header-owner'/><w:id w:val='32'/><w:richText/></w:sdtPr><w:sdtContent>{run}</w:sdtContent></w:sdt>"
        ),
        format!(
            "<mc:AlternateContent><mc:Choice Requires='w14'>{run}</mc:Choice><mc:Fallback><w:r><w:rPr><w:i/></w:rPr><w:t>fallback payload</w:t></w:r></mc:Fallback></mc:AlternateContent>"
        ),
    ];
    for form in forms {
        for footer in [false, true] {
            for mark_properties in [false, true] {
                for deleting in [false, true] {
                    let kind = if footer { "ftr" } else { "hdr" };
                    let part = if footer {
                        "word/owned-footer.xml"
                    } else {
                        "word/owned-header.xml"
                    };
                    let mark = if mark_properties {
                        "<w:rPr><w:b/><w:color w:val='345678'/></w:rPr>"
                    } else {
                        ""
                    };
                    let xml = format!(
                        "<w:{kind} xmlns:w='{}' xmlns:mc='http://schemas.openxmlformats.org/markup-compatibility/2006' xmlns:w14='http://schemas.microsoft.com/office/word/2010/wordml'><w:p><w:pPr><w:spacing w:after='120'/><w:ind w:left='180'/>{mark}</w:pPr>{form}</w:p></w:{kind}>",
                        W::URI
                    );
                    let (source, root) = parsed(&xml);
                    let expected = semantic(&source, root);
                    let mut pkg = PartFs::open(include_bytes!(
                        "../../../../../tests/fixtures/word_probes/tokens/cell_a.docx"
                    ))
                    .unwrap();
                    pkg.set_part(part, xml.into_bytes());
                    pkg.add_content_type_override(&format!("/{part}"),if footer {"application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml"} else {"application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml"});
                    let relationship = pkg.add_document_relationship(
                            "word/document.xml",
                            if footer {
                                "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer"
                            } else {
                                "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header"
                            },
                            part.strip_prefix("word/").unwrap(),
                        );
                    let (mut main_dom, main_root) =
                        parsed(&pkg.part_string("word/document.xml").unwrap());
                    let body = main_dom.element(main_root, &W::body()).unwrap();
                    let section = main_dom.element(body, &W::sect_pr()).unwrap();
                    let reference = main_dom.new_element(W::name(if footer {
                        "footerReference"
                    } else {
                        "headerReference"
                    }));
                    main_dom.set_attribute_value(reference, &W::type_(), Some("default"));
                    main_dom.set_attribute_value(reference, &R::id(), Some(&relationship));
                    main_dom.add_first(section, reference);
                    pkg.set_part(
                        "word/document.xml",
                        main_dom.serialize_element(main_root).into_bytes(),
                    );
                    let settings = WmlComparerSettings {
                        author_for_revisions: "New owner".into(),
                        date_time_for_revisions: "2001-02-03T04:05:06Z".into(),
                        ..WmlComparerSettings::default()
                    };
                    let main = pkg.part_bytes("word/document.xml").unwrap().to_vec();
                    if deleting {
                        mark_hf_part_content_as_deleted(&mut pkg, part, &settings, false);
                    } else {
                        mark_adopted_hf_content_as_inserted(&mut pkg, part, &settings);
                    }
                    assert_eq!(pkg.part_bytes("word/document.xml"), Some(main.as_slice()));
                    let (mut dom, root) = parsed(&pkg.part_string(part).unwrap());
                    let marker = if deleting { W::del() } else { W::ins() };
                    let marks = dom.descendants(root, Some(&marker));
                    assert!(!marks.is_empty());
                    for mark in marks {
                        assert_eq!(dom.attribute(mark, &W::author()), Some("New owner"));
                        assert_eq!(
                            dom.attribute(mark, &W::date()),
                            Some("2001-02-03T04:05:06Z")
                        );
                    }
                    let result = if deleting {
                        crate::revision_processor::reject_revisions_document(&mut dom, root)
                    } else {
                        crate::revision_processor::accept_revisions_document(&mut dom, root)
                    };
                    assert_eq!(
                        semantic(&dom, result),
                        expected,
                        "footer={footer} deleted={deleting} paragraph-mark-properties={mark_properties} form={form}"
                    );
                }
            }
        }
    }
}

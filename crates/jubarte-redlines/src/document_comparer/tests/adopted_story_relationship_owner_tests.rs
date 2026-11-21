// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;
const MAIN: &str = "word/document.xml";
const REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/";

fn signature(dom: &Dom, node: NodeId) -> String {
    if (dom.name_is(node, &W::r_pr()) || dom.name_is(node, &W::p_pr()))
        && dom.nodes(node).is_empty()
        && dom
            .attributes(node)
            .iter()
            .all(|(name, _)| dom.is_namespace_declaration(name))
    {
        return String::new();
    }
    let mut attrs = dom
        .attributes_iter(node)
        .filter(|(name, _)| !dom.is_namespace_declaration(name))
        .map(|(name, value)| (format!("{name:?}"), value))
        .collect::<Vec<_>>();
    attrs.sort();
    let children = dom
        .nodes(node)
        .into_iter()
        .map(|child| {
            let s = signature(dom, child);
            if s.is_empty() {
                String::new()
            } else {
                format!("{}:{s}", s.len())
            }
        })
        .collect::<String>();
    format!(
        "{:?}:{attrs:?}:{:?}:{children}",
        dom.name(node),
        dom.text_value(node)
    )
}
fn parsed(xml: &str) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).unwrap();
    (dom, root)
}
fn empty_main() -> String {
    format!(
        "<w:document xmlns:w='{}' xmlns:r='{}'><w:body><w:p><w:r><w:t>Body source remains owned</w:t></w:r></w:p><w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr></w:body></w:document>",
        W::URI,
        R::URI
    )
}
#[test]
fn adopted_root_and_nested_stories_keep_colliding_media_relationships_resolvable() {
    let images = PartFs::open(include_bytes!(
        "../../../../../tests/fixtures/relids/image_doc.docx"
    ))
    .unwrap();
    let revised_image = images.part_bytes("word/media/image1.png").unwrap().to_vec();
    let original_package = PartFs::open(include_bytes!(
        "../../../../../tests/fixtures/word_probes/keep_next_picture_kn_1001.docx"
    ))
    .unwrap();
    let original_image = original_package
        .part_bytes("word/media/image1.png")
        .unwrap()
        .to_vec();
    assert_ne!(revised_image, original_image);
    for directory in ["", "word/", "stories/"] {
        for media in [
            "image.png",
            "assets/owned.png",
            "stories/owned.png",
            "word/media/owned.png",
        ] {
            for footer in [false, true] {
                for collision in [false, true] {
                    let kind = if footer { "footer" } else { "header" };
                    let root_name = if footer { "ftr" } else { "hdr" };
                    let part = format!("{directory}owned-{kind}.xml");
                    let mut source = PartFs::open(include_bytes!(
                        "../../../../../tests/fixtures/relids/image_doc.docx"
                    ))
                    .unwrap();
                    source.set_part(MAIN, empty_main().into_bytes());
                    source.set_part(media, revised_image.clone());
                    source.add_content_type_default("png", "image/png");
                    let image_id = source.add_document_relationship(
                        &part,
                        &format!("{REL}image"),
                        &crate::opc::relative_rel_target(&part, media),
                    );
                    let header_xml = format!(
                        "<w:{root_name} xmlns:w='{}' xmlns:r='{}' xmlns:v='urn:schemas-microsoft-com:vml'><w:p><w:pPr><w:spacing w:after='80'/></w:pPr><w:r><w:rPr><w:b/><w:color w:val='123456'/></w:rPr><w:pict><v:rect id='authored-image' style='width:10pt;height:10pt'><v:imagedata r:id='{image_id}'/></v:rect></w:pict></w:r></w:p></w:{root_name}>",
                        W::URI,
                        R::URI
                    );
                    source.set_part(&part, header_xml.clone().into_bytes());
                    source.add_content_type_override(&format!("/{part}"), &format!("application/vnd.openxmlformats-officedocument.wordprocessingml.{kind}+xml"));
                    let ref_id = source.add_document_relationship(
                        MAIN,
                        &format!("{REL}{kind}"),
                        &crate::opc::relative_rel_target(MAIN, &part),
                    );
                    source.set_part(
                        MAIN,
                        empty_main()
                            .replace(
                                "<w:pgSz",
                                &format!(
                                    "<w:{kind}Reference w:type='default' r:id='{ref_id}'/><w:pgSz"
                                ),
                            )
                            .into_bytes(),
                    );
                    let snapshot = source
                        .parts()
                        .into_iter()
                        .map(|name| {
                            let bytes = source.part_bytes(&name).unwrap().to_vec();
                            (name, bytes)
                        })
                        .collect::<Vec<_>>();
                    let source_rels = format!("{:?}", source.read_rels_for(&part).unwrap());
                    let mut out = PartFs::open(include_bytes!(
                        "../../../../../tests/fixtures/relids/image_doc.docx"
                    ))
                    .unwrap();
                    out.set_part(MAIN, empty_main().into_bytes());
                    if collision {
                        out.set_part(media, original_image.clone());
                    } else {
                        out.remove_part(media);
                    }
                    let (mut dom, root) = parsed(&empty_main());
                    let settings = WmlComparerSettings::default();
                    adopt_revised_header_footer(&mut dom, root, &source, &mut out, MAIN, &settings);
                    let body = dom.element(root, &W::body()).unwrap();
                    let section = dom.element(body, &W::sect_pr()).unwrap();
                    let refs = dom.elements(section, Some(&W::name(&format!("{kind}Reference"))));
                    assert_eq!(refs.len(), 1);
                    let rid = dom.attribute(refs[0], &R::id()).unwrap();
                    let main_rels = out.read_rels_for(MAIN).unwrap();
                    let relationship = main_rels.items.iter().find(|r| r.id == rid).unwrap();
                    let adopted = out.resolve_rel_target(MAIN, &relationship.target);
                    assert_eq!(adopted, part);
                    let owned_rels=out.read_rels_for(&adopted).unwrap_or_else(||panic!("lost owning relationships part={part} media={media} collision={collision}"));
                    let image = owned_rels.items.iter().find(|r| r.id == image_id).unwrap();
                    assert_eq!(image.rel_type, format!("{REL}image"));
                    assert_eq!(image.target_mode, None);
                    let copied = out.resolve_rel_target(&adopted, &image.target);
                    assert_eq!(
                        out.part_bytes(&copied),
                        Some(revised_image.as_slice()),
                        "part={part} media={media} collision={collision} target={}",
                        image.target
                    );
                    if collision {
                        assert_ne!(copied, media);
                        assert_eq!(out.part_bytes(media), Some(original_image.as_slice()));
                    } else {
                        assert_eq!(copied, media);
                    }
                    let (mut header_dom, header_root) = parsed(&out.part_string(&adopted).unwrap());
                    let accepted = crate::revision_processor::accept_revisions_document(
                        &mut header_dom,
                        header_root,
                    );
                    let (source_dom, source_root) = parsed(&header_xml);
                    assert_eq!(
                        signature(&header_dom, accepted),
                        signature(&source_dom, source_root)
                    );
                    assert_eq!(
                        format!("{:?}", source.read_rels_for(&part).unwrap()),
                        source_rels
                    );
                    for (name, bytes) in &snapshot {
                        assert_eq!(
                            source.part_bytes(name),
                            Some(bytes.as_slice()),
                            "source mutated {name}"
                        );
                    }
                }
            }
        }
    }
}

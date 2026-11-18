// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;

fn canonical(dom: &Dom, node: NodeId) -> String {
    let mut attrs = dom
        .attributes_iter(node)
        .filter(|(name, _)| !dom.is_namespace_declaration(name))
        .collect::<Vec<_>>();
    attrs.sort_by_key(|(name, value)| {
        (
            name.namespace_name().to_owned(),
            name.local_name().to_owned(),
            *value,
        )
    });
    if attrs.is_empty()
        && dom.nodes(node).is_empty()
        && [W::p_pr(), W::r_pr()]
            .iter()
            .any(|name| dom.name_is(node, name))
    {
        return String::new();
    }
    let mut result = format!("{:?}:{attrs:?}:{:?}", dom.name(node), dom.text_value(node));
    for child in dom.nodes(node) {
        let part = canonical(dom, child);
        if !part.is_empty() {
            result.push_str(&format!("{}:{part}", part.len()));
        }
    }
    result
}

// Numbering IDs are package-local references, not formatting. Resolve the
// complete concrete+abstract definition before comparing either projection.
fn owned_projection(bytes: &[u8]) -> Vec<String> {
    let pkg = PartFs::open(bytes).unwrap();
    let mut dom = Dom::new();
    let document = dom.parse_xdocument(&pkg.part_string("word/document.xml").unwrap());
    let body = dom
        .element(dom.root(document).unwrap(), &W::body())
        .unwrap();
    let numbering = dom.parse_xdocument(&pkg.part_string("word/numbering.xml").unwrap());
    let numbering = dom.root(numbering).unwrap();
    let mut result = Vec::new();
    for child in dom.elements(body, None) {
        if !dom.name_is(child, &W::p()) {
            result.push(canonical(&dom, child));
            continue;
        }
        result.push("begin paragraph".into());
        if let Some(ppr) = dom.element(child, &W::p_pr()) {
            let copy = dom.clone_subtree(ppr);
            if let Some(nid) = dom
                .descendants(copy, Some(&W::name("numId")))
                .first()
                .copied()
            {
                let id = dom.attribute(nid, &W::val()).unwrap().to_owned();
                let nums = dom
                    .elements(numbering, Some(&W::name("num")))
                    .into_iter()
                    .filter(|&num| dom.attribute(num, &W::name("numId")) == Some(id.as_str()))
                    .collect::<Vec<_>>();
                assert_eq!(
                    nums.len(),
                    1,
                    "reference {id} resolves one concrete definition"
                );
                let num = nums[0];
                let abstract_ref = dom.element(num, &W::abstract_num_id()).unwrap();
                let abstract_id = dom.attribute(abstract_ref, &W::val()).unwrap().to_owned();
                let abstracts = dom
                    .elements(numbering, Some(&W::name("abstractNum")))
                    .into_iter()
                    .filter(|&owner| {
                        dom.attribute(owner, &W::abstract_num_id()) == Some(abstract_id.as_str())
                    })
                    .collect::<Vec<_>>();
                assert_eq!(
                    abstracts.len(),
                    1,
                    "concrete definition resolves one abstract definition"
                );
                let abstract_copy = dom.clone_subtree(abstracts[0]);
                dom.set_attribute_value(abstract_copy, &W::abstract_num_id(), None);
                let num_copy = dom.clone_subtree(num);
                dom.set_attribute_value(num_copy, &W::name("numId"), None);
                let abstract_ref_copy = dom.element(num_copy, &W::abstract_num_id()).unwrap();
                dom.set_attribute_value(abstract_ref_copy, &W::val(), Some("resolved"));
                let resolved = format!(
                    "{}:{}",
                    canonical(&dom, num_copy),
                    canonical(&dom, abstract_copy)
                );
                dom.set_attribute_value(nid, &W::val(), Some(&resolved));
            }
            let props = canonical(&dom, copy);
            if !props.is_empty() {
                result.push(props);
            }
        }
        for run in dom.elements(child, Some(&W::r())) {
            let props = dom
                .element(run, &W::r_pr())
                .map(|rpr| canonical(&dom, rpr))
                .unwrap_or_default();
            for payload in dom.elements(run, None) {
                if dom.name_is(payload, &W::r_pr()) {
                    continue;
                }
                if dom.name_is(payload, &W::t()) {
                    result.extend(
                        dom.value(payload)
                            .chars()
                            .map(|ch| format!("text:{ch}:{props}")),
                    );
                } else {
                    result.push(format!("payload:{}:{props}", canonical(&dom, payload)));
                }
            }
        }
        assert!(
            dom.elements(child, None)
                .iter()
                .all(|&node| dom.name_is(node, &W::p_pr()) || dom.name_is(node, &W::r())),
            "clean projection has only authored paragraph owners"
        );
        result.push("end paragraph".into());
    }
    result
}

fn package(
    revised: bool,
    collision: bool,
    replacement: bool,
    changed_indent: bool,
    overrides: bool,
) -> Vec<u8> {
    let mut pkg = PartFs::open(include_bytes!(
        "../../../../../tests/fixtures/relids/image_doc.docx"
    ))
    .unwrap();
    let id = if revised && !collision { 7 } else { 1 };
    let format = if revised { "upperRoman" } else { "decimal" };
    let start = if revised { 3 } else { 1 };
    let override_xml = if overrides {
        format!("<w:lvlOverride w:ilvl='0'><w:startOverride w:val='{start}'/></w:lvlOverride>")
    } else {
        String::new()
    };
    pkg.set_part("word/numbering.xml", format!("<w:numbering xmlns:w='{}'><w:abstractNum w:abstractNumId='{id}'><w:multiLevelType w:val='singleLevel'/><w:lvl w:ilvl='0'><w:start w:val='1'/><w:numFmt w:val='{format}'/><w:lvlText w:val='%1.'/><w:lvlJc w:val='left'/><w:pPr><w:tabs><w:tab w:val='num' w:pos='720'/></w:tabs><w:ind w:left='720' w:hanging='360'/></w:pPr><w:rPr><w:rFonts w:ascii='Calibri' w:hAnsi='Calibri'/></w:rPr></w:lvl></w:abstractNum><w:num w:numId='{id}'><w:abstractNumId w:val='{id}'/>{override_xml}</w:num></w:numbering>", W::URI).into_bytes());
    pkg.add_content_type_override(
        "/word/numbering.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml",
    );
    if !pkg.read_rels_for("word/document.xml").is_some_and(|rels| {
        rels.items
            .iter()
            .any(|rel| rel.rel_type.ends_with("/numbering"))
    }) {
        pkg.add_document_relationship(
            "word/document.xml",
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering",
            "numbering.xml",
        );
    }
    pkg.set_part("word/styles.xml", format!("<w:styles xmlns:w='{}'><w:style w:type='paragraph' w:default='1' w:styleId='Normal'><w:name w:val='Normal'/></w:style></w:styles>", W::URI).into_bytes());
    let side = if replacement {
        if revised {
            "botanical violet glacier"
        } else {
            "archival copper walnut"
        }
    } else {
        "retained independent shared"
    };
    let indent = if revised && changed_indent { 360 } else { 180 };
    let body = (0..3).map(|index| format!("<w:p><w:pPr><w:numPr><w:ilvl w:val='0'/><w:numId w:val='{id}'/></w:numPr><w:spacing w:before='120' w:after='80'/><w:ind w:left='{indent}'/></w:pPr><w:r><w:rPr><w:rFonts w:ascii='Calibri' w:hAnsi='Calibri'/><w:color w:val='123456'/><w:sz w:val='22'/><w:lang w:val='en-US'/></w:rPr><w:t>{side} numbered owner {index} maintains exact source properties and ordered substantive payload</w:t></w:r></w:p>")).collect::<String>();
    pkg.set_part("word/document.xml", format!("<w:document xmlns:w='{}'><w:body>{body}<w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr></w:body></w:document>", W::URI).into_bytes());
    pkg.to_zip().unwrap()
}

#[test]
fn public_colliding_list_definitions_preserve_each_source_level_override_and_paragraph_history() {
    // Word compares unchanged list definitions, while the faithful preset
    // only rewrites B-inserted paragraphs. Its public cases use disjoint
    // IDs, so ownership does not depend on a replaced pilcrow. Exercise supported
    // cases; do not invent a faithful equal-definition tracking contract.
    for word in [false, true] {
        for replacement in [false, true] {
            if !word && !replacement {
                continue;
            }
            for collision in [false, true] {
                if !word && collision {
                    continue;
                }
                for changed_indent in [false, true] {
                    for overrides in [false, true] {
                        let a = package(false, collision, replacement, changed_indent, overrides);
                        let b = package(true, collision, replacement, changed_indent, overrides);
                        let expected = (owned_projection(&a), owned_projection(&b));
                        let settings = WmlComparerSettings {
                            merge_replaced_paragraphs: word,
                            detect_moves: false,
                            ..WmlComparerSettings::default()
                        };
                        let compared = compare_documents_with_settings(&a, &b, &settings).unwrap();
                        for (accepted, source) in [(true, &expected.1), (false, &expected.0)] {
                            let projected = if accepted {
                                accept_revisions(&compared)
                            } else {
                                reject_revisions(&compared)
                            }
                            .unwrap();
                            let actual = owned_projection(&projected);
                            let difference = actual
                                .iter()
                                .zip(source)
                                .position(|(a, b)| a != b)
                                .or_else(|| {
                                    (actual.len() != source.len())
                                        .then_some(actual.len().min(source.len()))
                                });
                            if actual != *source {
                                if let Some(index) = difference {
                                    let left = actual.get(index).map(String::as_str).unwrap_or("");
                                    let right = source.get(index).map(String::as_str).unwrap_or("");
                                    let character = left
                                        .chars()
                                        .zip(right.chars())
                                        .position(|(a, b)| a != b)
                                        .unwrap_or(left.chars().count().min(right.chars().count()));
                                    eprintln!(
                                        "NUMBERING FIRST DIFFERENCE character={character} actual={:?} expected={:?}",
                                        left.chars()
                                            .skip(character.saturating_sub(80))
                                            .take(350)
                                            .collect::<String>(),
                                        right
                                            .chars()
                                            .skip(character.saturating_sub(80))
                                            .take(350)
                                            .collect::<String>()
                                    );
                                }
                                for (owner, bytes) in [
                                    ("original", &a),
                                    ("revised", &b),
                                    ("compared", &compared),
                                    ("projected", &projected),
                                ] {
                                    let pkg = PartFs::open(bytes).unwrap();
                                    for part in ["word/document.xml", "word/numbering.xml"] {
                                        eprintln!(
                                            "NUMBERING OWNER {owner} {part}={}",
                                            pkg.part_string(part).unwrap()
                                        );
                                    }
                                }
                            }
                            assert!(
                                actual == *source,
                                "Word={word} replacement={replacement} collision={collision} changed_indent={changed_indent} overrides={overrides} accept={accepted}: events {}/{} first difference {difference:?} actual={:?} expected={:?}",
                                actual.len(),
                                source.len(),
                                difference
                                    .and_then(|i| actual.get(i))
                                    .map(|s| s.chars().take(450).collect::<String>()),
                                difference
                                    .and_then(|i| source.get(i))
                                    .map(|s| s.chars().take(450).collect::<String>())
                            );
                        }
                    }
                }
            }
        }
    }
}

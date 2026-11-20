// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;

const DATE: &str = "2026-01-02T03:04:05Z";
const FORMAT: &str = "<w:rPr><w:b/><w:color w:val='123456'/><w:lang w:val='en-US'/></w:rPr>";
const TABLE: &str = "<w:tbl><w:tblPr><w:tblW w:w='1800' w:type='dxa'/></w:tblPr><w:tblGrid><w:gridCol w:w='1800'/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w='1800' w:type='dxa'/></w:tcPr><w:p><w:r><w:t>Independent table payload</w:t></w:r></w:p></w:tc></w:tr></w:tbl>";
fn settings() -> WmlComparerSettings {
    WmlComparerSettings {
        author_for_revisions: "Boundary editor".into(),
        date_time_for_revisions: DATE.into(),
        ..WmlComparerSettings::default()
    }
}
fn package(body: &str) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let document = dom.parse_xdocument(&format!(
        "<w:document xmlns:w='{}' xmlns:pt='{}' xmlns:r='{}'><w:body>{body}</w:body></w:document>",
        W::URI,
        PT::URI,
        R::URI
    ));
    let root = dom.root(document).unwrap();
    (dom, root)
}
fn semantic(dom: &Dom, node: NodeId) -> String {
    if !dom.is_element(node) {
        return format!("text:{:?}", dom.text_value(node));
    }
    let mut attrs = dom
        .attributes_iter(node)
        .filter(|(name, _)| !dom.is_namespace_declaration(name))
        .map(|(name, value)| {
            (
                name.namespace_name().to_owned(),
                name.local_name().to_owned(),
                value,
            )
        })
        .collect::<Vec<_>>();
    attrs.sort();
    format!(
        "{:?}{attrs:?}[{}]",
        dom.name(node),
        dom.nodes(node)
            .into_iter()
            .map(|child| semantic(dom, child))
            .collect::<String>()
    )
}
fn expected(dom: &Dom, root: NodeId, body: &str, label: &str) {
    let (oracle, oracle_root) = package(body);
    assert_eq!(
        semantic(dom, root),
        semantic(&oracle, oracle_root),
        "{label}"
    );
}
fn run(kind: &str, text: &str) -> String {
    let leaf = if kind == "del" { "delText" } else { "t" };
    let r = format!("<w:r>{FORMAT}<w:{leaf} xml:space='preserve'>{text}</w:{leaf}></w:r>");
    if kind == "live" {
        r
    } else {
        format!("<w:{kind} w:id='17' w:author='Source editor' w:date='{DATE}'>{r}</w:{kind}>")
    }
}
fn para(kind: &str, text: &str, props: &str) -> String {
    format!("<w:p><w:pPr>{props}</w:pPr>{}</w:p>", run(kind, text))
}
fn marked(kind: &str, text: &str, props: &str) -> String {
    {
        let mark = format!(
            "<w:rPr><w:{kind} w:id='18' w:author='Source editor' w:date='{DATE}'/></w:rPr>"
        );
        let properties = if props.contains("<w:pPrChange") {
            props.replacen("<w:pPrChange", &(mark + "<w:pPrChange"), 1)
        } else {
            format!("{props}{mark}")
        };
        para(kind, text, &properties)
    }
}
fn stable(dom: &mut Dom, root: NodeId, transform: impl Fn(&mut Dom, NodeId)) {
    let once = semantic(dom, root);
    transform(dom, root);
    assert_eq!(
        semantic(dom, root),
        once,
        "second application must preserve established ownership"
    );
}

#[test]
fn explicit_default_direction_is_removed_only_from_inserted_layout() {
    for status in ["ins", "del", "live"] {
        for (style, kept_style) in [
            ("Normal", false),
            ("NORMAL", false),
            ("BodyText", true),
            ("", true),
        ] {
            for (value, kept_bidi) in [
                (Some("0"), false),
                (Some("FALSE"), false),
                (Some("false"), false),
                (Some("1"), true),
                (Some("true"), true),
                (None, true),
            ] {
                let style_xml = if style.is_empty() {
                    String::new()
                } else {
                    format!("<w:pStyle w:val='{style}'/>")
                };
                let bidi = value
                    .map(|value| format!("<w:bidi w:val='{value}'/>"))
                    .unwrap_or_else(|| "<w:bidi/>".to_string());
                let own = "<w:spacing w:after='80'/><w:ind w:left='120'/><w:contextualSpacing/>";
                let history = format!(
                    "<w:pPrChange w:id='11' w:author='Prior editor' w:date='{DATE}'><w:pPr><w:jc w:val='right'/></w:pPr></w:pPrChange>"
                );
                let source = para(
                    status,
                    "Owned directional text",
                    &format!("{style_xml}{bidi}{own}{history}"),
                ) + TABLE;
                let (mut dom, root) = package(&source);
                strip_redundant_normal_pstyle_and_bidi(&mut dom, root);
                let expected_style = if status == "ins" && !kept_style {
                    ""
                } else {
                    &style_xml
                };
                let expected_bidi = if status == "ins" && !kept_bidi {
                    ""
                } else {
                    &bidi
                };
                let oracle = para(
                    status,
                    "Owned directional text",
                    &format!("{expected_style}{expected_bidi}{own}{history}"),
                ) + TABLE;
                expected(&dom, root, &oracle, &format!("{status}/{style}/{value:?}"));
                stable(&mut dom, root, strip_redundant_normal_pstyle_and_bidi);
            }
        }
    }
}

#[test]
fn demo_spacing_respects_authored_metrics_history_and_source_defaults() {
    // Expected outcomes are Word's recorded restatement versus authored
    // metric contracts (M352/M358/M391/M67), not a generated gate oracle.
    let metrics = [
        ("line='276'", true),
        ("line='276' after='200' lineRule='auto'", true),
        ("line='275'", false),
        ("line='276' after='201'", false),
        ("line='276' before='1'", false),
        ("line='276' lineRule='exact'", false),
    ];
    for status in ["ins", "del", "live"] {
        for default in [None, Some("276"), Some("240")] {
            for (attrs, restatement) in metrics {
                for old_same in [false, true] {
                    let sp = format!("<w:spacing w:{}/>", attrs.replace(" ", " w:"));
                    let old_sp = if old_same {
                        sp.clone()
                    } else {
                        "<w:spacing w:line='360' w:lineRule='exact'/>".to_string()
                    };
                    let history = format!(
                        "<w:pPrChange w:id='11' w:author='Prior editor' w:date='{DATE}'><w:pPr><w:keepNext/>{old_sp}</w:pPr></w:pPrChange>"
                    );
                    let source = para(
                        status,
                        "Several independently authored words",
                        &format!("{sp}{history}"),
                    ) + TABLE;
                    let (mut dom, root) = package(&source);
                    strip_redundant_demo_default_spacing(&mut dom, root, (default, default));
                    let removed = restatement && default != Some("240");
                    let old = if removed && old_same { "" } else { &old_sp };
                    let expected_history = format!(
                        "<w:pPrChange w:id='11' w:author='Prior editor' w:date='{DATE}'><w:pPr><w:keepNext/>{old}</w:pPr></w:pPrChange>"
                    );
                    let live = if removed { "" } else { &sp };
                    expected(
                        &dom,
                        root,
                        &(para(
                            status,
                            "Several independently authored words",
                            &format!("{live}{expected_history}"),
                        ) + TABLE),
                        "demo source/metric/history boundary",
                    );
                    stable(&mut dom, root, |d, r| {
                        strip_redundant_demo_default_spacing(d, r, (default, default));
                    });
                }
            }
        }
    }
    for (text, props, kept) in [
        ("PARTIES", "<w:spacing w:line='276'/>", true),
        ("", "<w:spacing w:line='276'/>", true),
        (
            "Product line",
            "<w:spacing w:line='276'/><w:ind w:right='-30'/>",
            true,
        ),
        (
            "PARTIES",
            "<w:pStyle w:val='Title'/><w:spacing w:line='276'/>",
            false,
        ),
    ] {
        let (mut dom, root) = package(&para("ins", text, props));
        strip_redundant_demo_default_spacing(&mut dom, root, (None, None));
        assert_eq!(
            dom.descendants(root, Some(&W::spacing_el())).len(),
            usize::from(kept)
        );
        assert_eq!(dom.value(root), text);
    }
}

#[test]
fn an_equal_alignment_change_is_attributed_once_and_never_claims_source_history() {
    for status in ["live", "ins", "del"] {
        for source_owns in [false, true] {
            for obstruction in [
                "",
                "<w:rPr><w:b/></w:rPr>",
                "<w:pPrChange w:id='9' w:author='Prior editor'><w:pPr><w:jc w:val='left'/></w:pPr></w:pPrChange>",
            ] {
                let props = format!("<w:jc w:val='right'/><w:spacing w:after='80'/>{obstruction}");
                let source = para(status, "Alignment ownership", &props) + TABLE;
                let (mut dom, root) = package(&source);
                let mut original = HashSet::new();
                if source_owns {
                    original.insert(("Alignment ownership".into(), "right".into()));
                }
                let mut next = 42;
                ensure_empty_pprchange_on_eq_with_live_jc(
                    &mut dom,
                    root,
                    &original,
                    &settings(),
                    &mut next,
                );
                let added = status == "live" && !source_owns && obstruction.is_empty();
                let history = if added {
                    format!(
                        "<w:pPrChange w:id='42' w:author='Boundary editor' w:date='{DATE}'><w:pPr/></w:pPrChange>"
                    )
                } else {
                    String::new()
                };
                expected(
                    &dom,
                    root,
                    &(para(status, "Alignment ownership", &format!("{props}{history}")) + TABLE),
                    "equal alignment attribution",
                );
                assert_eq!(next, 42 + u32::from(added));
                let once = semantic(&dom, root);
                ensure_empty_pprchange_on_eq_with_live_jc(
                    &mut dom,
                    root,
                    &original,
                    &settings(),
                    &mut next,
                );
                assert_eq!(semantic(&dom, root), once);
            }
        }
    }
}

#[test]
fn the_final_empty_spacing_history_has_one_source_owner() {
    for content in [
        "",
        "<w:r><w:t>Authored content</w:t></w:r>",
        "<w:ins w:id='7'><w:r><w:t>Inserted content</w:t></w:r></w:ins>",
    ] {
        for other in [
            "",
            "<w:keepNext/>",
            "<w:pPrChange w:id='9' w:author='Prior editor'><w:pPr><w:spacing w:after='400'/></w:pPr></w:pPrChange>",
        ] {
            for same_revised in [false, true] {
                let spacing = "<w:spacing w:before='120' w:after='80'/>";
                let p = format!("<w:p><w:pPr>{spacing}<w:rPr/>{other}</w:pPr>{content}</w:p>");
                let body =
                    format!("{TABLE}{p}<w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr>");
                let (mut dom, root) = package(&body);
                let attrs = vec![
                    (W::URI.into(), "after".into(), "80".into()),
                    (W::URI.into(), "before".into(), "120".into()),
                ];
                let mut next = 42;
                trailing_empty_spacing_to_pprchange(
                    &mut dom,
                    root,
                    same_revised.then_some(attrs.as_slice()),
                    &settings(),
                    &mut next,
                );
                let added = content.is_empty() && other.is_empty() && !same_revised;
                let oracle = if added {
                    format!(
                        "{TABLE}<w:p><w:pPr><w:pPrChange w:id='42' w:author='Boundary editor' w:date='{DATE}'><w:pPr>{spacing}</w:pPr></w:pPrChange></w:pPr></w:p><w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr>"
                    )
                } else {
                    body
                };
                expected(&dom, root, &oracle, "final empty spacing source ownership");
                assert_eq!(next, 42 + u32::from(added));
                let once = semantic(&dom, root);
                trailing_empty_spacing_to_pprchange(
                    &mut dom,
                    root,
                    same_revised.then_some(attrs.as_slice()),
                    &settings(),
                    &mut next,
                );
                assert_eq!(semantic(&dom, root), once);
            }
        }
    }
}

#[test]
fn preexisting_insertions_keep_innermost_provenance_across_real_wrappers() {
    for wrapper in ["plain", "hyperlink", "sdt", "smartTag"] {
        for outer_author in [None, Some("Outer editor")] {
            for outer_date in [None, Some(DATE)] {
                let provenance = format!(
                    "{}{}",
                    outer_author
                        .map(|a| format!(" w:author='{a}'"))
                        .unwrap_or_default(),
                    outer_date
                        .map(|d| format!(" w:date='{d}'"))
                        .unwrap_or_default()
                );
                let own = format!("<w:r>{FORMAT}<w:t>Outer owned text</w:t></w:r>");
                let inner = format!(
                    "<w:ins w:id='21' w:author='Inner editor' w:date='{DATE}'><w:r>{FORMAT}<w:t>Inner owned text</w:t></w:r></w:ins>"
                );
                let deletion = format!(
                    "<w:del w:id='22' w:author='Earlier editor' w:date='{DATE}'><w:r>{FORMAT}<w:delText>Historical deletion</w:delText></w:r></w:del>"
                );
                let payload = format!("{own}{inner}{deletion}");
                let wrapped = match wrapper {
                    "hyperlink" => format!("<w:hyperlink r:id='rId7'>{payload}</w:hyperlink>"),
                    "sdt" => format!(
                        "<w:sdt><w:sdtPr><w:tag w:val='Owned clause'/><w:id w:val='31'/></w:sdtPr><w:sdtContent>{payload}</w:sdtContent></w:sdt>"
                    ),
                    "smartTag" => format!(
                        "<w:smartTag w:uri='urn:contract' w:element='clause'>{payload}</w:smartTag>"
                    ),
                    _ => payload,
                };
                let source = format!(
                    "<w:p><w:pPr><w:keepNext/><w:rPr><w:ins w:id='18'{provenance}/></w:rPr></w:pPr><w:ins w:id='20'{provenance}>{wrapped}</w:ins></w:p>{TABLE}"
                );
                let (mut dom, root) = package(&source);
                let body = dom.element(root, &W::body()).unwrap();
                let table_before = semantic(&dom, dom.element(body, &W::tbl()).unwrap());
                flatten_tracked_insertions_stamped(&mut dom, body);
                let runs = dom.descendants(root, Some(&W::r()));
                for &r in &runs {
                    let text = dom.value(r);
                    if text == "Outer owned text" || text == "Inner owned text" {
                        assert_eq!(dom.attribute(r, &PT::name("PreIns")), Some("1"));
                        assert_eq!(
                            dom.attribute(r, &PT::name("PreInsAuthor")),
                            if text == "Inner owned text" {
                                Some("Inner editor")
                            } else {
                                outer_author
                            }
                        );
                        assert_eq!(
                            dom.attribute(r, &PT::name("PreInsDate")),
                            if text == "Inner owned text" {
                                Some(DATE)
                            } else {
                                outer_date
                            }
                        );
                        expected_run_format(&dom, r);
                    } else {
                        assert!(dom.attribute(r, &PT::name("PreIns")).is_none());
                    }
                }
                assert_eq!(dom.descendants(root, Some(&W::ins())).len(), 0);
                let historical = dom.descendants(root, Some(&W::del()))[0];
                assert_eq!(
                    dom.attribute(historical, &W::author()),
                    Some("Earlier editor")
                );
                assert_eq!(dom.value(historical), "Historical deletion");
                assert_eq!(
                    semantic(&dom, dom.element(body, &W::tbl()).unwrap()),
                    table_before
                );
                stable(&mut dom, root, |d, r| {
                    let body = d.element(r, &W::body()).unwrap();
                    flatten_tracked_insertions_stamped(d, body);
                });
            }
        }
    }
}
fn expected_run_format(dom: &Dom, run: NodeId) {
    let (oracle, root) = package(&format!("<w:p><w:r>{FORMAT}<w:t>x</w:t></w:r></w:p>"));
    assert_eq!(
        semantic(dom, dom.element(run, &W::r_pr()).unwrap()),
        semantic(&oracle, oracle.descendants(root, Some(&W::r_pr()))[0])
    );
}

#[test]
fn a_pending_insertion_splits_deleted_siblings_without_stealing_their_payloads() {
    for placement in [0usize, 1, 2] {
        for outer in ["del", "ins", "live"] {
            for metadata in [false, true] {
                let mut runs = Vec::new();
                for index in 0..3 {
                    let stamp = if index == placement {
                        format!(
                            " pt:PreIns='1'{}",
                            if metadata {
                                format!(" pt:PreInsAuthor='Original editor' pt:PreInsDate='{DATE}'")
                            } else {
                                String::new()
                            }
                        )
                    } else {
                        String::new()
                    };
                    let leaf = if outer == "del" { "delText" } else { "t" };
                    runs.push(format!(
                        "<w:r{stamp}>{FORMAT}<w:{leaf}>Source payload {index}</w:{leaf}></w:r>"
                    ));
                }
                let content = runs.concat();
                let content = if outer == "live" {
                    content
                } else {
                    format!(
                        "<w:{outer} w:id='17' w:author='Comparison editor' w:date='{DATE}'>{content}</w:{outer}>"
                    )
                };
                let (mut dom, root) = package(&format!(
                    "<w:p><w:pPr><w:keepNext/></w:pPr>{content}</w:p>{TABLE}"
                ));
                let mut next = 42;
                convert_stamped_preins(&mut dom, root, &settings(), &mut next);
                let ordered = dom
                    .descendants(root, Some(&W::r()))
                    .into_iter()
                    .filter(|&r| dom.value(r).starts_with("Source payload"))
                    .collect::<Vec<_>>();
                assert_eq!(ordered.len(), 3);
                for (index, &r) in ordered.iter().enumerate() {
                    assert_eq!(dom.value(r), format!("Source payload {index}"));
                    expected_run_format(&dom, r);
                    let parent = dom.parent(r).unwrap();
                    if index == placement {
                        assert!(dom.name_is(parent, &W::ins()));
                        assert_eq!(
                            dom.attribute(parent, &W::author()),
                            Some(if metadata {
                                "Original editor"
                            } else if outer == "ins" {
                                "Comparison editor"
                            } else {
                                "Boundary editor"
                            })
                        );
                        assert_eq!(dom.attribute(parent, &W::date()), Some(DATE));
                        assert!(dom.descendants(r, Some(&W::del_text())).is_empty());
                        assert!(
                            dom.attributes(r)
                                .iter()
                                .all(|(name, _)| name.namespace_name() != PT::URI)
                        );
                    } else if outer == "del" {
                        assert!(dom.name_is(parent, &W::del()));
                        assert_eq!(
                            dom.attribute(parent, &W::author()),
                            Some("Comparison editor")
                        );
                    }
                }
                let once = semantic(&dom, root);
                let end = next;
                convert_stamped_preins(&mut dom, root, &settings(), &mut next);
                assert_eq!(semantic(&dom, root), once);
                assert_eq!(next, end);
            }
        }
    }
}

#[test]
fn move_simplification_changes_only_lifetime_names_and_range_markers() {
    for direction in ["moveFrom", "moveTo"] {
        for nested in [false, true] {
            for metadata in [false, true] {
                let attrs = if metadata {
                    format!(" w:id='17' w:author='Move editor' w:date='{DATE}'")
                } else {
                    " w:id='17' w:author='Move editor'".into()
                };
                let leaf = if direction == "moveFrom" {
                    "delText"
                } else {
                    "t"
                };
                let payload = format!(
                    "<w:r w:rsidR='00112233'>{FORMAT}<w:{leaf} xml:space='preserve'> Moved source payload </w:{leaf}></w:r>"
                );
                let wrapper = if nested {
                    format!(
                        "<w:smartTag w:uri='urn:contract' w:element='clause'>{payload}</w:smartTag>"
                    )
                } else {
                    payload
                };
                let markers = "<w:moveFromRangeStart w:id='17' w:name='MoveClause'/><w:moveFromRangeEnd w:id='17'/><w:moveToRangeStart w:id='18' w:name='MoveClause'/><w:moveToRangeEnd w:id='18'/>";
                let input = format!(
                    "<w:p w:rsidP='AABBCCDD'><w:pPr><w:keepNext/><w:spacing w:after='80'/></w:pPr>{markers}<w:{direction}{attrs}>{wrapper}</w:{direction}><w:r><w:t>Untouched anchor</w:t></w:r></w:p>{TABLE}"
                );
                let (mut dom, root) = package(&input);
                let before = semantic(&dom, root);
                let result = simplify_move_markup_to_del_ins(&mut dom, root);
                let lifetime = if direction == "moveFrom" {
                    "del"
                } else {
                    "ins"
                };
                let oracle = format!(
                    "<w:p w:rsidP='AABBCCDD'><w:pPr><w:keepNext/><w:spacing w:after='80'/></w:pPr><w:{lifetime}{attrs}>{wrapper}</w:{lifetime}><w:r><w:t>Untouched anchor</w:t></w:r></w:p>{TABLE}"
                );
                expected(
                    &dom,
                    result,
                    &oracle,
                    "move lifetime conversion preserves complete descendants and geometry",
                );
                assert_eq!(
                    semantic(&dom, root),
                    before,
                    "functional transform does not consume its input"
                );
                let second = simplify_move_markup_to_del_ins(&mut dom, result);
                assert_eq!(semantic(&dom, second), semantic(&dom, result));
            }
        }
    }
}

#[test]
fn equal_shared_labels_cannot_cross_opaque_or_historical_run_ownership() {
    let old = "<w:del w:id='17' w:author='Source editor'><w:r><w:rPr><w:b/></w:rPr><w:delText>Lvl 1 – a</w:delText></w:r></w:del>";
    let plain = "<w:ins w:id='19' w:author='Source editor'><w:r><w:rPr><w:b/></w:rPr><w:t>a </w:t></w:r></w:ins>";
    for obstacle in [
        "nested-hyperlink",
        "nested-control",
        "foreign-property",
        "historical-property",
        "drawing",
        "tab",
        "break",
        "intermediate-live",
        "alternating-lifetimes",
        "different-property-untracked",
        "trailing-original-space",
        "different-casing",
        "long-label",
        "nonmatching-label",
        "short-original",
    ] {
        let mut inserted = plain.to_string();
        let mut deleted = old.to_string();
        let mut tracking = true;
        match obstacle {
                "nested-hyperlink"=>inserted=format!("<w:hyperlink r:id='rId7'>{plain}</w:hyperlink>"),
                "nested-control"=>inserted=format!("<w:sdt><w:sdtPr><w:tag w:val='Protected label'/><w:id w:val='31'/></w:sdtPr><w:sdtContent>{plain}</w:sdtContent></w:sdt>"),
                "foreign-property"=>inserted=plain.replace("<w:b/>","<x:metadata xmlns:x='urn:contract' x:owner='A'/>") ,
                "historical-property"=>inserted=plain.replace("<w:b/>","<w:rPrChange w:id='7' w:author='Prior editor'><w:rPr><w:i/></w:rPr></w:rPrChange>"),
                "drawing"=>inserted=plain.replace("<w:t>","<w:drawing><wp:inline xmlns:wp='http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing'><wp:extent cx='127000' cy='127000'/><wp:docPr id='41' name='Owned graphic'/><a:graphic xmlns:a='http://schemas.openxmlformats.org/drawingml/2006/main'><a:graphicData uri='urn:boundary-graphic'/></a:graphic></wp:inline></w:drawing><w:t>"),
                "tab"=>inserted=plain.replace("<w:t>","<w:tab/><w:t>"),
                "break"=>inserted=plain.replace("<w:t>","<w:br/><w:t>"),
                "intermediate-live"=>inserted=plain.to_string()+"<w:r><w:t>Authored anchor</w:t></w:r>",
                "alternating-lifetimes"=>inserted=plain.to_string()+&old.replace("Lvl 1 – a","Original prefix ")+plain,
                "different-property-untracked"=>{inserted=plain.replace("<w:b/>","<w:i/>");tracking=false;},
                "trailing-original-space"=>deleted=old.replace("Lvl 1 – a","Lvl 1 – a "),
                "different-casing"=>deleted=old.replace("Lvl 1 – a","Lvl 1 – A"),
                "long-label"=>{inserted=plain.replace("a ","item ");deleted=old.replace("Lvl 1 – a","Lvl 1 – item");},
                "nonmatching-label"=>deleted=old.replace("Lvl 1 – a","Lvl 1 – b"),
                "short-original"=>deleted=old.replace("Lvl 1 – a","Item a"),
                _=>unreachable!(),
            }
        let body = format!(
            "<w:p><w:pPr><w:keepNext/><w:rPr><w:del w:id='11' w:author='Paragraph owner'/></w:rPr></w:pPr>{inserted}{deleted}</w:p>{TABLE}"
        );
        let (mut dom, root) = package(&body);
        let p = dom.descendants(root, Some(&W::p()))[0];
        let before = semantic(&dom, root);
        let mut configured = settings();
        configured.detect_format_changes = tracking;
        mesh_short_label_shared_eq_with_settings(&mut dom, p, &configured);
        assert_eq!(
            semantic(&dom, root),
            before,
            "opaque or unmatched label owner must remain unchanged: {obstacle}"
        );
    }
}

#[test]
fn whitespace_layout_before_a_deleted_block_is_not_empty_content() {
    // Each preserved fixture corresponds to a documented Word exception:
    // layout payloads, inserted structures after a blank, short-title
    // spacers, and three-blank title-page/wholesale runs.
    for payload in [
        "<w:br w:type='page'/>",
        "<w:drawing><wp:inline xmlns:wp='http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing'><wp:extent cx='127000' cy='127000'/><wp:docPr id='41' name='Owned graphic'/><a:graphic xmlns:a='http://schemas.openxmlformats.org/drawingml/2006/main'><a:graphicData uri='urn:boundary-graphic'/></a:graphic></wp:inline></w:drawing>",
        "<w:pict><v:rect xmlns:v='urn:schemas-microsoft-com:vml' id='OwnedRect' style='width:12pt;height:12pt'/></w:pict>",
        "<w:object><v:shape xmlns:v='urn:schemas-microsoft-com:vml' id='OwnedObject' style='width:12pt;height:12pt'/></w:object>",
    ] {
        let spacer = format!(
            "<w:p><w:pPr><w:spacing w:after='80'/><w:rPr><w:ins w:id='3' w:author='Layout editor'/></w:rPr></w:pPr><w:ins w:id='4' w:author='Layout editor'><w:r>{FORMAT}{payload}</w:r></w:ins></w:p>"
        );
        let deleted = marked(
            "del",
            "Original substantive contract obligation remains valid",
            "<w:pStyle w:val='BodyText'/>",
        );
        let input = spacer + &deleted + TABLE;
        let (mut dom, root) = package(&input);
        let before = semantic(&dom, root);
        fold_whitespace_pure_ins_into_following_pure_del(&mut dom, root);
        assert_eq!(
            semantic(&dom, root),
            before,
            "layout {payload} must keep independent lifetime and source properties"
        );
    }
    for fixture in [
        "inserted-block-after",
        "short-title",
        "wholesale-three-blanks",
        "title-three-blanks",
    ] {
        let blank = "<w:p><w:pPr><w:spacing w:after='80'/><w:rPr><w:ins w:id='3' w:author='Layout editor'/></w:rPr></w:pPr></w:p>";
        let input = match fixture {
            "inserted-block-after" => {
                blank.to_owned()
                    + &marked(
                        "del",
                        "Original substantive contract obligation remains valid",
                        "",
                    )
                    + &marked("ins", "Later inserted structure", "")
                    + TABLE
            }
            "short-title" => {
                blank.to_owned()
                    + &marked("del", "Contract Review", "<w:pStyle w:val='Title'/>")
                    + TABLE
            }
            "wholesale-three-blanks" => {
                blank.repeat(3)
                    + &marked(
                        "del",
                        "Original substantive contract obligation remains valid",
                        "",
                    )
                    + TABLE
            }
            _ => {
                marked("ins", "Revised cover date", "")
                    + &blank.repeat(3)
                    + &marked(
                        "del",
                        "Original substantive contract obligation remains valid",
                        "",
                    )
                    + TABLE
            }
        };
        let (mut dom, root) = package(&input);
        let before = semantic(&dom, root);
        fold_whitespace_pure_ins_into_following_pure_del(&mut dom, root);
        assert_eq!(semantic(&dom, root), before, "{fixture}");
    }
}

#[test]
fn list_layout_contamination_does_not_erase_real_list_or_terminal_ownership() {
    let list_space = "<w:spacing w:after='0' w:line='240' w:lineRule='auto'/>";
    for donor in [true, false] {
        for structure in [
            "",
            "<w:pStyle w:val='ListParagraph'/>",
            "<w:numPr><w:ilvl w:val='1'/><w:numId w:val='7'/></w:numPr>",
            "<w:pStyle w:val='BodyText'/>",
        ] {
            for alignment in ["both", "distribute", "center", "right"] {
                let donor_props = if donor {
                    list_space
                } else {
                    "<w:spacing w:line='360' w:lineRule='exact'/>"
                };
                let first = marked("ins", "List donor", donor_props);
                let props = format!("{structure}{list_space}<w:jc w:val='{alignment}'/>");
                let mid = marked("del", "Independent original paragraph", &props);
                let last = marked("del", "Terminal original paragraph", &props);
                let (mut dom, root) = package(&(first.clone() + &mid + &last));
                strip_list_layout_from_mid_pure_del(&mut dom, root);
                let protected =
                    !donor || structure.contains("ListParagraph") || structure.contains("numPr");
                let middle = if protected {
                    mid
                } else {
                    marked(
                        "del",
                        "Independent original paragraph",
                        &format!(
                            "{structure}{}",
                            if alignment == "both" || alignment == "distribute" {
                                String::new()
                            } else {
                                format!("<w:jc w:val='{alignment}'/>")
                            }
                        ),
                    )
                };
                expected(
                    &dom,
                    root,
                    &(first + &middle + &last),
                    "list contamination versus authored structure/terminal carrier",
                );
                stable(&mut dom, root, strip_list_layout_from_mid_pure_del);
            }
        }
    }
}

#[test]
fn faithful_merged_closing_carrier_records_both_complete_property_sources() {
    for tracking in [false, true] {
        for lifetime in ["del", "ins", "live"] {
            for content in ["both", "original", "revised"] {
                for same in [false, true] {
                    let old_props =
                        "<w:pStyle w:val='BodyText'/><w:keepNext/><w:spacing w:after='80'/>";
                    let new_props = if same {
                        old_props
                    } else {
                        "<w:pStyle w:val='Heading1'/><w:keepLines/><w:spacing w:before='240'/>"
                    };
                    let mark = if lifetime == "live" {
                        String::new()
                    } else {
                        format!("<w:rPr><w:{lifetime} w:id='18' w:author='Source editor'/></w:rPr>")
                    };
                    let texts = match content {
                        "both" => {
                            run("ins", "Revised owned payload")
                                + &run("del", "Original owned payload")
                        }
                        "original" => run("del", "Original owned payload"),
                        _ => run("ins", "Revised owned payload"),
                    };
                    let input = format!(
                        "{TABLE}<w:p><w:pPr>{old_props}{mark}</w:pPr>{texts}</w:p><w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr>"
                    );
                    let (mut dom, root) = package(&input);
                    let source=dom.parse_xdocument(&format!("<w:source xmlns:w='{}'><w:pPr>{old_props}</w:pPr><w:pPr>{new_props}</w:pPr></w:source>",W::URI));
                    let source_root = dom.root(source).unwrap();
                    let property_sources = dom.elements(source_root, Some(&W::p_pr()));
                    let before = semantic(&dom, source_root);
                    let mut configured = settings();
                    configured.detect_format_changes = tracking;
                    let mut next = 42;
                    merged_closing_carrier_takes_source_properties(
                        &mut dom,
                        root,
                        Some(property_sources[0]),
                        Some(property_sources[1]),
                        &configured,
                        &mut next,
                    );
                    let repair = tracking && lifetime == "del" && content == "both" && !same;
                    let oracle = if repair {
                        format!(
                            "{TABLE}<w:p><w:pPr>{mark}{new_props}<w:pPrChange w:id='42' w:author='Boundary editor' w:date='{DATE}'><w:pPr>{old_props}</w:pPr></w:pPrChange></w:pPr>{texts}</w:p><w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr>"
                        )
                    } else {
                        input
                    };
                    expected(
                        &dom,
                        root,
                        &oracle,
                        "faithful closing property owner and setting boundary",
                    );
                    assert_eq!(next, 42 + u32::from(repair));
                    assert_eq!(semantic(&dom, source_root), before);
                    let once = semantic(&dom, root);
                    merged_closing_carrier_takes_source_properties(
                        &mut dom,
                        root,
                        Some(property_sources[0]),
                        Some(property_sources[1]),
                        &configured,
                        &mut next,
                    );
                    assert_eq!(
                        semantic(&dom, root),
                        once,
                        "saved complete original is not replaced on a second pass"
                    );
                }
            }
        }
    }
}

#[test]
fn the_short_list_label_uses_only_an_empty_deleted_carrier_with_continuation() {
    let numbered = "<w:numPr><w:ilvl w:val='0'/><w:numId w:val='7'/></w:numPr>";
    for (label, folds) in [
        ("a", true),
        ("Item two", true),
        ("Three separate words", false),
        ("", false),
    ] {
        for empty in [true, false] {
            for continuation in [true, false] {
                let first = marked("ins", label, numbered);
                let old_props = "<w:spacing w:after='80'/><w:rPr><w:del w:id='18' w:author='Source editor' w:date='2026-01-02T03:04:05Z'/></w:rPr>";
                let middle = if empty {
                    format!("<w:p><w:pPr>{old_props}</w:pPr></w:p>")
                } else {
                    marked("del", "Original owned text", "<w:spacing w:after='80'/>")
                };
                let last = if continuation {
                    marked("del", "Later original owned text", "")
                } else {
                    para("live", "Unchanged continuation", "")
                };
                let input = first.clone() + &middle + &last + TABLE;
                let (mut dom, root) = package(&input);
                fold_short_list_label_into_empty_pure_del(&mut dom, root);
                let oracle = if folds && empty && continuation {
                    format!(
                        "<w:p><w:pPr>{old_props}</w:pPr>{}</w:p>{last}{TABLE}",
                        run("ins", label)
                    )
                } else {
                    input
                };
                expected(
                    &dom,
                    root,
                    &oracle,
                    "short label and independently deleted empty mark",
                );
                stable(&mut dom, root, fold_short_list_label_into_empty_pure_del);
            }
        }
    }
}

#[test]
fn historical_legal_numbering_uses_the_donor_id_and_keeps_authored_old_level() {
    for (text, legal) in [
        ("14.11Survival obligations", true),
        (" 3.01Continuation", true),
        ("Num 4", true),
        ("Num 4.", true),
        ("1. Item", false),
        ("Num A", false),
        ("Ordinary original paragraph", false),
    ] {
        for donor in [true, false] {
            for protected in [false, true] {
                for old_level in [None, Some("2")] {
                    let source_num = "<w:numPr><w:ilvl w:val='0'/><w:numId w:val='7'/></w:numPr>";
                    let source = marked(
                        "ins",
                        if donor {
                            "a"
                        } else {
                            "Several revised heading words"
                        },
                        source_num,
                    );
                    let level = old_level
                        .map(|level| format!("<w:ilvl w:val='{level}'/>"))
                        .unwrap_or_default();
                    let parked = format!(
                        "<w:pPrChange w:id='11' w:author='Original property editor' w:date='{DATE}'><w:pPr><w:pStyle w:val='StandardL1'/><w:numPr>{level}<w:numId w:val='0'/></w:numPr></w:pPr></w:pPrChange>"
                    );
                    let live = if protected {
                        "<w:pStyle w:val='ListParagraph'/>"
                    } else {
                        "<w:keepNext/>"
                    };
                    let target = marked("del", text, &format!("{live}{parked}"));
                    let input = source.clone() + &target + TABLE;
                    let (mut dom, root) = package(&input);
                    promote_live_numpr_on_pure_d_from_pprchange(&mut dom, root);
                    let repaired = donor && legal && !protected;
                    let oracle = if repaired {
                        let adopted = format!(
                            "<w:numPr><w:ilvl w:val='{}'/><w:numId w:val='7'/></w:numPr>",
                            old_level.unwrap_or("0")
                        );
                        source + &marked("del", text, &format!("{adopted}{live}{parked}")) + TABLE
                    } else {
                        input
                    };
                    expected(
                        &dom,
                        root,
                        &oracle,
                        "legal numbering donor versus parked original properties",
                    );
                    stable(&mut dom, root, promote_live_numpr_on_pure_d_from_pprchange);
                }
            }
        }
    }
}

#[test]
fn list_cluster_reorders_only_the_observed_nested_word_family() {
    // Literal expected index orders are the Word list-family contract:
    // source I order and source D order remain individually unchanged.
    type ListClusterCase<'a> = (&'a [&'a str], &'a [&'a str], &'a [usize], &'a [usize]);
    let cases: &[ListClusterCase<'_>] = &[
        (
            &["Heading", "a", "b"],
            &["0", "1", "0"],
            &[0, 3, 4, 1, 2, 5],
            &[0, 1, 2, 3, 4, 5],
        ),
        (
            &["Heading", "a", "b"],
            &["0", "2", "0"],
            &[0, 3, 4, 1, 2, 5],
            &[0, 1, 2, 3, 4, 5],
        ),
        (
            &["Heading", "a", "b"],
            &["0", "0", "0"],
            &[0, 1, 2, 3, 4, 5],
            &[0, 1, 2, 3, 4, 5],
        ),
        (
            &["Heading", "a", "b"],
            &["1", "1", "1"],
            &[0, 1, 2, 3, 4, 5],
            &[0, 1, 2, 3, 4, 5],
        ),
        (
            &["Heading", "Four separate revised heading words", "b"],
            &["0", "1", "0"],
            &[0, 1, 2, 3, 4, 5],
            &[0, 1, 2, 3, 4, 5],
        ),
        (
            &["test", "test", "test"],
            &["0", "1", "0"],
            &[0, 1, 2, 3, 4, 5],
            &[0, 1, 2, 3, 4, 5],
        ),
    ];
    for &(inserted, levels, order, original) in cases {
        for blank in [false, true] {
            let mut blocks = inserted
                .iter()
                .map(|text| marked("ins", text, "<w:pStyle w:val='ListParagraph'/>"))
                .collect::<Vec<_>>();
            for (index, level) in levels.iter().enumerate() {
                blocks.push(marked(
                    "del",
                    &format!("Original list item {index}"),
                    &format!("<w:numPr><w:ilvl w:val='{level}'/><w:numId w:val='7'/></w:numPr>"),
                ));
            }
            if blank {
                blocks.push("<w:p><w:pPr><w:spacing w:after='80'/></w:pPr></w:p>".into());
            }
            let input = blocks.concat() + "<w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr>";
            let (mut dom, root) = package(&input);
            let old_lifetimes = dom
                .descendants(root, Some(&W::del()))
                .into_iter()
                .map(|node| semantic(&dom, node))
                .collect::<Vec<_>>();
            let new_lifetimes = dom
                .descendants(root, Some(&W::ins()))
                .into_iter()
                .map(|node| semantic(&dom, node))
                .collect::<Vec<_>>();
            interleave_list_cluster_after_coalesce(&mut dom, root);
            let indices = if order == original { original } else { order };
            let mut oracle = indices
                .iter()
                .map(|&index| blocks[index].as_str())
                .collect::<String>();
            if blank {
                oracle.push_str(blocks.last().unwrap());
            }
            oracle.push_str("<w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr>");
            expected(
                &dom,
                root,
                &oracle,
                "nested Word interleave versus unrelated source family",
            );
            assert_eq!(
                dom.descendants(root, Some(&W::del()))
                    .into_iter()
                    .map(|node| semantic(&dom, node))
                    .collect::<Vec<_>>(),
                old_lifetimes
            );
            assert_eq!(
                dom.descendants(root, Some(&W::ins()))
                    .into_iter()
                    .map(|node| semantic(&dom, node))
                    .collect::<Vec<_>>(),
                new_lifetimes
            );
            stable(&mut dom, root, interleave_list_cluster_after_coalesce);
        }
    }
}

#[test]
fn a_stranded_mark_can_be_copied_without_deleting_its_layout_shell() {
    for props in [
        "",
        "<w:pStyle w:val='BodyText'/>",
        "<w:spacing w:after='80'/>",
    ] {
        for shell_extra in ["", "<w:b/>"] {
            for shell_content in ["", "<w:r><w:t>Independent live text</w:t></w:r>"] {
                let mark = format!(
                    "<w:rPr>{shell_extra}<w:del w:id='21' w:author='Original mark owner' w:date='{DATE}'/></w:rPr>"
                );
                let content = para("del", "Original owned paragraph", props);
                let shell = format!("<w:p><w:pPr>{mark}</w:pPr>{shell_content}</w:p>");
                let input = content.clone() + &shell + TABLE;
                let (mut dom, root) = package(&input);
                restamp_stranded_del_mark_onto_del_only_paragraph(&mut dom, root);
                let oracle =
                    if props.is_empty() && shell_extra.is_empty() && shell_content.is_empty() {
                        para("del", "Original owned paragraph", &mark) + &shell + TABLE
                    } else {
                        input
                    };
                expected(
                    &dom,
                    root,
                    &oracle,
                    "stranded pilcrow clone preserves content and source shell",
                );
                stable(
                    &mut dom,
                    root,
                    restamp_stranded_del_mark_onto_del_only_paragraph,
                );
            }
        }
    }
}

#[test]
fn a_terminal_property_history_replaces_the_mark_but_keeps_its_formats() {
    for status in ["del", "mix", "ins"] {
        for history in [false, true] {
            for format in ["", "<w:b/><w:color w:val='123456'/>"] {
                let saved = if history {
                    format!(
                        "<w:pPrChange w:id='11' w:author='Original layout owner' w:date='{DATE}'><w:pPr><w:spacing w:after='400'/></w:pPr></w:pPrChange>"
                    )
                } else {
                    String::new()
                };
                let mark = format!(
                    "<w:rPr>{format}<w:del w:id='18' w:author='Original mark owner'/></w:rPr>"
                );
                let texts = if status == "mix" {
                    run("ins", "Revised owned text") + &run("del", "Original owned text")
                } else {
                    run(status, "Owned text")
                };
                let source =
                    format!("{TABLE}<w:p><w:pPr><w:keepNext/>{mark}{saved}</w:pPr>{texts}</w:p>");
                let (mut dom, root) = package(&source);
                strip_last_pure_del_mark_when_pprchange(&mut dom, root);
                // A pure-I payload with an explicit D mark is MIX and
                // owns the same terminal property-history rule.
                let eligible = history;
                let kept_mark = if eligible {
                    if format.is_empty() {
                        String::new()
                    } else {
                        format!("<w:rPr>{format}</w:rPr>")
                    }
                } else {
                    mark
                };
                expected(
                    &dom,
                    root,
                    &format!(
                        "{TABLE}<w:p><w:pPr><w:keepNext/>{kept_mark}{saved}</w:pPr>{texts}</w:p>"
                    ),
                    "terminal history retains complete current/old formatting",
                );
                stable(&mut dom, root, strip_last_pure_del_mark_when_pprchange);
            }
        }
    }
}

#[test]
fn title_page_empty_relocation_is_scoped_to_an_authored_cover_tail() {
    let blank = format!(
        "<w:p><w:pPr><w:spacing w:after='80'/><w:rPr><w:ins w:id='18' w:author='Original blank owner' w:date='{DATE}'/></w:rPr></w:pPr></w:p>"
    );
    let bare = "<w:p><w:pPr><w:spacing w:after='80'/></w:pPr></w:p>";
    for (cover, authored_cover) in [
        ("Prepared for client", true),
        ("client@example.test", true),
        ("January release", true),
        ("November release", true),
        ("Version 2040", true),
        ("Revision 2026", true),
        ("Serial 1234", true),
        ("Ordinary heading", false),
    ] {
        for count in [5usize, 6] {
            for later_owner in [false, true] {
                let anchor = marked("ins", cover, "<w:jc w:val='center'/>");
                let deleted = marked("del", "Original cover text", "");
                let later = if later_owner {
                    para("live", "Independent trailing paragraph", "<w:keepNext/>")
                } else {
                    String::new()
                };
                let input = anchor.clone() + &blank.repeat(count) + &deleted + &later + TABLE;
                let (mut dom, root) = package(&input);
                relocate_title_page_last_empty_after_pure_dels(&mut dom, root);
                let oracle = if authored_cover && count == 6 && !later_owner {
                    anchor + &blank.repeat(5) + &deleted + bare + TABLE
                } else {
                    input
                };
                expected(
                    &dom,
                    root,
                    &oracle,
                    "cover-only blank relocation retains independently owned trailing layout",
                );
                stable(
                    &mut dom,
                    root,
                    relocate_title_page_last_empty_after_pure_dels,
                );
            }
        }
    }
}

#[test]
fn a_short_title_mesh_cannot_consume_foreign_revision_provenance_or_unrelated_titles() {
    for (inserted, deleted) in [
        ("Distinct revised violet harbor", "Original quartz meadow"),
        ("This document and document repeats", "Original document"),
        (
            "Long revised title has seven separate words",
            "Original document",
        ),
    ] {
        for foreign in [false, true] {
            let old = run("del", deleted);
            let new = run("ins", inserted);
            let old = if foreign {
                old.replace("Source editor", "Earlier editor")
            } else {
                old
            };
            // These are distinct nonmatching, repeated-token, and
            // long-title guards; the observed M377 positive shape stays
            // covered by the existing fixture tests.
            let input = format!("<w:p><w:pPr><w:keepNext/></w:pPr>{new}{old}</w:p>{TABLE}");
            let (mut dom, root) = package(&input);
            let before = semantic(&dom, root);
            free_mesh_shared_title_token_in_mix(&mut dom, root);
            assert_eq!(
                semantic(&dom, root),
                before,
                "{inserted}/{deleted}, foreign={foreign}"
            );
        }
    }
}

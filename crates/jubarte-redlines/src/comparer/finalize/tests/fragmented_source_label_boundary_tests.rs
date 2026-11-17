// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;
const DATE: &str = "2026-01-02T03:04:05Z";
const SECTION: &str = "<w:sectPr><w:pgSz w:w='12240' w:h='15840'/><w:pgMar w:top='720' w:right='900' w:bottom='720' w:left='900'/></w:sectPr>";

fn package(source: &str) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&format!(
        "<w:document xmlns:w='{}'><w:body>{source}</w:body></w:document>",
        W::URI
    ));
    let root = dom.root(doc).expect("document");
    (dom, root)
}
fn semantic(dom: &Dom, node: NodeId) -> String {
    if !dom.is_element(node) {
        // An empty text node created by set_value and no text child in a
        // parsed empty element carry the same XML payload. Keep every
        // empty element, property, attribute, and nonempty character.
        if dom.text_value(node) == Some("") {
            return String::new();
        }
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

const PROPS: &str = "<w:pPr><w:pStyle w:val='ListParagraph'/><w:numPr><w:ilvl w:val='1'/><w:numId w:val='7'/></w:numPr><w:spacing w:after='80'/><w:rPr><w:del w:id='18' w:author='Original pilcrow owner' w:date='2025-02-03T04:05:06Z'/><w:i/></w:rPr><w:pPrChange w:id='21' w:author='Original layout owner' w:date='2025-03-04T05:06:07Z'><w:pPr><w:spacing w:after='60'/></w:pPr></w:pPrChange></w:pPr>";
const FORMAT: &str = "<w:rPr><w:b/><w:color w:val='123456'/></w:rPr>";
const OTHER_FORMAT: &str = "<w:rPr><w:i/><w:color w:val='654321'/></w:rPr>";
const TABLE: &str = "<w:tbl><w:tblPr><w:tblW w:w='1800' w:type='dxa'/></w:tblPr><w:tblGrid><w:gridCol w:w='1800'/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w='1800' w:type='dxa'/></w:tcPr><w:p><w:r><w:t>Independent table payload</w:t></w:r></w:p></w:tc></w:tr></w:tbl>";
fn run(kind: &str, text: &str, format: &str) -> String {
    let tag = if kind == "del" { "delText" } else { "t" };
    format!("<w:r>{format}<w:{tag} xml:space='preserve'>{text}</w:{tag}></w:r>")
}
fn wrapper(kind: &str, runs: &str) -> String {
    format!(
        "<w:{kind} w:id='{}' w:author='Body revision owner' w:date='{DATE}'>{runs}</w:{kind}>",
        if kind == "del" { 12 } else { 11 }
    )
}
fn assert_complete(dom: &Dom, root: NodeId, expected: &str, label: &str) {
    let (want, wr) = package(expected);
    let actual = semantic(dom, root);
    let oracle = semantic(&want, wr);
    if actual != oracle {
        let first = actual
            .chars()
            .zip(oracle.chars())
            .position(|(a, b)| a != b)
            .unwrap_or_else(|| actual.chars().count().min(oracle.chars().count()));
        let context = |v: &str| {
            v.chars()
                .skip(first.saturating_sub(40))
                .take(180)
                .collect::<String>()
        };
        panic!(
            "{label}: complete source differs at {first}: actual {:?}, expected {:?}",
            context(&actual),
            context(&oracle)
        );
    }
}
#[test]
fn fragmented_labels_keep_literal_source_chars_formats_and_revision_history() {
    // The expected deleted runs are literal retained source leaves, not
    // derived by the implementation's stripping algorithm. A zero-length
    // delText remains a valid source leaf; empty inserted run shells go.
    type FragmentCase<'a> = (
        &'a str,
        &'a [&'a str],
        &'a [&'a str],
        &'a str,
        &'a [&'a str],
        &'a str,
    );
    let cases: &[FragmentCase<'_>] = &[
        (
            "literal period separator",
            &["Lvl 1.ab"],
            &["ab"],
            "ab",
            &["Lvl 1."],
            "",
        ),
        (
            "Unicode trailing space after punctuation",
            &["Lvl 1 é", ""],
            &["é. "],
            "é",
            &["Lvl 1 ", ""],
            ". ",
        ),
        (
            "Unicode punctuation without spaces",
            &["Lvl 1 é", ""],
            &["é."],
            "é",
            &["Lvl 1 ", ""],
            ".",
        ),
        (
            "authored leading prefix across fragments",
            &[" Lvl 1 a", "b"],
            &["ab"],
            "ab",
            &[" Lvl 1 ", ""],
            "",
        ),
        (
            "authored leading prefix on one leaf",
            &[" Lvl 1 ab"],
            &["ab"],
            "ab",
            &[" Lvl 1 "],
            "",
        ),
        (
            "ASCII split label",
            &["Lvl 1 a", "b"],
            &["ab"],
            "ab",
            &["Lvl 1 ", ""],
            "",
        ),
        (
            "ASCII split label with authored suffix",
            &["Lvl 1 a", "b"],
            &["a", "b "],
            "ab",
            &["Lvl 1 ", ""],
            " ",
        ),
        (
            "empty source leaves",
            &["", "Lvl 1 a", "b", ""],
            &["", "a", "b", ""],
            "ab",
            &["", "Lvl 1 ", "", ""],
            "",
        ),
        (
            "Unicode literal space",
            &["Lvl 1 é", ""],
            &["é "],
            "é",
            &["Lvl 1 ", ""],
            " ",
        ),
        (
            "Unicode literal punctuation",
            &["", "Lvl 1 é", ""],
            &["", "é .", ""],
            "é",
            &["", "Lvl 1 ", ""],
            " .",
        ),
    ];
    for &(case, original, revised, label, remaining_original, remaining_revised) in cases {
        for inserted_first in [false, true] {
            for tracking in [false, true] {
                let old = wrapper(
                    "del",
                    &original
                        .iter()
                        .map(|text| run("del", text, FORMAT))
                        .collect::<String>(),
                );
                let new = wrapper(
                    "ins",
                    &revised
                        .iter()
                        .map(|text| run("ins", text, FORMAT))
                        .collect::<String>(),
                );
                let body = if inserted_first {
                    new + &old
                } else {
                    old + &new
                };
                let input = format!("<w:p>{PROPS}{body}</w:p>{TABLE}{SECTION}");
                let (mut dom, root) = package(&input);
                let p = dom.elements(dom.element(root, &W::body()).unwrap(), Some(&W::p()))[0];
                let settings = WmlComparerSettings {
                    detect_format_changes: tracking,
                    author_for_revisions: "Boundary editor".into(),
                    date_time_for_revisions: DATE.into(),
                    ..WmlComparerSettings::default()
                };
                mesh_short_label_shared_eq_with_settings(&mut dom, p, &settings);
                let deleted = wrapper(
                    "del",
                    &remaining_original
                        .iter()
                        .map(|text| run("del", text, FORMAT))
                        .collect::<String>(),
                );
                let equal = format!("<w:r>{FORMAT}<w:t>{label}</w:t></w:r>");
                let inserted = if remaining_revised.is_empty() {
                    String::new()
                } else {
                    wrapper("ins", &run("ins", remaining_revised, FORMAT))
                };
                let expected =
                    format!("<w:p>{PROPS}{deleted}{equal}{inserted}</w:p>{TABLE}{SECTION}");
                let coordinate = format!("{case}/ins-first={inserted_first}/tracking={tracking}");
                assert_complete(&dom, root, &expected, &coordinate);
                let once = semantic(&dom, root);
                let nodes = dom.node_count();
                mesh_short_label_shared_eq_with_settings(&mut dom, p, &settings);
                assert_eq!(semantic(&dom, root), once, "{coordinate}: repeat");
                assert_eq!(dom.node_count(), nodes);
            }
        }
    }
}
#[test]
fn label_format_boundaries_and_disabled_history_leave_complete_original_sources_intact() {
    for (property_boundary, revised_boundary) in [(false, false), (true, false), (true, true)] {
        for inserted_first in [false, true] {
            let old_runs = if property_boundary && !revised_boundary {
                run("del", "Lvl 1 a", FORMAT) + &run("del", "b", OTHER_FORMAT)
            } else {
                run("del", "Lvl 1 ab", FORMAT)
            };
            let new_runs = if property_boundary && revised_boundary {
                run("ins", "a", FORMAT) + &run("ins", "b", OTHER_FORMAT)
            } else {
                run("ins", "ab", OTHER_FORMAT)
            };
            let old = wrapper("del", &old_runs);
            let new = wrapper("ins", &new_runs);
            let body = if inserted_first {
                new + &old
            } else {
                old + &new
            };
            let input = format!("<w:p>{PROPS}{body}</w:p>{TABLE}{SECTION}");
            let (mut dom, root) = package(&input);
            let p = dom.elements(dom.element(root, &W::body()).unwrap(), Some(&W::p()))[0];
            // A label spanning two differently formatted source runs
            // cannot become one EQ. If both labels have a consistent
            // but different format, disabled tracking cannot own both.
            let settings = WmlComparerSettings {
                detect_format_changes: property_boundary,
                author_for_revisions: "Boundary editor".into(),
                date_time_for_revisions: DATE.into(),
                ..WmlComparerSettings::default()
            };
            let original = semantic(&dom, root);
            let nodes = dom.node_count();
            mesh_short_label_shared_eq_with_settings(&mut dom, p, &settings);
            assert_complete(&dom, root, &input, "source format boundary refusal");
            assert_eq!(dom.node_count(), nodes);
            mesh_short_label_shared_eq_with_settings(&mut dom, p, &settings);
            assert_eq!(semantic(&dom, root), original);
            assert_eq!(dom.node_count(), nodes);
        }
    }
}
#[test]
fn midstream_heading_search_preserves_controls_live_anchors_and_identical_property_owners() {
    let layout = "<w:pStyle w:val='Heading1'/><w:spacing w:after='120'/>";
    let heading_body = wrapper("ins", &run("ins", "1. Revised heading", FORMAT));
    let title_body = wrapper("del", &run("del", "Double Spacing Bold Demo", FORMAT));
    let heading = format!(
        "<w:p><w:pPr>{layout}<w:rPr><w:ins w:id='18' w:author='Revised pilcrow owner' w:date='{DATE}'/><w:b/></w:rPr></w:pPr>{heading_body}</w:p>"
    );
    let title = format!(
        "<w:p><w:pPr>{layout}<w:rPr><w:del w:id='19' w:author='Original pilcrow owner' w:date='{DATE}'/><w:b/></w:rPr></w:pPr>{title_body}</w:p>"
    );
    let intro = (0..5)
        .map(|index| {
            format!(
                "<w:p>{}</w:p>",
                wrapper(
                    "ins",
                    &run("ins", &format!("Revised introduction {index}"), FORMAT)
                )
            )
        })
        .collect::<String>();
    let control = "<w:sdt><w:sdtPr><w:alias w:val='Independent source control'/><w:tag w:val='SourceControl'/><w:id w:val='81'/><w:richText/></w:sdtPr><w:sdtContent><w:p><w:pPr><w:spacing w:after='80'/></w:pPr><w:r><w:rPr><w:i/></w:rPr><w:t>Independent control payload</w:t></w:r></w:p></w:sdtContent></w:sdt>";
    let live = "<w:p><w:pPr><w:spacing w:after='80'/></w:pPr><w:r><w:rPr><w:i/></w:rPr><w:t>Independent live anchor</w:t></w:r></w:p>";
    for tracking in [false, true] {
        let settings = WmlComparerSettings {
            detect_format_changes: tracking,
            author_for_revisions: "Boundary editor".into(),
            date_time_for_revisions: DATE.into(),
            ..WmlComparerSettings::default()
        };
        for barrier in ["none", "control", "live", "control-only-tail"] {
            let (input, expected) = if barrier == "none" {
                let input = format!("{heading}{intro}{title}{TABLE}{SECTION}");
                let expected = format!(
                    "<w:p><w:pPr>{layout}<w:rPr><w:b/></w:rPr></w:pPr>{heading_body}{title_body}</w:p>{intro}{TABLE}{SECTION}"
                );
                (input, expected)
            } else if barrier == "control-only-tail" {
                let extra = format!(
                    "<w:p>{}</w:p>",
                    wrapper("ins", &run("ins", "Revised final introduction", FORMAT))
                );
                let input = format!("{heading}{intro}{extra}{title}{control}{SECTION}");
                (input.clone(), input)
            } else {
                // A real authored section break on the old title makes
                // this a conservative source-ownership refusal after
                // nearest-heading search has crossed the control/live
                // node. It may not erase or transfer that section owner.
                let section_title = title.replace(
                    "</w:pPr>",
                    "<w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr></w:pPr>",
                );
                let intervening = if barrier == "control" { control } else { live };
                let input = format!("{heading}{intro}{intervening}{section_title}{TABLE}{SECTION}");
                (input.clone(), input)
            };
            let (mut dom, root) = package(&input);
            let mut id = 100;
            fold_midstream_demo_title_into_numbered_heading_with_settings(
                &mut dom, root, &settings, &mut id,
            );
            assert_complete(
                &dom,
                root,
                &expected,
                &format!("numbered heading/{barrier}/tracking={tracking}"),
            );
            assert_eq!(
                id, 100,
                "identical full source properties or conservative refusal need no synthesized history"
            );
            let once = semantic(&dom, root);
            let next = id;
            fold_midstream_demo_title_into_numbered_heading_with_settings(
                &mut dom, root, &settings, &mut id,
            );
            assert_eq!(semantic(&dom, root), once);
            assert_eq!(id, next);
        }
    }
}

/// Word M374 keeps the deleted list owner, donates only missing scalar
/// layout properties, and leaves an already adjacent inserted spacer in
/// source order. A story may omit optional final section properties.
#[test]
fn public_label_zip_preserves_already_adjacent_spacer_and_missing_layout_ownership() {
    let old_spacing = "<w:spacing w:after='80'/>";
    let new_spacing = "<w:spacing w:line='276'/>";
    let old_indent = "<w:ind w:left='720'/>";
    let new_indent = "<w:ind w:left='900' w:hanging='300'/>";
    let list = "<w:pStyle w:val='ListParagraph'/><w:numPr><w:ilvl w:val='1'/><w:numId w:val='7'/></w:numPr>";
    let mark = |kind: &str, id: u32, author: &str| {
        format!("<w:rPr><w:{kind} w:id='{id}' w:author='{author}' w:date='{DATE}'/><w:b/></w:rPr>")
    };
    for old_has_spacing in [false, true] {
        for new_has_spacing in [false, true] {
            for old_has_indent in [false, true] {
                for new_has_indent in [false, true] {
                    for inserted_first in [false, true] {
                        for has_tail in [false, true] {
                            for whitespace_spacer in [false, true] {
                                let a_spacing = if old_has_spacing { old_spacing } else { "" };
                                let b_spacing = if new_has_spacing { new_spacing } else { "" };
                                let a_indent = if old_has_indent { old_indent } else { "" };
                                let b_indent = if new_has_indent { new_indent } else { "" };
                                let old_mark = mark("del", 31, "Original list mark owner");
                                let new_mark = mark("ins", 32, "Revised label mark owner");
                                let spacer_mark = mark("ins", 33, "Independent spacer owner");
                                let old = format!(
                                    "<w:p><w:pPr>{list}{a_spacing}{a_indent}{old_mark}</w:pPr>{}</w:p>",
                                    wrapper("del", &run("del", "Lvl 1 a", FORMAT))
                                );
                                let new = format!(
                                    "<w:p><w:pPr>{b_spacing}{b_indent}{new_mark}</w:pPr>{}</w:p>",
                                    wrapper("ins", &run("ins", "a ", FORMAT))
                                );
                                let spacer_body = if whitespace_spacer {
                                    wrapper("ins", &run("ins", " ", OTHER_FORMAT))
                                } else {
                                    String::new()
                                };
                                let spacer = format!(
                                    "<w:p><w:pPr><w:spacing w:after='40'/>{spacer_mark}</w:pPr>{spacer_body}</w:p>"
                                );
                                let tail = if has_tail {
                                    format!("{TABLE}{SECTION}")
                                } else {
                                    String::new()
                                };
                                let input = if inserted_first {
                                    format!("{new}{spacer}{old}{tail}")
                                } else {
                                    format!("{old}{new}{spacer}{tail}")
                                };
                                let donated_spacing = if !old_has_spacing { b_spacing } else { "" };
                                let donated_indent = if !old_has_indent { b_indent } else { "" };
                                let deleted = wrapper("del", &run("del", "Lvl 1 ", FORMAT));
                                let equal = format!("<w:r>{FORMAT}<w:t>a</w:t></w:r>");
                                let inserted = wrapper("ins", &run("ins", " ", FORMAT));
                                let expected = format!(
                                    "<w:p><w:pPr>{list}{a_spacing}{a_indent}{old_mark}{donated_spacing}{donated_indent}</w:pPr>{deleted}{equal}{inserted}</w:p>{spacer}{tail}"
                                );
                                let (mut dom, root) = package(&input);
                                let settings = WmlComparerSettings {
                                    author_for_revisions: "Configured list editor".into(),
                                    date_time_for_revisions: DATE.into(),
                                    ..WmlComparerSettings::default()
                                };
                                let coordinate = format!(
                                    "old-spacing={old_has_spacing}/new-spacing={new_has_spacing}/old-indent={old_has_indent}/new-indent={new_has_indent}/ins-first={inserted_first}/tail={has_tail}/space={whitespace_spacer}"
                                );
                                residual_short_label_zip_with_settings(&mut dom, root, &settings);
                                assert_complete(&dom, root, &expected, &coordinate);
                                let once = semantic(&dom, root);
                                let nodes = dom.node_count();
                                residual_short_label_zip_with_settings(&mut dom, root, &settings);
                                assert_eq!(semantic(&dom, root), once, "{coordinate}: repeat");
                                assert_eq!(dom.node_count(), nodes, "{coordinate}: repeat arena");
                            }
                        }
                    }
                }
            }
        }
    }
}

/// A common significant word may exist on both sides while the longest
/// ordered match contains only glue words. Such a mesh has no substantive
/// shared anchor and must leave both complete source streams untouched.
#[test]
fn wholesale_glue_only_lcs_preserves_crossed_significant_source_anchors() {
    for glue in ["is of to", "we on at", "as by or"] {
        for inserted_first in [false, true] {
            for capitalized in [false, true] {
                let anchor = if capitalized {
                    "Sharedanchor"
                } else {
                    "sharedanchor"
                };
                let old_text = format!("{anchor} {glue} original");
                let new_text = format!("revised {glue} {anchor}");
                let deleted = wrapper("del", &run("del", &old_text, FORMAT));
                let inserted = wrapper("ins", &run("ins", &new_text, OTHER_FORMAT));
                let sources = if inserted_first {
                    inserted + &deleted
                } else {
                    deleted + &inserted
                };
                let input = format!("<w:p>{PROPS}{sources}</w:p>{TABLE}{SECTION}");
                let (mut dom, root) = package(&input);
                let original = semantic(&dom, root);
                let nodes = dom.node_count();
                free_mesh_wholesale_body_mix(&mut dom, root);
                let coordinate =
                    format!("glue={glue}/ins-first={inserted_first}/capitalized={capitalized}");
                assert_complete(&dom, root, &input, &coordinate);
                assert_eq!(semantic(&dom, root), original);
                assert_eq!(dom.node_count(), nodes);
                free_mesh_wholesale_body_mix(&mut dom, root);
                assert_eq!(semantic(&dom, root), original, "{coordinate}: repeat");
                assert_eq!(dom.node_count(), nodes);
            }
        }
    }
}

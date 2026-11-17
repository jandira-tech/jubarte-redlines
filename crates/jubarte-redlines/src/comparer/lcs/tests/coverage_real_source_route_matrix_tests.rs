// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;
use crate::namespaces::{M, R};

fn paragraph(text: &str, style: &str, numbered: Option<u32>, alternate: bool) -> String {
    let num = numbered.map_or_else(String::new, |level| {
        format!("<w:numPr><w:ilvl w:val=\"{level}\"/><w:numId w:val=\"9\"/></w:numPr>")
    });
    format!(
        "<w:p><w:pPr><w:pStyle w:val=\"{style}\"/>{num}<w:spacing w:before=\"120\" w:after=\"80\"/><w:ind w:left=\"240\"/></w:pPr><w:r><w:rPr>{}<w:color w:val=\"{}\"/><w:lang w:val=\"en-US\"/></w:rPr><w:t xml:space=\"preserve\">{text}</w:t></w:r></w:p>",
        if alternate { "<w:b/>" } else { "<w:i/>" },
        if alternate { "123456" } else { "654321" }
    )
}

fn table(side: &str, index: usize, digits: bool) -> String {
    let mut cells = String::new();
    for column in 0..2 {
        let text = if digits {
            format!("{}", index * 2 + column + 1)
        } else {
            format!("{side} cell{} item{}", column + 1, index + 1)
        };
        cells.push_str(&format!(
            "<w:tc><w:tcPr><w:tcW w:w=\"{}\" w:type=\"dxa\"/></w:tcPr>{}</w:tc>",
            1800 + column * 600,
            paragraph(&text, "BodyText", None, column == 0)
        ));
    }
    format!(
        "<w:tbl><w:tblPr><w:tblW w:w=\"4200\" w:type=\"dxa\"/></w:tblPr><w:tblGrid><w:gridCol w:w=\"1800\"/><w:gridCol w:w=\"2400\"/></w:tblGrid><w:tr>{cells}</w:tr></w:tbl>"
    )
}

fn body(family: &str, side: &str, count: usize, shared: usize, blank: bool) -> String {
    let vocab = if side == "original" {
        [
            "quartz", "bronze", "saffron", "walnut", "orchard", "harvest", "copper", "meadow",
        ]
    } else {
        [
            "violet", "cobalt", "glacier", "silver", "harbor", "mariner", "sapphire", "summit",
        ]
    };
    let mut out = String::new();
    if blank {
        out.push_str(&paragraph("", "Normal", None, false));
    }
    for i in 0..count {
        let text = match family {
            "stamped" if i == 0 => {
                format!("file_{}.docx", if side == "original" { 137 } else { 138 })
            }
            "demo" if i == 0 => format!("{} Alignment Demo", vocab[0]),
            "demo" => format!(
                "This document demonstrates {} {} controls",
                vocab[i % 8],
                vocab[(i + 1) % 8]
            ),
            "labels" => ["ONE", "a", "b", "TWO", "c", "d", "THREE", "e"][i % 8].to_string(),
            "nested" | "numbered" => format!("{} {}", vocab[i % 8], i + 1),
            "html" if i == 1 => "html input type text".to_string(),
            "annotation" if i == 0 => {
                format!("Oftentimes {} authors suggest amendments", vocab[0])
            }
            "cover" => match i % 5 {
                0 => "Service agreement".to_string(),
                1 => "Prepared by Alice".to_string(),
                2 => "January 2026".to_string(),
                3 => "Client Name".to_string(),
                _ => "March 2040".to_string(),
            },
            "wrap" if i == count - 1 => (0..5)
                .map(|_| format!("{} tightly wraps every {} line ", vocab[0], vocab[1]))
                .collect(),
            "math" if i == 0 => "Math borderbox formulas".to_string(),
            "sectioned" => format!(
                "{}) {} section {}",
                (b'A' + (i % 5) as u8) as char,
                vocab[i % 8],
                i + 1
            ),
            "statistics" if i < 3 => {
                format!("Num {} {}", ["words", "chars", "pages"][i], 20 + i)
            }
            "table" if i == 0 => format!("{} table report", vocab[0]),
            _ => format!(
                "{} {} {} record{}",
                vocab[i % 8],
                vocab[(i + 1) % 8],
                vocab[(i + 2) % 8],
                i
            ),
        };
        let text = if shared == 0 {
            text
        } else {
            format!(
                "{} {text}",
                [
                    "shared", "retained", "common", "contract", "report", "review", "document",
                    "context"
                ][..shared]
                    .join(" ")
            )
        };
        let numbered = if family == "nested" {
            Some(if i % 4 == 1 || i % 4 == 2 { 1 } else { 0 })
        } else if family == "numbered" {
            Some(0)
        } else {
            None
        };
        let style = if family == "cover" && i == 0 {
            "Title"
        } else {
            "BodyText"
        };
        out.push_str(&paragraph(&text, style, numbered, side == "original"));
        if family == "math" && i == 1 {
            out.push_str("<w:p><m:oMath><m:r><m:t>x+y</m:t></m:r></m:oMath></w:p>");
        }
        if ["table", "digits", "tableprose", "multitable"].contains(&family)
            && (i == 0 || family == "multitable" && i % 4 == 1)
        {
            out.push_str(&table(side, i, family == "digits"));
        }
        if blank && i == count / 2 {
            out.push_str(&paragraph("", "Normal", None, false));
        }
    }
    out
}

fn source(dom: &mut Dom, fragment: &str, settings: &WmlComparerSettings) -> Vec<ComparisonUnit> {
    let doc = dom.parse_xdocument(&format!("<w:document xmlns:w=\"{}\" xmlns:m=\"{}\" xmlns:r=\"{}\"><w:body>{fragment}<w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr></w:body></w:document>", W::URI, M::URI, R::URI));
    let root = dom.root(doc).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    super::super::preprocess::add_sha1_hash_to_block_level_content(
        dom,
        body,
        settings,
        &super::super::preprocess::null_rel_resolver,
    );
    let atoms = super::super::atomize::create_comparison_unit_atom_list(dom, body, settings);
    super::super::units::get_comparison_unit_list(dom, &atoms, settings)
}

fn clean_xml(dom: &mut Dom, node: NodeId) -> String {
    let copy = dom.clone_subtree(node);
    super::super::finalize::remove_powertools_scratch_markup(dom, copy);
    dom.serialize_element(copy)
}

#[derive(Debug, PartialEq, Eq)]
struct OwnedAtom {
    node: NodeId,
    ancestry: Vec<NodeId>,
    text_or_payload: String,
    paragraph_properties: Option<String>,
    run_properties: Option<String>,
    geometry: Vec<String>,
}

fn frozen(dom: &mut Dom, units: &[ComparisonUnit]) -> Vec<OwnedAtom> {
    let mut xml_cache = std::collections::HashMap::<NodeId, String>::new();
    let mut clean = |dom: &mut Dom, node: NodeId| {
        xml_cache
            .entry(node)
            .or_insert_with(|| clean_xml(dom, node))
            .clone()
    };
    units
        .iter()
        .flat_map(ComparisonUnit::descendant_atoms)
        .map(|a| {
            let ancestors = a.ancestor_elements.to_vec();
            let para = ancestors
                .iter()
                .rev()
                .find(|&&n| dom.name_is(n, &W::p()))
                .copied();
            let run = ancestors
                .iter()
                .rev()
                .find(|&&n| dom.name_is(n, &W::r()))
                .copied();
            let paragraph_properties = para
                .and_then(|p| dom.element(p, &W::p_pr()))
                .map(|n| clean(dom, n));
            let run_properties = run
                .and_then(|r| dom.element(r, &W::r_pr()))
                .map(|n| clean(dom, n));
            let mut geometry = Vec::new();
            for ancestor in ancestors {
                for (parent, child) in [
                    (W::tbl(), W::name("tblGrid")),
                    (W::tbl(), W::tbl_pr()),
                    (W::tc(), W::tc_pr()),
                    (W::sdt(), W::sdt_pr()),
                    (W::sdt(), W::name("sdtEndPr")),
                ] {
                    if dom.name_is(ancestor, &parent)
                        && let Some(n) = dom.element(ancestor, &child)
                    {
                        geometry.push(clean(dom, n));
                    }
                }
            }
            OwnedAtom {
                node: a.content_element,
                ancestry: a.ancestor_elements.to_vec(),
                text_or_payload: clean(dom, a.content_element),
                paragraph_properties,
                run_properties,
                geometry,
            }
        })
        .collect()
}

fn assert_owned(
    dom: &mut Dom,
    seqs: &[CorrelatedSequence],
    expected: &(Vec<OwnedAtom>, Vec<OwnedAtom>),
    label: &str,
) {
    for revised in [false, true] {
        let units = seqs
            .iter()
            .flat_map(|s| {
                if revised {
                    s.com_units_2.as_deref().unwrap_or_default()
                } else {
                    s.com_units_1.as_deref().unwrap_or_default()
                }
            })
            .cloned()
            .collect::<Vec<_>>();
        let actual = frozen(dom, &units);
        let authored = if revised { &expected.1 } else { &expected.0 };
        // The documented interior Word carrier consumes exactly B's
        // final pilcrow while deleting A's first carrier pilcrow. Its
        // other atoms retain their authored pPr/rPr/geometry snapshots.
        // This exception never applies to text, math, drawings or an
        // interior revised mark, nor to a missing original atom.
        let absorbed_final_mark = revised
            && !authored.is_empty()
            && actual.len() + 1 == authored.len()
            && actual == authored[..authored.len() - 1]
            && dom.name_is(authored.last().unwrap().node, &W::p_pr());
        if absorbed_final_mark {
            let carrier_index = expected
                .0
                .iter()
                .position(|atom| dom.name_is(atom.node, &W::p_pr()))
                .unwrap();
            assert!(
                carrier_index + 1 < expected.0.len(),
                "{label}: carrier must be interior"
            );
            let carrier = expected.0[carrier_index].node;
            assert!(
                seqs.iter()
                    .any(|seq| seq.correlation_status == CorrelationStatus::Deleted
                        && seq
                            .com_units_1
                            .as_deref()
                            .unwrap_or_default()
                            .iter()
                            .flat_map(ComparisonUnit::descendant_atoms)
                            .any(|atom| atom.content_element == carrier)),
                "{label}: absorbed revised pilcrow requires its original carrier mark Deleted"
            );
        } else {
            let difference = actual
                .iter()
                .zip(authored)
                .position(|(a, b)| a != b)
                .or_else(|| {
                    (actual.len() != authored.len()).then_some(actual.len().min(authored.len()))
                });
            assert!(
                &actual == authored,
                "{label}: revised={revised}; text, pPr, rPr, geometry and ownership must all survive; lengths actual={} expected={}; first difference={difference:?}; actual={:?}; expected={:?}",
                actual.len(),
                authored.len(),
                difference.and_then(|index| actual.get(index)),
                difference.and_then(|index| authored.get(index))
            );
        }
    }
}

#[test]
fn positional_title_zip_keeps_interior_textless_payloads_and_properties_in_source_order() {
    for gap in [
        "<w:p><w:pPr><w:spacing w:after='240'/></w:pPr></w:p>",
        "<w:p><w:pPr><w:jc w:val='center'/></w:pPr><m:oMath><m:r><m:t>x+y</m:t></m:r></m:oMath></w:p>",
        "<w:p><w:pPr><w:ind w:left='360'/></w:pPr><w:r><w:tab/><w:br w:type='page'/></w:r></w:p>",
    ] {
        for reverse in [false, true] {
            let a = format!(
                "{}{}{}{}",
                paragraph("shared original first", "Title", None, true),
                gap,
                paragraph("shared original second", "BodyText", None, true),
                gap
            );
            let b = format!(
                "{gap}{}{}{}",
                paragraph("shared revised first", "Title", None, false),
                paragraph("shared revised second", "BodyText", None, false),
                paragraph("revised residual", "Normal", None, false)
            );
            let settings = WmlComparerSettings::default();
            let mut dom = Dom::new();
            let (left_xml, right_xml) = if reverse { (&b, &a) } else { (&a, &b) };
            let left = source(&mut dom, left_xml, &settings);
            let right = source(&mut dom, right_xml, &settings);
            let expected = (frozen(&mut dom, &left), frozen(&mut dom, &right));
            let out = positional_title_token_zip(&mut dom, &left, &right, &settings).unwrap();
            assert_owned(&mut dom, &out, &expected, "textless positional gap");
        }
    }
}

#[test]
fn opaque_math_groups_do_not_invent_textual_demo_titles_or_list_items() {
    for display in [false, true] {
        for title in [false, true] {
            for reverse in [false, true] {
                let equation = if display {
                    "<m:oMathPara><m:oMath><m:r><m:rPr><m:sty m:val='p'/></m:rPr><m:t>x+y</m:t></m:r></m:oMath></m:oMathPara>"
                } else {
                    "<m:oMath><m:r><m:rPr><m:sty m:val='p'/></m:rPr><m:t>x+y</m:t></m:r></m:oMath>"
                };
                let math_para = format!(
                    "<w:p><w:pPr><w:jc w:val='center'/><w:spacing w:after='180'/></w:pPr>{equation}</w:p>"
                );
                let opaque = format!(
                    "{}{}",
                    if title {
                        paragraph("Slate Demo", "Title", None, true)
                    } else {
                        math_para.clone()
                    },
                    math_para
                );
                let listed = format!(
                    "{}{}{}",
                    paragraph("Azure Demo", "Title", None, false),
                    paragraph("First violet item", "BodyText", Some(0), false),
                    paragraph("Second kapok item", "BodyText", Some(0), false)
                );
                let settings = WmlComparerSettings::default();
                let mut dom = Dom::new();
                let (a, b) = if reverse {
                    (&listed, &opaque)
                } else {
                    (&opaque, &listed)
                };
                let left = source(&mut dom, a, &settings);
                let right = source(&mut dom, b, &settings);
                let before = (frozen(&mut dom, &left), frozen(&mut dom, &right));
                // These are the actual caller's counts: math carries source
                // content although it supplies neither a lexical title nor
                // a list item. The titled side has only one w:t paragraph.
                let n1 = contentful_group_sha1s(&dom, &left).len();
                let n2 = contentful_group_sha1s(&dom, &right).len();
                assert_eq!((n1, n2), if reverse { (3, 2) } else { (2, 3) });
                assert_eq!(short_demo_list_x_prose(&dom, &left, &right, n1, n2), title);
                assert!(!titles_share_last_sig(&dom, &left, &right));
                assert_eq!(
                    (frozen(&mut dom, &left), frozen(&mut dom, &right)),
                    before,
                    "classification must preserve every source math property, paragraph mark and owner"
                );
            }
        }
    }
}

fn exact_window_ownership(
    dom: &mut Dom,
    seqs: &[CorrelatedSequence],
    expected: &(Vec<OwnedAtom>, Vec<OwnedAtom>),
    label: &str,
) {
    for revised in [false, true] {
        let units = seqs
            .iter()
            .flat_map(|seq| {
                if revised {
                    seq.com_units_2.as_deref().unwrap_or_default()
                } else {
                    seq.com_units_1.as_deref().unwrap_or_default()
                }
            })
            .cloned()
            .collect::<Vec<_>>();
        let actual = frozen(dom, &units);
        let authored = if revised { &expected.1 } else { &expected.0 };
        assert_eq!(
            actual.len(),
            authored.len(),
            "{label}: revised={revised} atom count"
        );
        let difference = actual.iter().zip(authored).position(|(a, b)| a != b);
        assert_eq!(
            difference, None,
            "{label}: revised={revised}, source atom/properties/geometry order"
        );
    }
}

#[test]
fn production_field_control_windows_keep_authored_payloads_under_supported_options() {
    let field = |instruction: &str, result: &str| {
        format!(
            "<w:r><w:fldChar w:fldCharType='begin'/></w:r><w:r><w:instrText xml:space='preserve'> {instruction} </w:instrText></w:r><w:r><w:fldChar w:fldCharType='separate'/></w:r><w:r><w:rPr><w:i/></w:rPr><w:t>{result}</w:t></w:r><w:r><w:fldChar w:fldCharType='end'/></w:r>"
        )
    };
    let old_body = "obsolete zoological vocabulary distinguishes archived chapters";
    let new_body = "replacement mechanical wording describes brandnew sections";
    for family in [
        "date",
        "ref",
        "changed-ref",
        "simple-ref",
        "inline-control",
        "tabs",
        "space-case",
    ] {
        let contents = |revised: bool| {
            let text = if revised { new_body } else { old_body };
            let payload = match family {
                "date" => field("DATE", text),
                "ref" => field("REF Clause", text),
                "changed-ref" => field(if revised { "REF Other" } else { "REF Clause" }, text),
                "simple-ref" => format!(
                    "<w:fldSimple w:instr='REF Clause'><w:r><w:rPr><w:b/></w:rPr><w:t>{text}</w:t></w:r></w:fldSimple>"
                ),
                "inline-control" => format!(
                    "<w:sdt><w:sdtPr><w:id w:val='42'/><w:tag w:val='Clause'/><w:alias w:val='Clause owner'/></w:sdtPr><w:sdtContent><w:r><w:rPr><w:u w:val='single'/></w:rPr><w:t>{text}</w:t></w:r></w:sdtContent></w:sdt>"
                ),
                "tabs" => format!(
                    "<w:r><w:tab/><w:br w:type='page'/><w:t>{text}</w:t><w:br w:clear='all'/><w:tab/></w:r>"
                ),
                "space-case" => format!(
                    "<w:r><w:t>{}</w:t></w:r>",
                    if revised {
                        "SHARED anchor replacement"
                    } else {
                        "shared anchor obsolete"
                    }
                ),
                _ => unreachable!(),
            };
            format!(
                "<w:p><w:pPr><w:spacing w:after='120'/></w:pPr><w:bookmarkStart w:id='1' w:name='Clause'/><w:bookmarkEnd w:id='1'/><w:bookmarkStart w:id='2' w:name='Other'/><w:bookmarkEnd w:id='2'/>{payload}</w:p>"
            )
        };
        for option in 0..5 {
            let mut settings = if option == 0 {
                WmlComparerSettings::powertools_faithful()
            } else {
                WmlComparerSettings::default()
            };
            settings.case_insensitive = option >= 3;
            settings.conflate_breaking_and_nonbreaking_spaces = option == 4;
            settings.detail_threshold = if option == 2 {
                1.0
            } else if option == 3 {
                0.0
            } else {
                super::super::DEFAULT_DETAIL_THRESHOLD
            };
            if option == 4 {
                settings.word_separators.push(' ');
            }
            for fragment in [false, true] {
                let mut dom = Dom::new();
                let mut left = source(&mut dom, &contents(false), &settings)
                    .iter()
                    .flat_map(group_contents)
                    .collect::<Vec<_>>();
                let mut right = source(&mut dom, &contents(true), &settings)
                    .iter()
                    .flat_map(group_contents)
                    .collect::<Vec<_>>();
                if fragment {
                    left.retain(|u| !unit_is_single_atom_ppr(&dom, u));
                    right.retain(|u| !unit_is_single_atom_ppr(&dom, u));
                    settings.in_word_level_paragraph = true;
                }
                let expected = (frozen(&mut dom, &left), frozen(&mut dom, &right));
                let unknown = CorrelatedSequence::paired(CorrelationStatus::Unknown, left, right);
                let resolved = resolve_paragraph_window(&mut dom, unknown, &settings);
                let out = match resolved {
                    Ok(out) => out,
                    Err(original) => vec![original],
                };
                exact_window_ownership(
                    &mut dom,
                    &out,
                    &expected,
                    &format!("{family} option={option} fragment={fragment}"),
                );
                if option == 0 {
                    assert_eq!(out.len(), 1);
                    assert_eq!(out[0].correlation_status, CorrelationStatus::Unknown);
                } else {
                    assert!(
                        out.iter()
                            .all(|seq| seq.correlation_status != CorrelationStatus::Unknown)
                    );
                }
            }
        }
    }
}

#[test]
fn instruction_only_and_nontext_only_windows_decline_without_losing_field_boundaries() {
    for payload in [
        "<w:r><w:fldChar w:fldCharType='begin'/><w:instrText xml:space='preserve'> DATE </w:instrText><w:fldChar w:fldCharType='separate'/><w:fldChar w:fldCharType='end'/></w:r>",
        "<w:r><w:tab/><w:br w:type='page'/></w:r>",
        "<m:oMath><m:r><m:t>x+y</m:t></m:r></m:oMath>",
    ] {
        let mut dom = Dom::new();
        let settings = WmlComparerSettings::default();
        let left = source(&mut dom, &format!("<w:p>{payload}</w:p>"), &settings)
            .iter()
            .flat_map(group_contents)
            .collect::<Vec<_>>();
        let right = source(
            &mut dom,
            &paragraph("Actual revised words", "Normal", None, false),
            &settings,
        )
        .iter()
        .flat_map(group_contents)
        .collect::<Vec<_>>();
        let expected = (frozen(&mut dom, &left), frozen(&mut dom, &right));
        let original = resolve_paragraph_window(
            &mut dom,
            CorrelatedSequence::paired(CorrelationStatus::Unknown, left, right),
            &settings,
        )
        .unwrap_err();
        assert_eq!(original.correlation_status, CorrelationStatus::Unknown);
        exact_window_ownership(
            &mut dom,
            &[original],
            &expected,
            "wordless instruction/nontext window",
        );
    }
}

// These are related short stories with different residual paragraph
// shapes. The unrelated 20-family sweeps do not exercise their vocabulary,
// overlap, title or first/last residual guards.
fn related_residual_pair(
    family: usize,
) -> (
    &'static str,
    &'static str,
    &'static [&'static str],
    &'static [&'static str],
) {
    match family {
        0 => (
            "Large Font Demo",
            "Small Font Demo",
            &[
                "This document demonstrates font size in ordinary body text",
                "Larger font sizes improve readability for the final clause",
            ],
            &[
                "This document demonstrates font size with a revised caption",
                "This text uses a larger font size of eighteen points",
                "Font size impacts spacing and readability",
            ],
        ),
        1 => (
            "Text Highlight Demo",
            "Blue Underline Demo",
            &[
                "This text combines bold and underline",
                "Underline remains visible in the concluding body clause",
            ],
            &[
                "This document demonstrates a blue underline",
                "Blue formatting remains visible in a different body clause",
            ],
        ),
        2 => (
            "Paragraph Heading Demo",
            "Paragraph Body Demo",
            &[
                "This paragraph demonstrates an original heading style",
                "This text follows the original heading",
                "The original final paragraph closes the story",
            ],
            &[
                "This paragraph demonstrates revised heading text",
                "The revised final paragraph closes the story",
            ],
        ),
        3 => (
            "Bold Formatting Demo",
            "Italic Formatting Demo",
            &[
                "This document demonstrates bold text",
                "Bold formatting marks the original final body",
            ],
            &[
                "This document demonstrates italic text with additional words",
                "Italic formatting changes the revised final body",
            ],
        ),
        4 => (
            "Numbered List Demo",
            "Numbered Intro Demo",
            &["First item", "Second item"],
            &[
                "This document introduces the numbered list",
                "First italic item",
                "Second italic item",
            ],
        ),
        5 => (
            "Font Family Demo",
            "Font Colour Demo",
            &[
                "This document demonstrates several different font families",
                "This text is rendered using a selected font",
                "The last original clause records the body style",
            ],
            &[
                "This document demonstrates several different font colours",
                "Different colours improve distinction across the body text",
                "The last revised clause records the updated body style",
            ],
        ),
        _ => unreachable!("closed deterministic family table"),
    }
}

fn related_residual_story(
    title: &str,
    paragraphs: &[&str],
    revised: bool,
    family: usize,
    blanks: usize,
) -> String {
    let mut xml = String::new();
    if blanks & 1 != 0 {
        xml.push_str(&paragraph("", "OpeningLayout", None, revised));
    }
    xml.push_str(&paragraph(title, "Title", None, revised));
    for (index, text) in paragraphs.iter().enumerate() {
        let level = (family == 4 && !text.starts_with("This")).then_some(u32::from(index > 0));
        xml.push_str(&paragraph(
            text,
            if index == 0 { "FirstBody" } else { "LaterBody" },
            level,
            revised,
        ));
    }
    if blanks & 2 != 0 {
        xml.push_str(&paragraph("", "ClosingLayout", None, revised));
    }
    xml
}

fn exercise_related_residual_boundaries(family: usize) {
    let (title_a, title_b, body_a, body_b) = related_residual_pair(family);
    for blanks in 0..4 {
        for word in [false, true] {
            for threshold in [0.0, 0.15, 1.0] {
                for reverse in [false, true] {
                    let settings = WmlComparerSettings {
                        merge_replaced_paragraphs: word,
                        detail_threshold: threshold,
                        ..WmlComparerSettings::default()
                    };
                    let a = related_residual_story(title_a, body_a, false, family, blanks);
                    let b = related_residual_story(title_b, body_b, true, family, blanks);
                    let (a, b) = if reverse { (&b, &a) } else { (&a, &b) };
                    let mut dom = Dom::new();
                    let left = source(&mut dom, a, &settings);
                    let right = source(&mut dom, b, &settings);
                    let expected = (frozen(&mut dom, &left), frozen(&mut dom, &right));
                    let label = format!(
                        "related family={family} blanks={blanks} Word={word} threshold={threshold} reverse={reverse}"
                    );
                    let out = step_h(&mut dom, &left, &right, &settings);
                    assert_owned(
                        &mut dom,
                        &out,
                        &expected,
                        &format!("block dispatch {label}"),
                    );
                    let left_words = flatten_groups_one_level(&left);
                    let right_words = flatten_groups_one_level(&right);
                    let expected_words = (
                        frozen(&mut dom, &left_words),
                        frozen(&mut dom, &right_words),
                    );
                    let out = step_h(&mut dom, &left_words, &right_words, &settings);
                    assert_owned(
                        &mut dom,
                        &out,
                        &expected_words,
                        &format!("word dispatch {label}"),
                    );
                    let out = do_lcs_algorithm(
                        &mut dom,
                        CorrelatedSequence::paired(
                            CorrelationStatus::Unknown,
                            left.clone(),
                            right.clone(),
                        ),
                        &settings,
                    );
                    assert_owned(
                        &mut dom,
                        &out,
                        &expected,
                        &format!("complete LCS step {label}"),
                    );
                    // Resolve the actual caller worklist, not only the first
                    // structural dispatch. Both authored streams remain
                    // independent frozen oracles across recursive windows.
                    let out = resolve_correlated_sequences(
                        &mut dom,
                        vec![CorrelatedSequence::paired(
                            CorrelationStatus::Unknown,
                            left,
                            right,
                        )],
                        &settings,
                    );
                    assert!(
                        out.iter()
                            .all(|s| s.correlation_status != CorrelationStatus::Unknown)
                    );
                    assert_owned(
                        &mut dom,
                        &out,
                        &expected,
                        &format!("resolved worklist {label}"),
                    );
                }
            }
        }
    }
}

#[test]
fn related_font_size_residuals_keep_every_authored_owner_at_each_dispatch() {
    exercise_related_residual_boundaries(0);
}
#[test]
fn related_highlight_underline_residuals_keep_every_authored_owner_at_each_dispatch() {
    exercise_related_residual_boundaries(1);
}
#[test]
fn related_heading_body_residuals_keep_every_authored_owner_at_each_dispatch() {
    exercise_related_residual_boundaries(2);
}
#[test]
fn related_bold_italic_residuals_keep_every_authored_owner_at_each_dispatch() {
    exercise_related_residual_boundaries(3);
}
#[test]
fn related_numbered_intro_residuals_keep_every_authored_owner_at_each_dispatch() {
    exercise_related_residual_boundaries(4);
}
#[test]
fn related_font_family_colour_residuals_keep_every_authored_owner_at_each_dispatch() {
    exercise_related_residual_boundaries(5);
}

fn exercise(family: &str) {
    for other in [
        "plain",
        "demo",
        "stamped",
        "labels",
        "numbered",
        "nested",
        "html",
        "annotation",
        "cover",
        "wrap",
        "math",
        "table",
        "digits",
        "tableprose",
        "multitable",
        "sectioned",
        "statistics",
    ] {
        for count in [5, 8, 12, 21, 31] {
            for shared in [0, 3, 8] {
                for reverse in [false, true] {
                    let left_xml = body(
                        family,
                        "original",
                        if family == "wrap" { 1 } else { 3 },
                        shared,
                        false,
                    );
                    let right_xml = body(other, "revised", count, shared, false);
                    let (a, b) = if reverse {
                        (&right_xml, &left_xml)
                    } else {
                        (&left_xml, &right_xml)
                    };
                    let settings = WmlComparerSettings::default();
                    let mut dom = Dom::new();
                    let left = source(&mut dom, a, &settings);
                    let right = source(&mut dom, b, &settings);
                    assert!(!left.is_empty() && !right.is_empty());
                    let expected = (frozen(&mut dom, &left), frozen(&mut dom, &right));
                    let label = format!(
                        "{family} x {other}, n={count}, shared={shared}, reverse={reverse}"
                    );
                    if let Some(out) =
                        detect_unrelated_sources_word_mode_inner(&mut dom, &left, &right, &settings)
                    {
                        assert_owned(&mut dom, &out, &expected, &format!("unrelated {label}"));
                    }
                    let out = step_h(&mut dom, &left, &right, &settings);
                    assert_owned(&mut dom, &out, &expected, &format!("step_h {label}"));
                    // These are the production recursion's structural windows:
                    // paragraphs flatten to words, tables to rows, rows to cells.
                    // Keep each window's independently frozen source oracle.
                    let flat_left = flatten_groups_one_level(&left);
                    let flat_right = flatten_groups_one_level(&right);
                    let expected_flat =
                        (frozen(&mut dom, &flat_left), frozen(&mut dom, &flat_right));
                    let out = step_h(&mut dom, &flat_left, &flat_right, &settings);
                    assert_owned(
                        &mut dom,
                        &out,
                        &expected_flat,
                        &format!("word/row window {label}"),
                    );
                    let tables = |units: &[ComparisonUnit]| {
                        units
                            .iter()
                            .filter(|u| {
                                as_group(u)
                                    .is_some_and(|g| g.group_type == ComparisonUnitGroupType::Table)
                            })
                            .cloned()
                            .collect::<Vec<_>>()
                    };
                    let mut table_left = tables(&left);
                    let mut table_right = tables(&right);
                    for depth in 0..3 {
                        if table_left.is_empty() || table_right.is_empty() {
                            break;
                        }
                        let expected_window = (
                            frozen(&mut dom, &table_left),
                            frozen(&mut dom, &table_right),
                        );
                        let out = step_h(&mut dom, &table_left, &table_right, &settings);
                        assert_owned(
                            &mut dom,
                            &out,
                            &expected_window,
                            &format!("table recursion depth={depth} {label}"),
                        );
                        table_left = flatten_groups_one_level(&table_left);
                        table_right = flatten_groups_one_level(&table_right);
                    }
                }
            }
        }
    }
}

fn numbered_admission_story(
    count: usize,
    words: usize,
    numbered: usize,
    nested_at: Option<usize>,
    revised: bool,
    uniform: Option<&str>,
    shared_filler: bool,
) -> String {
    let vocabulary = if revised {
        [
            "violet", "cobalt", "silver", "glacier", "harbor", "mariner", "sapphire", "summit",
        ]
    } else {
        [
            "amber", "bronze", "copper", "saffron", "orchard", "walnut", "meadow", "harvest",
        ]
    };
    (0..count)
        .map(|index| {
            let text = uniform.map(str::to_string).unwrap_or_else(|| {
                let mut tokens = vec![vocabulary[index % vocabulary.len()]];
                tokens.extend(std::iter::repeat_n(
                    if shared_filler {
                        "amber"
                    } else if revised {
                        "revised"
                    } else {
                        "former"
                    },
                    words.saturating_sub(1),
                ));
                tokens.join(" ")
            });
            paragraph(
                &text,
                "ListParagraph",
                (index < numbered).then_some(if nested_at == Some(index) { 1 } else { 0 }),
                revised,
            )
        })
        .collect()
}

fn assert_public_numbered_admission_sources(left: &str, right: &str, label: &str) {
    let package = |story: &str| {
        let mut package = crate::opc::PartFs::open(include_bytes!(
            "../../../../../../tests/fixtures/relids/image_doc.docx"
        ))
        .unwrap();
        package.set_part(
                "word/document.xml",
                format!("<w:document xmlns:w='{}'><w:body>{story}<w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr></w:body></w:document>", W::URI).into_bytes(),
            );
        package.set_part(
                "word/styles.xml",
                format!("<w:styles xmlns:w='{}'><w:style w:type='paragraph' w:default='1' w:styleId='Normal'><w:name w:val='Normal'/></w:style><w:style w:type='paragraph' w:styleId='ListParagraph'><w:name w:val='ListParagraph'/></w:style></w:styles>", W::URI).into_bytes(),
            );
        package.set_part(
                "word/numbering.xml",
                format!("<w:numbering xmlns:w='{}'><w:abstractNum w:abstractNumId='1'><w:multiLevelType w:val='multilevel'/><w:lvl w:ilvl='0'><w:start w:val='1'/><w:numFmt w:val='decimal'/><w:lvlText w:val='%1.'/><w:lvlJc w:val='left'/></w:lvl><w:lvl w:ilvl='1'><w:start w:val='1'/><w:numFmt w:val='lowerLetter'/><w:lvlText w:val='%2.'/><w:lvlJc w:val='left'/></w:lvl></w:abstractNum><w:num w:numId='9'><w:abstractNumId w:val='1'/></w:num></w:numbering>", W::URI).into_bytes(),
            );
        package.to_zip().unwrap()
    };
    let a = package(left);
    let b = package(right);
    for word in [false, true] {
        let mut settings = if word {
            WmlComparerSettings::default()
        } else {
            WmlComparerSettings::powertools_faithful()
        };
        settings.author_for_revisions = "Numbered admission editor".into();
        settings.date_time_for_revisions = "2026-01-02T03:04:05Z".into();
        let compared =
            crate::document_comparer::compare_documents_with_settings(&a, &b, &settings).unwrap();
        for (accept, source) in [(false, &a), (true, &b)] {
            let projection = if accept {
                crate::document_comparer::accept_revisions(&compared)
            } else {
                crate::document_comparer::reject_revisions(&compared)
            }
            .unwrap();
            let events = |bytes: &[u8]| {
                let events = opaque_source_events(bytes, true);
                if word {
                    // Word may absorb/reassign carrier paragraph properties;
                    // ordered characters and their complete authored run
                    // properties remain independent source owners.
                    events
                        .into_iter()
                        .filter(|event| event.starts_with("text:"))
                        .collect::<Vec<_>>()
                } else {
                    // Faithful keeps the complete paragraph/property tree,
                    // including every revised closing paragraph mark.
                    events
                }
            };
            assert_eq!(
                events(&projection),
                events(source),
                "public complete source {label}/word={word}/accept={accept}"
            );
            let repeated = if accept {
                crate::document_comparer::accept_revisions(&projection)
            } else {
                crate::document_comparer::reject_revisions(&projection)
            }
            .unwrap();
            assert_eq!(
                events(&repeated),
                events(&projection),
                "public repeat {label}/word={word}/accept={accept}"
            );
        }
    }
}

fn exercise_numbered_admission_sources(
    left_xml: &str,
    right_xml: &str,
    label: &str,
    expected_statuses: Option<&[CorrelationStatus]>,
) {
    let settings = WmlComparerSettings {
        author_for_revisions: "Numbered admission editor".to_string(),
        date_time_for_revisions: "2026-01-02T03:04:05Z".to_string(),
        ..WmlComparerSettings::default()
    };
    let mut dom = Dom::new();
    // These are the actual preprocess + atomizer + group-builder units;
    // no source hashes, statuses, ancestry or PT flags are invented.
    let left = source(&mut dom, left_xml, &settings);
    let right = source(&mut dom, right_xml, &settings);
    let expected = (frozen(&mut dom, &left), frozen(&mut dom, &right));
    let decision = detect_unrelated_sources_word_mode(&mut dom, &left, &right, &settings);
    let proposed = match decision {
        Some((proposed, _)) => proposed,
        None => vec![CorrelatedSequence::paired(
            CorrelationStatus::Unknown,
            left,
            right,
        )],
    };
    if let Some(expected_statuses) = expected_statuses {
        assert_eq!(
            proposed
                .iter()
                .map(|s| s.correlation_status)
                .collect::<Vec<_>>(),
            expected_statuses,
            "documented Word list shape {label}"
        );
    }
    assert_owned(
        &mut dom,
        &proposed,
        &expected,
        &format!("admission {label}"),
    );
    let resolved = resolve_correlated_sequences(&mut dom, proposed, &settings);
    assert!(
        resolved
            .iter()
            .all(|s| s.correlation_status != CorrelationStatus::Unknown),
        "complete public worklist resolution {label}"
    );
    assert_owned(
        &mut dom,
        &resolved,
        &expected,
        &format!("resolved admission {label}"),
    );
    // Word's documented interior carrier transfers the revised final
    // pilcrow to an original carrier. assert_owned checks its exact node
    // identity and all remaining properties; a public faithful projection
    // below independently retains every authored paragraph and property.
    assert_public_numbered_admission_sources(left_xml, right_xml, label);
}

#[test]
fn public_numbered_stub_admission_keeps_source_owners_at_real_density_and_prefix_edges() {
    for base_count in [5usize, 6, 7] {
        for numbered in [(base_count - 1) / 2, base_count.div_ceil(2), base_count] {
            for labels in [
                &["ONE", "a", "b", "c"][..],
                &["a", "b", "c", "d"][..],
                &["ONE", "TWO", "TEN", "SIX"][..],
                &["ONE", "a", "b"][..],
                &["ONE", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j"][..],
            ] {
                let left =
                    numbered_admission_story(base_count, 3, numbered, None, false, None, false);
                let right = labels
                    .iter()
                    .map(|text| paragraph(text, "ListParagraph", Some(0), true))
                    .collect::<String>();
                exercise_numbered_admission_sources(
                    &left,
                    &right,
                    &format!("M429 count={base_count}/numbered={numbered}/labels={labels:?}"),
                    None,
                );
            }
        }
    }
}

#[test]
fn public_nested_list_admission_keeps_exact_sources_when_one_earlier_operand_changes() {
    type Case = (&'static str, usize, usize, usize, usize, Option<usize>);
    let cases: &[Case] = &[
        ("documented interior cluster", 12, 12, 8, 5, Some(1)),
        ("old item one word beyond cap", 13, 12, 8, 5, Some(1)),
        ("new item one word beyond cap", 12, 13, 8, 5, Some(1)),
        ("old list below half", 12, 12, 3, 5, Some(1)),
        ("old list exactly half", 12, 12, 4, 5, Some(1)),
        ("new list below half", 12, 12, 8, 2, Some(1)),
        ("new list above half", 12, 12, 8, 3, Some(1)),
        (
            "nested first item closes cluster at one",
            12,
            12,
            8,
            5,
            Some(0),
        ),
        (
            "nested last item has no following cluster",
            12,
            12,
            8,
            5,
            Some(7),
        ),
        ("no authored nested item", 12, 12, 8, 5, None),
    ];
    for &(label, old_words, new_words, old_numbered, new_numbered, nested_at) in cases {
        let left =
            numbered_admission_story(8, old_words, old_numbered, nested_at, false, None, false);
        let right = numbered_admission_story(5, new_words, new_numbered, None, true, None, false);
        let expected = if label == "documented interior cluster" {
            Some(
                &[
                    CorrelationStatus::Inserted,
                    CorrelationStatus::Deleted,
                    CorrelationStatus::Inserted,
                    CorrelationStatus::Deleted,
                ][..],
            )
        } else {
            None
        };
        exercise_numbered_admission_sources(&left, &right, label, expected);
    }
}

#[test]
fn public_uniform_numbered_items_distinguish_disjoint_and_shared_cluster_ownership() {
    for &(label, uniform, shared_filler, expected) in &[
        (
            "disjoint uniform list uses wholesale ownership",
            "test",
            false,
            &[CorrelationStatus::Inserted, CorrelationStatus::Deleted][..],
        ),
        (
            "shared uniform list retains interior cluster",
            "amber",
            true,
            &[
                CorrelationStatus::Inserted,
                CorrelationStatus::Deleted,
                CorrelationStatus::Inserted,
                CorrelationStatus::Deleted,
            ][..],
        ),
    ] {
        let left = numbered_admission_story(8, 3, 8, Some(1), false, None, shared_filler);
        let right = numbered_admission_story(5, 1, 5, None, true, Some(uniform), false);
        exercise_numbered_admission_sources(&left, &right, label, Some(expected));
    }
}

fn structured_guard_table(revised: bool, columns: usize, nested: bool) -> String {
    let width = 5400 / columns;
    let mut grid = String::new();
    let mut cells = String::new();
    for column in 0..columns {
        grid.push_str(&format!("<w:gridCol w:w='{width}'/>"));
        let text = if revised {
            format!("revised cell {column} value")
        } else {
            format!("original cell {column} value")
        };
        let inner = if nested && column == 0 {
            structured_guard_table(revised, 1, false)
        } else {
            String::new()
        };
        cells.push_str(&format!("<w:tc><w:tcPr><w:tcW w:w='{width}' w:type='dxa'/><w:shd w:val='clear' w:fill='{}'/></w:tcPr>{}{inner}<w:p><w:pPr><w:spacing w:after='80'/></w:pPr></w:p></w:tc>",if revised {"ABCDEF"} else {"123456"},paragraph(&text,"CellBody",None,revised)));
    }
    format!(
        "<w:tbl><w:tblPr><w:tblW w:w='5400' w:type='dxa'/><w:tblBorders><w:top w:val='single' w:sz='8' w:color='445566'/></w:tblBorders></w:tblPr><w:tblGrid>{grid}</w:tblGrid><w:tr><w:trPr><w:trHeight w:val='320'/></w:trPr>{cells}</w:tr></w:tbl>"
    )
}

fn structured_guard_story(family: usize, revised: bool, root_math: usize, layout: usize) -> String {
    let (title, texts): (&str, Vec<&str>) = match (family, revised) {
        (0, false) => (
            "OOXML w:b Property Tester",
            vec!["A) Bold sample", "B) Plain sample"],
        ),
        (0, true) => (
            "Table alignment document",
            vec!["Aligned cell contents follow"],
        ),
        (1, false) => ("ST_OnOff property tester", vec!["w:b true sample"]),
        (1, true) => (
            "Here is some text about a comment",
            vec!["A revised comment anchors this ordinary body"],
        ),
        (2, false) => (
            "OOXML w:color tester",
            vec![
                "A) Red colour sample",
                "B) Blue colour sample",
                "C) Plain colour sample",
            ],
        ),
        (2, true) => (
            "OOXML w:highlight tester",
            vec![
                "A) Yellow highlight sample",
                "B) Green highlight sample",
                "C) Plain highlight sample",
            ],
        ),
        (3, false) => ("Cell table document", vec!["Short cell vocabulary"]),
        (3, true) => (
            "Review table document",
            vec![
                "Department review follows the original introduction",
                "A second paragraph explains the updated review process",
            ],
        ),
        (4, false) => (
            "Long table inventory document",
            vec!["Multiple independently owned tables follow"],
        ),
        (4, true) => (
            "Short table inventory document",
            vec!["A revised table follows"],
        ),
        (5, false) => (
            "Original sectioned document",
            vec![
                "A) First original clause",
                "B) Second original clause",
                "C) Third original clause",
            ],
        ),
        (5, true) => (
            "Revised sectioned document",
            vec![
                "A) First revised clause",
                "B) Second revised clause",
                "C) Third revised clause",
            ],
        ),
        (6, false) => (
            "Original lettered list document",
            vec![
                "A) Original list lead",
                "a) Original nested item",
                "B) Original second item",
            ],
        ),
        (6, true) => (
            "Revised lettered list document",
            vec![
                "A) Revised list lead",
                "b) Revised nested item",
                "B) Revised second item",
            ],
        ),
        (7, false) => (
            "Original mathematics document",
            vec![
                "This document describes m:borderbox and m:box notation",
                "Original mathematics follows the description",
            ],
        ),
        (7, true) => (
            "Revised mathematics document",
            vec![
                "This document describes m:box notation and a revised calculation",
                "Revised mathematics follows the description",
            ],
        ),
        _ => unreachable!("closed structured source family"),
    };
    let blank = paragraph("", "LayoutBoundary", None, revised);
    let math = match root_math {
        1 => "<m:oMath><m:r><m:rPr><m:sty m:val='p'/></m:rPr><m:t>x+y</m:t></m:r></m:oMath>",
        2 => {
            "<m:oMathPara><m:oMathParaPr><m:jc m:val='center'/></m:oMathParaPr><m:oMath><m:r><m:rPr><m:sty m:val='p'/></m:rPr><m:t>x+y</m:t></m:r></m:oMath></m:oMathPara>"
        }
        _ => "",
    };
    let mut xml = String::new();
    if layout & 1 != 0 {
        xml.push_str(&blank);
    }
    xml.push_str(&paragraph(title, "Title", None, revised));
    // A direct equation is a legitimate body unit alongside paragraph
    // groups. Unlike a paragraph containing math, it exercises mixed
    // Word/Group caller guards without inventing impossible empty groups.
    xml.push_str(math);
    for (index, text) in texts.iter().enumerate() {
        if index == 1 && layout & 2 != 0 {
            xml.push_str(&blank);
        }
        let level = (family == 6).then_some(if index == 1 { 1 } else { 0 });
        let para = paragraph(
            text,
            if index == 0 { "FirstBody" } else { "LaterBody" },
            level,
            revised,
        );
        if family == 2 && index == 1 {
            xml.push_str(&format!("<w:sdt><w:sdtPr><w:id w:val='42'/><w:tag w:val='SampleClause'/><w:alias w:val='Property sample'/></w:sdtPr><w:sdtEndPr><w:rPr><w:color w:val='778899'/></w:rPr></w:sdtEndPr><w:sdtContent>{para}</w:sdtContent></w:sdt>"));
        } else {
            xml.push_str(&para);
        }
    }
    let table_count = match family {
        0 => usize::from(revised),
        3 => {
            if revised {
                2
            } else {
                1
            }
        }
        4 => {
            if revised {
                1
            } else {
                4
            }
        }
        _ => 0,
    };
    for index in 0..table_count {
        xml.push_str(&structured_guard_table(
            revised,
            if revised { 3 } else { 2 },
            index == 0 && family == 3,
        ));
        xml.push_str(&paragraph(
            &format!(
                "Independent {} table tail {index}",
                if revised { "revised" } else { "original" }
            ),
            "TableTail",
            None,
            revised,
        ));
    }
    if layout & 2 != 0 {
        xml.push_str(&blank);
    }
    xml
}

fn exercise_structured_caller_guards(family: usize) {
    for root_math in 0..3 {
        for layout in 0..4 {
            for word in [false, true] {
                for reverse in [false, true] {
                    let settings = WmlComparerSettings {
                        merge_replaced_paragraphs: word,
                        ..WmlComparerSettings::default()
                    };
                    let a = structured_guard_story(family, false, root_math, layout);
                    let b = structured_guard_story(family, true, root_math, layout);
                    let (a, b) = if reverse { (&b, &a) } else { (&a, &b) };
                    let mut dom = Dom::new();
                    let left = source(&mut dom, a, &settings);
                    let right = source(&mut dom, b, &settings);
                    let expected = (frozen(&mut dom, &left), frozen(&mut dom, &right));
                    let label = format!(
                        "structured family={family}, root_math={root_math}, layout={layout}, Word={word}, reverse={reverse}"
                    );
                    if word
                        && let Some(out) = detect_unrelated_sources_word_mode_inner(
                            &mut dom, &left, &right, &settings,
                        )
                    {
                        assert_owned(&mut dom, &out, &expected, &format!("detector {label}"));
                    }
                    let out = step_h(&mut dom, &left, &right, &settings);
                    assert_owned(
                        &mut dom,
                        &out,
                        &expected,
                        &format!("block dispatcher {label}"),
                    );
                    let flat_left = flatten_groups_one_level(&left);
                    let flat_right = flatten_groups_one_level(&right);
                    let flat_expected =
                        (frozen(&mut dom, &flat_left), frozen(&mut dom, &flat_right));
                    assert_eq!(
                        flat_expected, expected,
                        "one-level expansion {label}: every source atom retains its owner"
                    );
                    let out = step_h(&mut dom, &flat_left, &flat_right, &settings);
                    assert_owned(
                        &mut dom,
                        &out,
                        &flat_expected,
                        &format!("mixed word/row dispatcher {label}"),
                    );
                    let out = resolve_correlated_sequences(
                        &mut dom,
                        vec![CorrelatedSequence::paired(
                            CorrelationStatus::Unknown,
                            left,
                            right,
                        )],
                        &settings,
                    );
                    assert!(
                        out.iter()
                            .all(|s| s.correlation_status != CorrelationStatus::Unknown)
                    );
                    assert_owned(
                        &mut dom,
                        &out,
                        &expected,
                        &format!("resolved caller {label}"),
                    );
                }
            }
        }
    }
}

fn opaque_source_events(bytes: &[u8], faithful: bool) -> Vec<String> {
    fn canonical(dom: &Dom, node: NodeId) -> String {
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
        let mut out = format!("{:?}:{attrs:?}:{:?}", dom.name(node), dom.text_value(node));
        for child in dom.nodes(node) {
            if dom.name_is(child, &W::r_pr())
                && dom
                    .attributes(child)
                    .iter()
                    .all(|(name, _)| dom.is_namespace_declaration(name))
                && dom.nodes(child).is_empty()
            {
                continue;
            }
            let value = canonical(dom, child);
            out.push_str(&format!("{}:{value}", value.len()));
        }
        out
    }
    fn walk(dom: &Dom, node: NodeId, faithful: bool, out: &mut Vec<String>) {
        if dom.name_is(node, &M::name("oMath")) || dom.name_is(node, &M::name("oMathPara")) {
            out.push(format!("math:{}", canonical(dom, node)));
            return;
        }
        if dom.name_is(node, &W::t()) {
            let props = if faithful {
                dom.ancestors(node, Some(&W::r()))
                    .first()
                    .and_then(|&run| dom.element(run, &W::r_pr()))
                    .map(|node| canonical(dom, node))
                    .unwrap_or_default()
            } else {
                String::new()
            };
            out.extend(
                dom.value(node)
                    .chars()
                    .map(|ch| format!("text:{ch}:{props}")),
            );
            return;
        }
        let structural = [W::tbl(), W::tr(), W::tc(), W::p()]
            .into_iter()
            .find(|name| dom.name_is(node, name));
        if faithful && let Some(name) = &structural {
            out.push(format!(
                "begin:{{{}}}{}",
                name.namespace_name(),
                name.local_name()
            ));
            for property in [
                W::tbl_pr(),
                W::name("tblGrid"),
                W::tr_pr(),
                W::tc_pr(),
                W::p_pr(),
            ] {
                if let Some(property) = dom.element(node, &property) {
                    out.push(canonical(dom, property));
                }
            }
        }
        for child in dom.elements(node, None) {
            if [
                W::tbl_pr(),
                W::name("tblGrid"),
                W::tr_pr(),
                W::tc_pr(),
                W::p_pr(),
                W::r_pr(),
                W::sect_pr(),
            ]
            .iter()
            .any(|name| dom.name_is(child, name))
            {
                continue;
            }
            walk(dom, child, faithful, out);
        }
        if faithful && let Some(name) = structural {
            out.push(format!(
                "end:{{{}}}{}",
                name.namespace_name(),
                name.local_name()
            ));
        }
    }
    let package = crate::opc::PartFs::open(bytes).unwrap();
    let mut dom = Dom::new();
    let document = dom.parse_xdocument(&package.part_string("word/document.xml").unwrap());
    let root = dom.root(document).unwrap();
    let mut out = Vec::new();
    walk(
        &dom,
        dom.element(root, &W::body()).unwrap(),
        faithful,
        &mut out,
    );
    out
}

#[test]
fn public_table_comparison_keeps_body_equations_and_every_source_payload_in_order() {
    let package = |fragment: String| {
        let mut package = crate::opc::PartFs::open(include_bytes!(
            "../../../../../../tests/fixtures/relids/image_doc.docx"
        ))
        .unwrap();
        package.set_part("word/document.xml",format!("<w:document xmlns:w='{}' xmlns:m='{}'><w:body>{fragment}<w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr></w:body></w:document>",W::URI,M::URI).into_bytes());
        // The source pStyle values have real, identical definitions in
        // both packages; undefined styles are deliberately normalized by
        // the public Word pipeline and are not a source-fidelity fixture.
        let styles = [
            "Normal",
            "Title",
            "BodyText",
            "FirstBody",
            "LaterBody",
            "CellBody",
            "TableTail",
        ]
        .into_iter()
        .map(|id| {
            format!("<w:style w:type='paragraph' w:styleId='{id}'><w:name w:val='{id}'/></w:style>")
        })
        .collect::<String>();
        package.set_part(
            "word/styles.xml",
            format!("<w:styles xmlns:w='{}'>{styles}</w:styles>", W::URI).into_bytes(),
        );
        package.to_zip().unwrap()
    };
    for family in [0, 3, 4] {
        for root_math in [1, 2] {
            let explicit = |fragment: String| {
                fragment.replace("<m:sty m:val='p'/></m:rPr>","<m:sty m:val='p'/></m:rPr><w:rPr><w:rFonts w:ascii='Cambria Math' w:hAnsi='Cambria Math'/></w:rPr>")
            };
            let a = package(explicit(structured_guard_story(
                family, false, root_math, 0,
            )));
            let b = package(explicit(structured_guard_story(family, true, root_math, 0)));
            for word in [false, true] {
                for reverse in [false, true] {
                    let (a, b) = if reverse { (&b, &a) } else { (&a, &b) };
                    let settings = WmlComparerSettings {
                        merge_replaced_paragraphs: word,
                        ..WmlComparerSettings::default()
                    };
                    let compared =
                        crate::document_comparer::compare_documents_with_settings(a, b, &settings)
                            .unwrap();
                    let accepted = crate::document_comparer::accept_revisions(&compared).unwrap();
                    let rejected = crate::document_comparer::reject_revisions(&compared).unwrap();
                    assert_eq!(
                        opaque_source_events(&accepted, !word),
                        opaque_source_events(b, !word),
                        "accepted family={family} math={root_math} Word={word} reverse={reverse}"
                    );
                    assert_eq!(
                        opaque_source_events(&rejected, !word),
                        opaque_source_events(a, !word),
                        "rejected family={family} math={root_math} Word={word} reverse={reverse}"
                    );
                }
            }
        }
    }
}

#[test]
fn public_justified_reverse_residual_keeps_complete_original_and_revised_paragraph_properties() {
    let package = |lines: &[&str], revised| {
        let fragment = lines
            .iter()
            .enumerate()
            .map(|(index, text)| {
                paragraph(
                    text,
                    if index == 0 { "Title" } else { "BodyText" },
                    None,
                    revised,
                )
            })
            .collect::<String>();
        let mut package = crate::opc::PartFs::open(include_bytes!(
            "../../../../../../tests/fixtures/relids/image_doc.docx"
        ))
        .unwrap();
        package.set_part("word/document.xml",format!("<w:document xmlns:w='{}'><w:body>{fragment}<w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr></w:body></w:document>",W::URI).into_bytes());
        package.set_part("word/styles.xml",format!("<w:styles xmlns:w='{}'><w:style w:type='paragraph' w:styleId='Title'><w:name w:val='Title'/></w:style><w:style w:type='paragraph' w:styleId='BodyText'><w:name w:val='BodyText'/></w:style></w:styles>",W::URI).into_bytes());
        package.to_zip().unwrap()
    };
    let original = package(
        &[
            "Justified Alignment Demo",
            "This document demonstrates revised conclusion",
        ],
        true,
    );
    let revised = package(
        &[
            "Justified Alignment Demo",
            "This document demonstrates",
            "Original trailing phrase",
        ],
        false,
    );
    for word in [false, true] {
        let settings = WmlComparerSettings {
            merge_replaced_paragraphs: word,
            ..WmlComparerSettings::default()
        };
        let compared = crate::document_comparer::compare_documents_with_settings(
            &original, &revised, &settings,
        )
        .unwrap();
        let rejected = crate::document_comparer::reject_revisions(&compared).unwrap();
        let accepted = crate::document_comparer::accept_revisions(&compared).unwrap();
        assert_eq!(
            opaque_source_events(&rejected, true),
            opaque_source_events(&original, true),
            "original complete paragraph ownership Word={word}"
        );
        assert_eq!(
            opaque_source_events(&accepted, true),
            opaque_source_events(&revised, true),
            "revised complete paragraph ownership Word={word}"
        );
    }
}

#[test]
fn one_level_source_expansion_keeps_standalone_equations_in_their_authored_position() {
    for math in ["oMath", "oMathPara"] {
        let equation =
            "<m:oMath><m:r><m:rPr><m:sty m:val='p'/></m:rPr><m:t>x+y</m:t></m:r></m:oMath>";
        let equation = if math == "oMathPara" {
            format!(
                "<m:oMathPara><m:oMathParaPr><m:jc m:val='center'/></m:oMathParaPr>{equation}</m:oMathPara>"
            )
        } else {
            equation.to_owned()
        };
        for before_table in [false, true] {
            let paragraph = paragraph("Authored property sample", "BodyText", None, false);
            let table = structured_guard_table(false, 2, false);
            let fragment = if before_table {
                format!("{paragraph}{equation}{table}")
            } else {
                format!("{table}{equation}{paragraph}")
            };
            let settings = WmlComparerSettings::default();
            let mut dom = Dom::new();
            let source = source(&mut dom, &fragment, &settings);
            let expected = frozen(&mut dom, &source);
            assert!(
                source
                    .iter()
                    .any(|unit| matches!(unit, ComparisonUnit::Word(_)))
            );
            assert!(
                source
                    .iter()
                    .any(|unit| matches!(unit, ComparisonUnit::Group(_)))
            );
            let expanded = flatten_groups_one_level(&source);
            assert_eq!(
                frozen(&mut dom, &expanded),
                expected,
                "{math}/before_table={before_table}"
            );
        }
    }
}

#[test]
fn paragraph_window_structured_shells_and_fragment_marks_keep_exact_source_owners() {
    let old_text = "amber bronze cedar delta elm fern granite hazel iris jade kiln lime moss nickel oak pine quartz reed silver thyme umber violet willow xenon yarrow zinc";
    let new_text = "apple birch copper dune earth flint grove harbor indigo jasper kelp linen maple north olive pearl river stone tulip union valley wheat yellow zephyr";
    for family in 0..8 {
        for explicit_properties in [false, true] {
            for word in [false, true] {
                for fragment in [false, true] {
                    let settings = WmlComparerSettings {
                        merge_replaced_paragraphs: word,
                        in_word_level_paragraph: fragment,
                        ..WmlComparerSettings::default()
                    };
                    let story = |text: &str, revised: bool| {
                        let rpr = if explicit_properties {
                            if revised {
                                "<w:rPr><w:i/><w:color w:val='345678'/></w:rPr>"
                            } else {
                                "<w:rPr><w:b/><w:color w:val='123456'/></w:rPr>"
                            }
                        } else {
                            ""
                        };
                        let run = format!("<w:r>{rpr}<w:t>{text}</w:t></w:r>");
                        let content = match family {
                            0 => run,
                            1 => {
                                format!("<w:r>{rpr}<w:tab/></w:r>{run}<w:r>{rpr}<w:tab/></w:r>")
                            }
                            2 => format!(
                                "<w:r>{rpr}<w:br w:type='page'/></w:r>{run}<w:r>{rpr}<w:br/></w:r>"
                            ),
                            3 => format!(
                                "<w:bookmarkStart w:id='31' w:name='Clause'/>{run}<w:bookmarkEnd w:id='31'/>"
                            ),
                            4 => format!(
                                "<w:r>{rpr}<w:fldChar w:fldCharType='begin'/></w:r><w:r>{rpr}<w:instrText xml:space='preserve'> REF Clause </w:instrText></w:r><w:r>{rpr}<w:fldChar w:fldCharType='separate'/></w:r>{run}<w:r>{rpr}<w:fldChar w:fldCharType='end'/></w:r><w:bookmarkStart w:id='31' w:name='Clause'/><w:bookmarkEnd w:id='31'/>"
                            ),
                            5 => format!("<w:fldSimple w:instr=' DATE '>{run}</w:fldSimple>"),
                            6 => format!(
                                "<w:sdt><w:sdtPr><w:alias w:val='Clause'/><w:tag w:val='stable-clause'/><w:id w:val='11'/></w:sdtPr><w:sdtContent>{run}</w:sdtContent></w:sdt>"
                            ),
                            _ => format!(
                                "<w:sdt><w:sdtPr><w:tag w:val='outer-owner'/><w:id w:val='12'/></w:sdtPr><w:sdtContent><w:sdt><w:sdtPr><w:tag w:val='inner-owner'/><w:id w:val='13'/></w:sdtPr><w:sdtContent>{run}<w:r>{rpr}<w:tab/></w:r></w:sdtContent></w:sdt></w:sdtContent></w:sdt>"
                            ),
                        };
                        let ppr = if explicit_properties {
                            if revised {
                                "<w:pPr><w:spacing w:after='240'/><w:ind w:left='360'/></w:pPr>"
                            } else {
                                "<w:pPr><w:spacing w:after='120'/><w:ind w:left='180'/></w:pPr>"
                            }
                        } else {
                            ""
                        };
                        format!("<w:p>{ppr}{content}</w:p>")
                    };
                    let mut dom = Dom::new();
                    let left_groups = source(&mut dom, &story(old_text, false), &settings);
                    let right_groups = source(&mut dom, &story(new_text, true), &settings);
                    let mut left = flatten_groups_one_level(&left_groups);
                    let right = flatten_groups_one_level(&right_groups);
                    // A judged paragraph can recursively expose a fragment
                    // after consuming its original closing mark. This is
                    // the supported in_word_level_paragraph caller state.
                    if fragment {
                        let removed = take_paragraph_mark(&dom, &mut left);
                        assert!(removed.is_some());
                    }
                    let expected = (frozen(&mut dom, &left), frozen(&mut dom, &right));
                    let unknown =
                        CorrelatedSequence::paired(CorrelationStatus::Unknown, left, right);
                    let attempt = resolve_paragraph_window(&mut dom, unknown, &settings);
                    if !word {
                        assert!(attempt.is_err());
                    }
                    let sequences = match attempt {
                        Ok(sequences) => sequences,
                        Err(unknown) => vec![unknown],
                    };
                    let label = format!(
                        "structured paragraph shell={family} properties={explicit_properties} Word={word} fragment={fragment}"
                    );
                    assert_owned(&mut dom, &sequences, &expected, &label);
                    let resolved = resolve_correlated_sequences(&mut dom, sequences, &settings);
                    assert!(
                        resolved
                            .iter()
                            .all(|seq| seq.correlation_status != CorrelationStatus::Unknown)
                    );
                    assert_owned(&mut dom, &resolved, &expected, &label);
                }
            }
        }
    }
}

#[test]
fn stamped_short_title_nesting_reaches_both_vocabulary_gates_without_residual_pairs() {
    for short_count in [2usize, 3, 6] {
        for long_count in [8usize, 20, 21] {
            for relation in 0..3 {
                for explicit_properties in [false, true] {
                    let settings = WmlComparerSettings::default();
                    let (old_title, subtitle) = match relation {
                        0 => (
                            "Copper Ledger Overview".to_owned(),
                            "Violet Glacier Items".to_owned(),
                        ),
                        1 => (
                            "Copper Ledger Overview".to_owned(),
                            "Violet Copper Items".to_owned(),
                        ),
                        _ => (
                            format!(
                                "Copper {} Overview",
                                (0..25)
                                    .map(|i| format!("oldword{i}"))
                                    .collect::<Vec<_>>()
                                    .join(" ")
                            ),
                            format!(
                                "Violet Copper {} Items",
                                (0..25)
                                    .map(|i| format!("newword{i}"))
                                    .collect::<Vec<_>>()
                                    .join(" ")
                            ),
                        ),
                    };
                    let mut a = vec!["file_130.docx".to_owned(), old_title];
                    let mut b = vec![
                        "file_7.docx".to_owned(),
                        "Independent revised main heading".to_owned(),
                        subtitle,
                    ];
                    for i in 1..short_count {
                        a.push(format!("ancient walnut clause source{i}"));
                    }
                    for i in 2..long_count {
                        b.push(format!("modern violet inventory revision{i}"));
                    }
                    let story = |lines: &[String], revised| {
                        lines
                            .iter()
                            .map(|text| {
                                if explicit_properties {
                                    paragraph(text, "BodyText", None, revised)
                                } else {
                                    format!("<w:p><w:r><w:t>{text}</w:t></w:r></w:p>")
                                }
                            })
                            .collect::<String>()
                    };
                    let mut dom = Dom::new();
                    let a = source(&mut dom, &story(&a, false), &settings);
                    let b = source(&mut dom, &story(&b, true), &settings);
                    let expected = (frozen(&mut dom, &a), frozen(&mut dom, &b));
                    assert!(stamp_residual_pairs(&dom, &a[1..], &b[1..]).is_empty());
                    let title_words = para_text_tokens_joined(&dom, &a[1]);
                    let subtitle_words = para_text_tokens_joined(&dom, &b[2]);
                    let j = token_jaccard(&title_words, &subtitle_words);
                    let shared = significant_tokens(&title_words)
                        .intersection(&significant_tokens(&subtitle_words))
                        .count();
                    assert_eq!(j + 1e-12 >= 0.08, relation == 1);
                    assert_eq!(shared > 0, relation != 0);
                    let out = stamp_confetti_then_replace(&mut dom, &a, &b, &settings).unwrap();
                    assert!(out.iter().any(|seq| {
                        seq.correlation_status == CorrelationStatus::Inserted
                            && seq
                                .com_units_2
                                .as_deref()
                                .unwrap_or_default()
                                .iter()
                                .any(|unit| unit.sha1() == b[1].sha1())
                    }));
                    let label = format!(
                        "short stamp title nesting {short_count}/{long_count} vocabulary={relation} explicit={explicit_properties}"
                    );
                    assert_owned(&mut dom, &out, &expected, &label);
                }
            }
        }
    }
}

#[test]
fn residual_forward_body_pair_respects_short_document_and_word_count_boundaries() {
    for original_count in [4usize, 5] {
        for body_words in [16usize, 17] {
            for reverse in [false, true] {
                let settings = WmlComparerSettings::default();
                let body = format!(
                    "{} needle",
                    (0..body_words - 1)
                        .map(|i| format!("oldword{i}"))
                        .collect::<Vec<_>>()
                        .join(" ")
                );
                let mut a = vec![
                    paragraph("Original lexical heading", "Title", None, false),
                    paragraph(&body, "BodyText", None, false),
                ];
                for i in 2..original_count {
                    a.push(paragraph(
                        &format!("ancient walnut sourceend{i}"),
                        "BodyText",
                        None,
                        false,
                    ));
                }
                let b = [
                    "Revised inventory catalog",
                    "violet glacier firstend",
                    "needle quartz lastend",
                    "modern copper finalend",
                ]
                .iter()
                .map(|text| paragraph(text, "BodyText", None, true))
                .collect::<String>();
                let a = a.concat();
                let mut dom = Dom::new();
                let a = source(&mut dom, &a, &settings);
                let b = source(&mut dom, &b, &settings);
                let expected = (frozen(&mut dom, &a), frozen(&mut dom, &b));
                let pairs = if reverse {
                    stamp_residual_pairs(&dom, &b, &a)
                } else {
                    stamp_residual_pairs(&dom, &a, &b)
                };
                assert_eq!(
                    pairs,
                    if !reverse && original_count == 4 && body_words == 16 {
                        vec![(1, 2)]
                    } else {
                        Vec::new()
                    },
                    "count={original_count} words={body_words} reverse={reverse}"
                );
                assert_eq!(
                    (frozen(&mut dom, &a), frozen(&mut dom, &b)),
                    expected,
                    "classification must not mutate source properties or payload"
                );
            }
        }
    }
}

#[test]
fn residual_pair_conflicts_keep_the_strongest_unique_authored_owner() {
    for (left_texts, right_texts, expected_pairs) in [
        (
            vec!["Copper Demo"],
            vec!["Copper Demo", "Cobalt Demo"],
            vec![(0, 0)],
        ),
        (
            vec!["Copper Demo", "Cobalt Demo"],
            vec!["Copper Demo"],
            vec![(0, 0)],
        ),
        (
            vec!["Copper Demo", "Cobalt Demo"],
            vec!["Copper Demo", "Copper Demo", "Cobalt Demo"],
            vec![(0, 0), (1, 2)],
        ),
    ] {
        let settings = WmlComparerSettings::default();
        let story = |lines: &[&str], revised| {
            lines
                .iter()
                .map(|text| paragraph(text, "Title", None, revised))
                .collect::<String>()
        };
        let mut dom = Dom::new();
        let a = source(&mut dom, &story(&left_texts, false), &settings);
        let b = source(&mut dom, &story(&right_texts, true), &settings);
        let expected = (frozen(&mut dom, &a), frozen(&mut dom, &b));
        assert_eq!(stamp_residual_pairs(&dom, &a, &b), expected_pairs);
        assert_eq!(
            (frozen(&mut dom, &a), frozen(&mut dom, &b)),
            expected,
            "greedy classification cannot rewrite duplicated title owners or formatting"
        );
    }
}

#[test]
fn multilingual_and_short_content_tokens_keep_every_authored_payload_in_demo_routes() {
    for (original_tail, revised_tail) in [
        (
            "租赁条款 合同附件 原始章节 支付条件 法律责任 文件签署",
            "更新目录 修订章节 新增清单 生效日期 交付地点 客户登记",
        ),
        (
            "عقد أصلي شروط تفاصيل توقيع مسؤولية",
            "قائمة جديدة بنود تحديث موعد مكان",
        ),
        ("a b c d e f", "g h i j k l"),
        (
            "Δ42 oldalpha oldbeta oldgamma olddelta oldepsilon",
            "Δ42 newalpha newbeta newgamma newdelta newepsilon",
        ),
    ] {
        let left = vec![
            "Original Alignment Demo".to_owned(),
            "This document obsolete copper walnut archival chapters".to_owned(),
            original_tail.to_owned(),
        ];
        let right = vec![
            "Revised Alignment Demo".to_owned(),
            "This document replacement violet glacier current inventory".to_owned(),
            revised_tail.to_owned(),
        ];
        // The same M180 admission shape is retained: shared significant
        // title, three real paragraphs and related This-document bodies.
        // Only the final content tokens vary at the ASCII/length gate.
        exercise_known_word_guard_window(
            &left,
            &right,
            &format!("M180 authored multilingual content {original_tail:?}/{revised_tail:?}"),
        );
    }
}

#[test]
fn cell_only_short_table_vocabulary_limits_preserve_complete_source_geometry() {
    for vocabulary in [3usize, 4, 40, 41] {
        for prose_titles in 0..=2 {
            for overlapping in [false, true] {
                let labels = (0..vocabulary)
                    .map(|i| format!("label{i}"))
                    .collect::<Vec<_>>()
                    .join(" ");
                let mut short = String::new();
                for index in 0..prose_titles {
                    short.push_str(&paragraph(
                        if index == 0 { "Clause" } else { "Appendix" },
                        "Title",
                        None,
                        false,
                    ));
                }
                short.push_str(&format!("<w:tbl><w:tblPr><w:tblW w:w='2400' w:type='dxa'/></w:tblPr><w:tblGrid><w:gridCol w:w='2400'/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w='2400' w:type='dxa'/><w:shd w:val='clear' w:fill='ABCDEF'/></w:tcPr>{}</w:tc></w:tr></w:tbl>", paragraph(&labels, "BodyText", None, false)));
                let mut long = String::new();
                for index in 0..14 {
                    let text = if overlapping && index == 0 {
                        labels.clone()
                    } else {
                        format!("reportword{index} narrativepiece{index}")
                    };
                    long.push_str(&paragraph(&text, "BodyText", None, true));
                }
                long.push_str(&table("report", 19, false));
                for reverse in [false, true] {
                    let settings = WmlComparerSettings::default();
                    let mut dom = Dom::new();
                    let (a, b) = if reverse {
                        (&long, &short)
                    } else {
                        (&short, &long)
                    };
                    let left = source(&mut dom, a, &settings);
                    let right = source(&mut dom, b, &settings);
                    let before = (frozen(&mut dom, &left), frozen(&mut dom, &right));
                    let n1 = contentful_group_sha1s(&dom, &left).len();
                    let n2 = contentful_group_sha1s(&dom, &right).len();
                    assert_eq!(
                        (n1, n2),
                        if reverse {
                            (15, prose_titles + 1)
                        } else {
                            (prose_titles + 1, 15)
                        }
                    );
                    let words =
                        para_text_tokens_from_units(&dom, if reverse { &right } else { &left });
                    assert_eq!(words.len(), vocabulary + prose_titles);
                    let j = token_jaccard(
                        &para_text_tokens_from_units(&dom, &left),
                        &para_text_tokens_from_units(&dom, &right),
                    );
                    let expected =
                        prose_titles <= 1 && (4..=40).contains(&words.len()) && j + 1e-12 < 0.12;
                    assert_eq!(
                        short_cell_table_x_long_table_doc(&dom, &left, &right, n1, n2),
                        expected,
                        "vocabulary={vocabulary} prose={prose_titles} overlapping={overlapping} reverse={reverse}"
                    );
                    assert_eq!((frozen(&mut dom, &left), frozen(&mut dom, &right)), before);
                    let label = format!(
                        "cell-only vocabulary={vocabulary} prose={prose_titles} overlap={overlapping} reverse={reverse}"
                    );
                    let proposed = step_h(&mut dom, &left, &right, &settings);
                    assert_owned(&mut dom, &proposed, &before, &label);
                    let resolved = resolve_correlated_sequences(&mut dom, proposed, &settings);
                    assert!(
                        resolved
                            .iter()
                            .all(|seq| seq.correlation_status != CorrelationStatus::Unknown)
                    );
                    assert_owned(&mut dom, &resolved, &before, &label);
                }
            }
        }
    }
}

#[test]
fn short_version_annotations_keep_significant_prefix_but_change_raw_body_operands() {
    for (original_version, revised_version) in [("", "(II) "), ("(I) ", ""), ("(v1) ", "(v2) ")] {
        let left = vec![
            "Original Alignment Demo".to_owned(),
            format!(
                "This document {original_version}demonstrates justified paragraph alignment across original archival sections independently"
            ),
        ];
        let right = vec![
            "Revised Alignment Demo".to_owned(),
            format!("This document {revised_version}demonstrates justified paragraph alignment"),
            "Violet copper distinct tail".to_owned(),
        ];
        let settings = WmlComparerSettings::default();
        let mut dom = Dom::new();
        let a = source(
            &mut dom,
            &paragraph(&left[1], "BodyText", None, false),
            &settings,
        );
        let b = source(
            &mut dom,
            &paragraph(&right[1], "BodyText", None, true),
            &settings,
        );
        let at = para_text_token_list(&dom, &a[0]);
        let bt = para_text_token_list(&dom, &b[0]);
        assert!(ordered_shared_prefix_sig(&at, &bt) >= 6);
        assert_eq!(
            at.get(2).map(String::as_str) == Some("demonstrates"),
            original_version.is_empty()
        );
        assert_eq!(
            bt.get(2).map(String::as_str) == Some("demonstrates"),
            revised_version.is_empty()
        );
        exercise_known_word_guard_window(
            &left,
            &right,
            &format!("M166 actual version annotations {original_version:?}/{revised_version:?}"),
        );
    }
}

fn exercise_known_word_guard_window(left: &[String], right: &[String], label: &str) {
    for word in [false, true] {
        for reverse in [false, true] {
            let settings = WmlComparerSettings {
                merge_replaced_paragraphs: word,
                ..WmlComparerSettings::default()
            };
            let story = |lines: &[String], revised| {
                lines
                    .iter()
                    .enumerate()
                    .map(|(index, text)| {
                        paragraph(
                            text,
                            if index == 0 { "Title" } else { "BodyText" },
                            None,
                            revised,
                        )
                    })
                    .collect::<String>()
            };
            let left_xml = story(left, false);
            let right_xml = story(right, true);
            let (left_xml, right_xml) = if reverse {
                (&right_xml, &left_xml)
            } else {
                (&left_xml, &right_xml)
            };
            let mut dom = Dom::new();
            let left = source(&mut dom, left_xml, &settings);
            let right = source(&mut dom, right_xml, &settings);
            let mut expected = (frozen(&mut dom, &left), frozen(&mut dom, &right));
            let label = format!("{label} Word={word} reverse={reverse}");
            let proposed = step_h(&mut dom, &left, &right, &settings);
            // M166/M178 deliberately absorbs only the original closing
            // pilcrow of the longer first body when its tail is meshed
            // against B's second body. The public complete-projection
            // regression above independently proves its source restoration.
            // This is the exact existing detector boundary, not a general
            // permission to lose paragraph marks in other routes.
            let absorbs_original_closing_mark = word
                && left.len() == 2
                && right.len() == 3
                && first_paras_share_last_sig(&dom, &left, &right)
                && residual_para_starts_this(&dom, &left[1])
                && residual_para_starts_this(&dom, &right[1])
                && {
                    let a = para_text_token_list(&dom, &left[1]);
                    let b = para_text_token_list(&dom, &right[1]);
                    ordered_shared_prefix_sig(&a, &b) >= 3
                        && a.get(2)
                            .is_some_and(|t| t.eq_ignore_ascii_case("demonstrates"))
                        && b.get(2)
                            .is_some_and(|t| t.eq_ignore_ascii_case("demonstrates"))
                        && b.len() <= 8
                        && a.len() > b.len()
                };
            if absorbs_original_closing_mark {
                let closing = expected.0.last().unwrap().node;
                assert!(dom.name_is(closing, &W::p_pr()));
                let still_present = proposed
                    .iter()
                    .flat_map(|seq| seq.com_units_1.as_deref().unwrap_or_default())
                    .flat_map(ComparisonUnit::descendant_atoms)
                    .any(|atom| atom.content_element == closing);
                if !still_present {
                    expected.0.pop();
                }
            }
            assert_owned(&mut dom, &proposed, &expected, &format!("StepH {label}"));
            let resolved = resolve_correlated_sequences(&mut dom, proposed, &settings);
            assert!(
                resolved
                    .iter()
                    .all(|seq| seq.correlation_status != CorrelationStatus::Unknown)
            );
            assert_owned(
                &mut dom,
                &resolved,
                &expected,
                &format!("resolved StepH {label}"),
            );
        }
    }
}

#[test]
fn short_demonstration_body_route_preserves_sources_at_each_reachable_prefix_boundary() {
    for verb in ["demonstrates", "illustrates"] {
        for revised_words in [8usize, 9] {
            for changed_prefix in [false, true] {
                let original = format!(
                    "This document {verb} justified paragraph alignment across each original archival section retained independently"
                );
                let revised = format!(
                    "This {} {verb} justified paragraph alignment across each{}",
                    if changed_prefix { "report" } else { "document" },
                    if revised_words == 9 { " revised" } else { "" }
                );
                let left = vec!["Original Alignment Demo".to_owned(), original];
                let right = vec![
                    "Revised Alignment Demo".to_owned(),
                    revised,
                    "Violet copper distinct tail".to_owned(),
                ];
                // This is the existing M166/M178 two-to-three paragraph
                // caller shape. Only a lexical operand or the documented
                // eight-word cap changes; no atom or source mark is removed.
                exercise_known_word_guard_window(
                    &left,
                    &right,
                    &format!(
                        "M166 verb={verb} words={revised_words} changed_prefix={changed_prefix}"
                    ),
                );
            }
        }
    }
}

#[test]
fn shared_demo_unrelated_residuals_keep_source_when_style_keyword_changes_eligibility() {
    for keyword in ["quartz", "heading", "paragraph", "style"] {
        for side in [false, true] {
            let mut left = vec![
                "Justified Alignment Demo".to_owned(),
                "walnut bronze orchard".to_owned(),
                "harvest copper meadow".to_owned(),
            ];
            let mut right = vec![
                "Centered Alignment Demo".to_owned(),
                "violet cobalt glacier".to_owned(),
                "silver harbor summit".to_owned(),
            ];
            if side {
                right[1] = format!("{keyword} cobalt glacier");
            } else {
                left[1] = format!("{keyword} bronze orchard");
            }
            exercise_known_word_guard_window(
                &left,
                &right,
                &format!("M149 style={keyword} revised={side}"),
            );
        }
    }
}

#[test]
fn this_text_document_role_guards_preserve_full_sources_when_each_prefix_operand_changes() {
    for left_start in ["This text", "This prose", "That text"] {
        for right_start in ["This document", "This text", "That document"] {
            let left = vec![
                "Justified Alignment Demo".to_owned(),
                format!("{left_start} quartz bronze orchard"),
                "Quartz trailing original clause".to_owned(),
            ];
            let right = vec![
                "Centered Alignment Demo".to_owned(),
                format!("{right_start} cobalt violet harbor"),
                "Cobalt trailing revised clause".to_owned(),
            ];
            exercise_known_word_guard_window(
                &left,
                &right,
                &format!("M151 {left_start}/{right_start}"),
            );
            exercise_known_word_guard_window(
                &left[1..],
                &right[1..],
                &format!("M151 title-peeled {left_start}/{right_start}"),
            );
        }
    }
}

#[test]
fn unequal_justified_residual_prefix_boundaries_keep_every_word_and_paragraph_owner() {
    let prefix = [
        "This",
        "document",
        "demonstrates",
        "justified",
        "paragraph",
        "alignment",
        "across",
        "each",
    ];
    for count in [3usize, 6, 7, 8] {
        for original_suffix in ["", " original"] {
            for revised_suffix in [" revised conclusion", " revised", ""] {
                let prefix = prefix[..count].join(" ");
                let left = vec![
                    "Justified Alignment Demo".to_owned(),
                    format!("{prefix}{original_suffix}"),
                    "Original trailing phrase".to_owned(),
                ];
                let right = vec![
                    "Justified Alignment Demo".to_owned(),
                    format!("{prefix}{revised_suffix}"),
                ];
                exercise_known_word_guard_window(
                    &left,
                    &right,
                    &format!(
                        "M152 prefix={count} original={original_suffix:?} revised={revised_suffix:?}"
                    ),
                );
            }
        }
    }
}

#[test]
fn related_first_residual_and_last_body_overlap_guards_conserve_full_payloads_and_formats() {
    for first_start in ["This document", "That document"] {
        for shared_last_words in [0usize, 1, 2, 4, 6] {
            let last = ["quartz", "bronze", "orchard", "harvest", "copper", "meadow"];
            let revised_tail = if shared_last_words == 6 {
                last.join(" ")
            } else {
                format!(
                    "{} violet cobalt glacier silver harbor summit",
                    last[..shared_last_words].join(" ")
                )
            };
            let left = vec![
                "Font Size Alignment Demo".to_owned(),
                "This document describes purple font size using common styles".to_owned(),
                last.join(" "),
            ];
            let right = vec![
                "Font Colour Alignment Demo".to_owned(),
                format!("{first_start} describes green font size using common styles"),
                revised_tail,
            ];
            exercise_known_word_guard_window(
                &left,
                &right,
                &format!("M165/M180 first={first_start} last_overlap={shared_last_words}"),
            );
        }
    }
}

#[test]
fn table_routes_distinguish_authored_math_content_from_missing_lexical_titles() {
    for long_route in [false, true] {
        for table_count in [3usize, 4] {
            for lexical_sides in 0..4 {
                for word in [false, true] {
                    for reverse in [false, true] {
                        let settings = WmlComparerSettings {
                            merge_replaced_paragraphs: word,
                            ..WmlComparerSettings::default()
                        };
                        let story = |revised: bool| {
                            let (groups, tables) = if long_route {
                                if revised { (4, 1) } else { (32, table_count) }
                            } else if revised {
                                (11, 1)
                            } else {
                                (10, table_count)
                            };
                            let lexical = lexical_sides & if revised { 2 } else { 1 } != 0;
                            let math = |index| {
                                format!(
                                    "<w:p><w:pPr><w:spacing w:after='180'/><w:jc w:val='center'/></w:pPr><m:oMath><m:r><m:rPr><m:sty m:val='p'/></m:rPr><w:rPr><w:rFonts w:ascii='Cambria Math' w:hAnsi='Cambria Math'/></w:rPr><m:t>{}equation{index}</m:t></m:r></m:oMath></w:p>",
                                    if revised { "revised" } else { "original" }
                                )
                            };
                            let mut xml = String::new();
                            for index in 0..groups - tables {
                                if index == 0 && lexical {
                                    xml.push_str(&paragraph(
                                        if revised {
                                            "Revised violet inventory"
                                        } else {
                                            "Original copper archive"
                                        },
                                        "BodyText",
                                        None,
                                        revised,
                                    ));
                                } else {
                                    xml.push_str(&math(index));
                                }
                            }
                            for index in 0..tables {
                                xml.push_str(&format!("<w:tbl><w:tblPr><w:tblW w:w='2400' w:type='dxa'/></w:tblPr><w:tblGrid><w:gridCol w:w='2400'/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w='2400' w:type='dxa'/></w:tcPr>{}</w:tc></w:tr></w:tbl>", math(index + groups)));
                            }
                            xml
                        };
                        let a = story(false);
                        let b = story(true);
                        let mut dom = Dom::new();
                        let left = source(&mut dom, if reverse { &b } else { &a }, &settings);
                        let right = source(&mut dom, if reverse { &a } else { &b }, &settings);
                        let n1 = contentful_group_sha1s(&dom, &left).len();
                        let n2 = contentful_group_sha1s(&dom, &right).len();
                        assert_eq!(
                            (n1, n2),
                            if long_route {
                                if reverse { (4, 32) } else { (32, 4) }
                            } else if reverse {
                                (11, 10)
                            } else {
                                (10, 11)
                            }
                        );
                        let expected = (frozen(&mut dom, &left), frozen(&mut dom, &right));
                        let classified = if long_route {
                            long_multitable_x_short_table_free_mesh(&dom, &left, &right, n1, n2)
                        } else {
                            both_tables_unrelated_free_mesh(&dom, &left, &right, n1, n2)
                        };
                        assert_eq!(
                            classified,
                            table_count == 4 && lexical_sides == 3,
                            "actual math content counts do not invent a lexical title"
                        );
                        assert_eq!(
                            (frozen(&mut dom, &left), frozen(&mut dom, &right)),
                            expected,
                            "classification preserves all equation/paragraph/cell owners"
                        );
                        let label = format!(
                            "math table long={long_route} tables={table_count} lexical={lexical_sides} Word={word} reverse={reverse}"
                        );
                        let out = step_h(&mut dom, &left, &right, &settings);
                        assert_owned(&mut dom, &out, &expected, &label);
                        let resolved = resolve_correlated_sequences(&mut dom, out, &settings);
                        assert!(
                            resolved
                                .iter()
                                .all(|seq| seq.correlation_status != CorrelationStatus::Unknown)
                        );
                        assert_owned(&mut dom, &resolved, &expected, &label);
                    }
                }
            }
        }
    }
}

#[test]
fn structured_property_table_guards_preserve_actual_source_geometry() {
    exercise_structured_caller_guards(0);
}
#[test]
fn structured_property_prose_guards_preserve_actual_source_ownership() {
    exercise_structured_caller_guards(1);
}
#[test]
fn structured_property_control_guards_preserve_original_metadata_and_effects() {
    exercise_structured_caller_guards(2);
}
#[test]
fn structured_nested_cell_tables_preserve_independent_row_and_cell_owners() {
    exercise_structured_caller_guards(3);
}
#[test]
fn structured_multitable_guards_preserve_every_original_and_revised_table() {
    exercise_structured_caller_guards(4);
}
#[test]
fn structured_parallel_section_guards_preserve_authored_labels_and_marks() {
    exercise_structured_caller_guards(5);
}
#[test]
fn structured_lettered_cluster_guards_preserve_list_levels_and_root_payloads() {
    exercise_structured_caller_guards(6);
}
#[test]
fn structured_math_guards_preserve_direct_equations_and_descriptive_text() {
    exercise_structured_caller_guards(7);
}

macro_rules! family_test {
    ($name:ident, $family:literal) => {
        #[test]
        fn $name() {
            exercise($family);
        }
    };
}
family_test!(ordinary_source_windows, "plain");
family_test!(demo_source_windows, "demo");
family_test!(stamped_source_windows, "stamped");
family_test!(alpha_label_source_windows, "labels");
family_test!(numbered_source_windows, "numbered");
family_test!(nested_list_source_windows, "nested");
family_test!(html_field_demo_source_windows, "html");
family_test!(annotation_source_windows, "annotation");
family_test!(title_page_source_windows, "cover");
family_test!(repeated_wrap_source_windows, "wrap");
family_test!(math_source_windows, "math");
family_test!(single_table_source_windows, "table");
family_test!(digit_table_source_windows, "digits");
family_test!(prose_table_source_windows, "tableprose");
family_test!(multiple_table_source_windows, "multitable");
family_test!(sectioned_source_windows, "sectioned");
family_test!(statistics_source_windows, "statistics");

#[test]
fn authored_filename_and_common_run_boundaries_keep_complete_source_ownership() {
    // These are real source tokens, never PT stamps or invented helper
    // states. A retained filename can require LCS even in an otherwise
    // unrelated large document; thresholds govern genuine shared runs.
    for stamp in ["file_contract", "archive.docx", "archive.doc", "archive"] {
        for common_words in [1, 4, 12] {
            for word in [false, true] {
                for threshold in [0.0, 0.15, 1.0] {
                    let settings = WmlComparerSettings {
                        merge_replaced_paragraphs: word,
                        detail_threshold: threshold,
                        ..WmlComparerSettings::default()
                    };
                    let story = |revised: bool| {
                        let owner = if revised { "violet" } else { "copper" };
                        let title = format!("{stamp} {owner} independently authored title");
                        let mut body = paragraph(&title, "Title", None, revised);
                        for index in 0..16 {
                            let tokens = (0..20)
                                .map(|token| format!("{owner}clause{index:02}word{token:02}"))
                                .collect::<Vec<_>>()
                                .join(" ");
                            let common = if index == 7 {
                                (0..common_words)
                                    .map(|token| format!("sharedclause{token:02}"))
                                    .collect::<Vec<_>>()
                                    .join(" ")
                            } else {
                                String::new()
                            };
                            let text = format!("{tokens} {common}");
                            body.push_str(&paragraph(&text, "BodyText", None, revised));
                        }
                        body
                    };
                    let a = story(false);
                    let b = story(true);
                    for reverse in [false, true] {
                        let (a, b) = if reverse { (&b, &a) } else { (&a, &b) };
                        let mut dom = Dom::new();
                        let left = source(&mut dom, a, &settings);
                        let right = source(&mut dom, b, &settings);
                        let expected = (frozen(&mut dom, &left), frozen(&mut dom, &right));
                        let label = format!(
                            "stamp={stamp} common={common_words} Word={word} threshold={threshold} reverse={reverse}"
                        );
                        let detected = if word {
                            detect_unrelated_sources_word_mode(&mut dom, &left, &right, &settings)
                                .map(|(sequences, _)| sequences)
                        } else {
                            detect_unrelated_sources(&left, &right)
                        };
                        if let Some(sequences) = detected {
                            assert_owned(
                                &mut dom,
                                &sequences,
                                &expected,
                                &format!("detector {label}"),
                            );
                        }
                        let sequences = step_h(&mut dom, &left, &right, &settings);
                        assert_owned(&mut dom, &sequences, &expected, &format!("block {label}"));
                        let words_a = flatten_groups_one_level(&left);
                        let words_b = flatten_groups_one_level(&right);
                        let expected_words =
                            (frozen(&mut dom, &words_a), frozen(&mut dom, &words_b));
                        let sequences = step_h(&mut dom, &words_a, &words_b, &settings);
                        assert_owned(
                            &mut dom,
                            &sequences,
                            &expected_words,
                            &format!("word {label}"),
                        );
                        assert_eq!(frozen(&mut dom, &left), expected.0, "original {label}");
                        assert_eq!(frozen(&mut dom, &right), expected.1, "revised {label}");
                    }
                }
            }
        }
    }
}

#[test]
fn source_memo_header_lifetimes_and_legal_size_boundaries_preserve_owned_order() {
    for count in [14, 15, 120, 121] {
        for ending in ["header", "salutation", "body"] {
            let mut memo = String::new();
            let mut prose = String::new();
            for index in 0..count {
                let label = if index + 1 == count && ending == "salutation" {
                    "Dear"
                } else if index + 1 == count && ending == "body" {
                    "originalbody"
                } else if index % 2 == 0 {
                    "TO"
                } else {
                    "FROM"
                };
                let text = format!(
                    "{label} originalopening{index} clause{index} terms{index} originaldetail{index} originalscope{index} originalparties{index} originalrecord{index} originalclosing{index}"
                );
                memo.push_str(&paragraph(&text, "MemoHeader", None, false));
                let text = format!(
                    "revisedopening{index} clause{index} terms{index} reviseddetail{index} revisedscope{index} revisedparties{index} revisedrecord{index} revisedclosing{index}"
                );
                prose.push_str(&paragraph(&text, "LegalBody", None, true));
            }
            for reverse in [false, true] {
                let settings = WmlComparerSettings::default();
                let mut dom = Dom::new();
                let memo_units = source(&mut dom, &memo, &settings);
                let prose_units = source(&mut dom, &prose, &settings);
                let expected_cut = match ending {
                    "salutation" => count,
                    "body" => count - 1,
                    "header" => count.min(12),
                    _ => unreachable!(),
                };
                assert!(looks_like_memo_doc(&dom, &memo_units));
                assert!(!looks_like_memo_doc(&dom, &prose_units));
                assert_eq!(memo_header_cut(&dom, &memo_units), Some(expected_cut));
                let (left, right) = if reverse {
                    (&prose_units, &memo_units)
                } else {
                    (&memo_units, &prose_units)
                };
                let expected = (frozen(&mut dom, left), frozen(&mut dom, right));
                let label = format!("memo count={count} ending={ending} reverse={reverse}");
                if let Some(out) =
                    detect_unrelated_sources_word_mode_inner(&mut dom, left, right, &settings)
                {
                    assert_owned(&mut dom, &out, &expected, &label);
                    if (15..=120).contains(&count) {
                        let statuses = out
                            .iter()
                            .map(|sequence| sequence.correlation_status)
                            .collect::<Vec<_>>();
                        let expected_statuses = if reverse {
                            vec![CorrelationStatus::Inserted, CorrelationStatus::Deleted]
                        } else if expected_cut == count {
                            vec![CorrelationStatus::Deleted, CorrelationStatus::Inserted]
                        } else {
                            vec![
                                CorrelationStatus::Deleted,
                                CorrelationStatus::Inserted,
                                CorrelationStatus::Deleted,
                            ]
                        };
                        assert_eq!(statuses, expected_statuses, "{label}");
                        if !reverse {
                            assert_eq!(
                                frozen(&mut dom, out[0].com_units_1.as_deref().unwrap()),
                                frozen(&mut dom, &memo_units[..expected_cut])
                            );
                        }
                    }
                } else {
                    assert!(
                        !(15..=120).contains(&count),
                        "validated legal route declined: {label}"
                    );
                }
                assert_eq!(frozen(&mut dom, left), expected.0);
                assert_eq!(frozen(&mut dom, right), expected.1);
            }
        }
    }
}
#[test]
fn compound_endpoint_boundary_correlation_keeps_every_original_character_before_production() {
    let paragraph = |text: &str| {
        format!(
            "<w:p><w:pPr><w:spacing w:before='120' w:after='80'/><w:ind w:left='180'/></w:pPr><w:r><w:rPr><w:rFonts w:ascii='Calibri' w:hAnsi='Calibri'/><w:color w:val='123456'/><w:sz w:val='22'/><w:lang w:val='en-US'/></w:rPr><w:t xml:space='preserve'>{text}</w:t></w:r></w:p>"
        )
    };
    let a = format!(
        "{}{}",
        paragraph("alpha-copper walnut"),
        paragraph("violet glacier closing")
    );
    let b = format!(
        "{}{}",
        paragraph("alpha"),
        paragraph("copper walnut violet glacier closing")
    );
    for reverse in [false, true] {
        let settings = WmlComparerSettings {
            merge_replaced_paragraphs: true,
            detect_moves: false,
            ..WmlComparerSettings::default()
        };
        let (a, b) = if reverse { (&b, &a) } else { (&a, &b) };
        let mut dom = Dom::new();
        let left = source(&mut dom, a, &settings);
        let right = source(&mut dom, b, &settings);
        let expected = (frozen(&mut dom, &left), frozen(&mut dom, &right));
        let mut sequences = detect_unrelated_sources_word_mode(&mut dom, &left, &right, &settings)
            .map(|(out, _)| out)
            .unwrap_or_else(|| lcs(&mut dom, left, right, &settings));
        assert_owned(
            &mut dom,
            &sequences,
            &expected,
            &format!("compound LCS reverse={reverse}"),
        );
        pair_story_final_marks(&dom, &mut sequences);
        super::super::cross_para::restream_cross_paragraph_regions(
            &mut dom,
            &mut sequences,
            &settings,
        );
        assert_owned(
            &mut dom,
            &sequences,
            &expected,
            &format!("compound restream reverse={reverse}"),
        );
        for seq in &sequences {
            if seq.correlation_status == CorrelationStatus::Equal {
                let before = seq
                    .com_units_1
                    .as_deref()
                    .unwrap_or_default()
                    .iter()
                    .flat_map(ComparisonUnit::descendant_atoms)
                    .collect::<Vec<_>>();
                let after = seq
                    .com_units_2
                    .as_deref()
                    .unwrap_or_default()
                    .iter()
                    .flat_map(ComparisonUnit::descendant_atoms)
                    .collect::<Vec<_>>();
                assert_eq!(
                    before.len(),
                    after.len(),
                    "compound Equal zip must retain both complete sources reverse={reverse}"
                );
            }
        }
    }
}

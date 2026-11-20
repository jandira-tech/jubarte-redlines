// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;

const DATE: &str = "2026-01-01T00:00:00Z";

fn properties(family: &str, nested: bool) -> String {
    let style = match family {
        "heading" => "Heading1",
        "liststyle" => "ListParagraph",
        _ => "BodyText",
    };
    let numbering = if family == "numbered" || nested {
        format!(
            "<w:numPr><w:ilvl w:val=\"{}\"/><w:numId w:val=\"9\"/></w:numPr>",
            usize::from(nested)
        )
    } else {
        String::new()
    };
    format!("<w:pStyle w:val=\"{style}\"/>{numbering}<w:spacing w:before=\"120\" w:after=\"80\"/>")
}

fn revision_para(kind: &str, text: &str, family: &str, props: &str, mark: bool) -> String {
    let leaf = if kind == "del" { "delText" } else { "t" };
    let marker = if mark {
        format!("<w:rPr><w:{kind} w:id=\"31\" w:author=\"Editor\" w:date=\"{DATE}\"/></w:rPr>")
    } else {
        String::new()
    };
    let payload = match family {
            "empty" => String::new(),
            "whitespace" => format!("<w:r><w:{leaf} xml:space=\"preserve\">   </w:{leaf}></w:r>"),
            "break" => "<w:r><w:br w:type=\"page\"/></w:r>".to_string(),
            "math" => "<m:oMath><m:r><w:rPr><w:rFonts w:ascii=\"Cambria Math\" w:hAnsi=\"Cambria Math\"/></w:rPr><m:t>x+y</m:t></m:r></m:oMath>".to_string(),
            "field" => format!(
                "<w:r><w:fldChar w:fldCharType=\"begin\"/></w:r><w:r><w:{} xml:space=\"preserve\"> PAGE </w:{}></w:r><w:r><w:fldChar w:fldCharType=\"separate\"/></w:r><w:r><w:{leaf}>{text}</w:{leaf}></w:r><w:r><w:fldChar w:fldCharType=\"end\"/></w:r>",
                if kind == "del" {
                    "delInstrText"
                } else {
                    "instrText"
                },
                if kind == "del" {
                    "delInstrText"
                } else {
                    "instrText"
                }
            ),
            _ => format!(
                "<w:r><w:rPr><w:b/><w:color w:val=\"123456\"/><w:lang w:val=\"en-US\"/></w:rPr><w:{leaf} xml:space=\"preserve\">{text}</w:{leaf}></w:r>"
            ),
        };
    // Empty paragraphs own only their pilcrow; nontext content owns its
    // complete structural run rather than a fabricated visible string.
    let body = if payload.is_empty() {
        String::new()
    } else {
        format!("<w:{kind} w:id=\"17\" w:author=\"Editor\" w:date=\"{DATE}\">{payload}</w:{kind}>")
    };
    format!("<w:p><w:pPr>{props}{marker}</w:pPr>{body}</w:p>")
}

fn authored_paragraph(text: &str, family: &str, props: &str) -> String {
    // Literal untracked source data: no revision processor manufactures
    // paragraph/control placement from a combined private D/I fixture.
    let payload = match family {
            "empty" => String::new(),
            "whitespace" => "<w:r><w:t xml:space=\"preserve\">   </w:t></w:r>".to_string(),
            "break" => "<w:r><w:br w:type=\"page\"/></w:r>".to_string(),
            "math" => "<m:oMath><m:r><w:rPr><w:rFonts w:ascii=\"Cambria Math\" w:hAnsi=\"Cambria Math\"/></w:rPr><m:t>x+y</m:t></m:r></m:oMath>".to_string(),
            "field" => format!("<w:r><w:fldChar w:fldCharType=\"begin\"/></w:r><w:r><w:instrText xml:space=\"preserve\"> PAGE </w:instrText></w:r><w:r><w:fldChar w:fldCharType=\"separate\"/></w:r><w:r><w:t>{text}</w:t></w:r><w:r><w:fldChar w:fldCharType=\"end\"/></w:r>"),
            _ => format!("<w:r><w:rPr><w:b/><w:color w:val=\"123456\"/><w:lang w:val=\"en-US\"/></w:rPr><w:t xml:space=\"preserve\">{text}</w:t></w:r>"),
        };
    format!("<w:p><w:pPr>{props}</w:pPr>{payload}</w:p>")
}

fn document(fragment: &str, container: &str) -> (Dom, NodeId, NodeId) {
    let fragment = match container {
        "tc" => format!(
            "<w:tbl><w:tblPr><w:tblW w:w=\"3600\" w:type=\"dxa\"/><w:tblLook w:val=\"04A0\" w:firstRow=\"1\" w:lastRow=\"0\" w:firstColumn=\"1\" w:lastColumn=\"0\" w:noHBand=\"0\" w:noVBand=\"1\"/></w:tblPr><w:tblGrid><w:gridCol w:w=\"3600\"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w=\"3600\" w:type=\"dxa\"/></w:tcPr>{fragment}<w:p/></w:tc></w:tr></w:tbl>"
        ),
        "sdtContent" => format!(
            "<w:sdt><w:sdtPr><w:id w:val=\"1\"/></w:sdtPr><w:sdtContent>{fragment}</w:sdtContent></w:sdt>"
        ),
        _ => fragment.to_string(),
    };
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&format!("<w:document xmlns:w=\"{}\" xmlns:m=\"{}\"><w:body>{fragment}<w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr></w:body></w:document>", W::URI, M::URI));
    let root = dom.root(doc).unwrap();
    let target = if container == "body" {
        dom.element(root, &W::body()).unwrap()
    } else {
        dom.descendants(root, Some(&W::name(container)))[0]
    };
    (dom, root, target)
}

#[derive(Debug, PartialEq, Eq)]
struct SourceView {
    // A character retains the formatting of the source run and source
    // paragraph even if the redline's MIX carrier changes its boundary.
    formatted_characters: Vec<(char, String, String)>,
    nontext_payloads: Vec<String>,
    table_geometry: Vec<String>,
}

fn projected(dom: &mut Dom, root: NodeId, accept: bool) -> SourceView {
    let copy = dom.clone_subtree(root);
    let root = if accept {
        crate::revision_processor::accept_revisions_document(dom, copy)
    } else {
        crate::revision_processor::reject_revisions_document(dom, copy)
    };
    remove_powertools_scratch_markup(dom, root);
    // Revision processing can leave empty property carriers; they have
    // no semantic formatting, unlike any actual property child.
    for local in ["rPr", "pPr"] {
        for n in dom.descendants(root, Some(&W::name(local))) {
            if dom.elements(n, None).is_empty() && !dom.has_attributes(n) {
                dom.remove(n);
            }
        }
    }
    let mut formatted_characters = Vec::new();
    for t in dom.descendants(root, Some(&W::t())) {
        let ancestors = dom.ancestors(t, None);
        let p = ancestors
            .iter()
            .find(|&&n| dom.name_is(n, &W::p()))
            .copied();
        let r = ancestors
            .iter()
            .find(|&&n| dom.name_is(n, &W::r()))
            .copied();
        let ppr = p
            .and_then(|p| dom.element(p, &W::p_pr()))
            .map(|n| dom.serialize_element(n))
            .unwrap_or_default();
        let rpr = r
            .and_then(|r| dom.element(r, &W::r_pr()))
            .map(|n| dom.serialize_element(n))
            .unwrap_or_default();
        for c in dom.value(t).chars() {
            formatted_characters.push((c, ppr.clone(), rpr.clone()));
        }
    }
    let nontext_payloads = dom
        .descendants(root, None)
        .into_iter()
        .filter(|&n| {
            [
                W::name("br"),
                W::fld_char(),
                W::instr_text(),
                M::name("oMath"),
            ]
            .iter()
            .any(|name| dom.name_is(n, name))
        })
        .map(|n| dom.serialize_element(n))
        .collect();
    let table_geometry = dom
        .descendants(root, None)
        .into_iter()
        .filter(|&n| {
            [W::tbl_grid(), W::tbl_pr(), W::tc_pr()]
                .iter()
                .any(|name| dom.name_is(n, name))
        })
        .map(|n| dom.serialize_element(n))
        .collect();
    SourceView {
        formatted_characters,
        nontext_payloads,
        table_geometry,
    }
}

fn text(family: &str, side: &str, index: usize, shared: bool) -> String {
    let head = if shared {
        "Shared contractual obligation"
    } else if side == "ins" {
        "Revised violet harbor"
    } else {
        "Original quartz meadow"
    };
    match family {
        "uniform" => {
            if side == "ins" {
                "test".to_string()
            } else {
                format!("Level {}", index + 1)
            }
        }
        "label" => ["ONE", "a", "TWO", "b", "THREE", "c"][index % 6].to_string(),
        "cover" => {
            if index.is_multiple_of(2) {
                "Prepared for client January 2040".to_string()
            } else {
                format!("{head} agreement")
            }
        }
        "wrap" => (0..5)
            .map(|_| format!("{head} tightly wraps this line "))
            .collect(),
        "long" => format!(
            "{head} shall deliver every completed report and preserve signed records until the review of all schedules has been completed number{index}"
        ),
        _ => format!("{head} record{index}"),
    }
}

fn compact_source_difference(actual: &SourceView, expected: &SourceView) -> String {
    let owned_difference = actual
        .formatted_characters
        .iter()
        .zip(&expected.formatted_characters)
        .position(|(actual, expected)| actual.0 != expected.0 || actual.2 != expected.2)
        .unwrap_or(
            actual
                .formatted_characters
                .len()
                .min(expected.formatted_characters.len()),
        );
    let context = |view: &SourceView| {
        view.formatted_characters
            .iter()
            .skip(owned_difference.saturating_sub(35))
            .take(90)
            .map(|(character, _, _)| *character)
            .collect::<String>()
    };
    let property_difference = actual
        .formatted_characters
        .iter()
        .zip(&expected.formatted_characters)
        .position(|(actual, expected)| actual.1 != expected.1);
    let short = |value: &str| value.chars().take(400).collect::<String>();
    let properties = property_difference
        .map(|index| {
            format!(
                "pPr difference at {index}: actual={:?}, expected={:?}",
                short(&actual.formatted_characters[index].1),
                short(&expected.formatted_characters[index].1)
            )
        })
        .unwrap_or_else(|| "pPr equal on paired characters".to_string());
    let run = actual
        .formatted_characters
        .get(owned_difference)
        .zip(expected.formatted_characters.get(owned_difference))
        .map(|(actual, expected)| {
            format!(
                "rPr actual={:?}, expected={:?}",
                short(&actual.2),
                short(&expected.2)
            )
        })
        .unwrap_or_else(|| "one source ends at the first owned difference".to_string());
    format!(
        "first owned difference={owned_difference}; chars actual={}, expected={}; actual context={:?}; expected context={:?}; {run}; {properties}; nontext equal={}, actual count={}, expected count={}; geometry equal={}, actual count={}, expected count={}",
        actual.formatted_characters.len(),
        expected.formatted_characters.len(),
        context(actual),
        context(expected),
        actual.nontext_payloads == expected.nontext_payloads,
        actual.nontext_payloads.len(),
        expected.nontext_payloads.len(),
        actual.table_geometry == expected.table_geometry,
        actual.table_geometry.len(),
        expected.table_geometry.len()
    )
}

fn exercise(family: &str) {
    let mut failures = Vec::new();
    let mut failure_count = 0;
    let mut cases = 0;
    for counts in [
        (1, 1),
        (1, 2),
        (2, 1),
        (2, 3),
        (3, 2),
        (3, 3),
        (4, 2),
        (5, 4),
        (10, 3),
    ] {
        for shared in [false, true] {
            for inserted_first in [false, true] {
                for tail in ["none", "live", "blank", "table"] {
                    for container in ["body", "tc", "sdtContent"] {
                        // These test source windows are contiguous replacement gaps,
                        // not the pending large legal mid-splice or property-demo repairs.
                        let props =
                            if family == "heading" || family == "numbered" || family == "uniform" {
                                properties(
                                    if family == "heading" {
                                        "heading"
                                    } else {
                                        "numbered"
                                    },
                                    false,
                                )
                            } else {
                                match (counts.0 + counts.1) % 4 {
                                    0 => String::new(),
                                    1 => "<w:spacing w:before=\"120\" w:after=\"80\"/>".to_string(),
                                    2 => "<w:pStyle w:val=\"BodyText\"/>".to_string(),
                                    _ => properties("ordinary", false),
                                }
                            };
                        let dels = (0..counts.0)
                            .map(|i| {
                                revision_para(
                                    "del",
                                    &text(family, "del", i, shared),
                                    family,
                                    &props,
                                    true,
                                )
                            })
                            .collect::<String>();
                        let inss = (0..counts.1)
                            .map(|i| {
                                revision_para(
                                    "ins",
                                    &text(family, "ins", i, shared),
                                    family,
                                    &props,
                                    true,
                                )
                            })
                            .collect::<String>();
                        let gap = if inserted_first {
                            inss + &dels
                        } else {
                            dels + &inss
                        };
                        let tail_kind = tail;
                        let tail = match tail {
                            "live" => "<w:p><w:r><w:t>Closing unchanged anchor</w:t></w:r></w:p>",
                            "blank" => "<w:p/>",
                            "table" => {
                                "<w:tbl><w:tblPr><w:tblW w:w=\"1800\" w:type=\"dxa\"/><w:tblLook w:val=\"04A0\" w:firstRow=\"1\" w:lastRow=\"0\" w:firstColumn=\"1\" w:lastColumn=\"0\" w:noHBand=\"0\" w:noVBand=\"1\"/></w:tblPr><w:tblGrid><w:gridCol w:w=\"1800\"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w=\"1800\" w:type=\"dxa\"/></w:tcPr><w:p><w:r><w:t>Unchanged cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"
                            }
                            _ => "",
                        };
                        let (mut dom, root, target) = document(&(gap + tail), container);
                        let accepted = projected(&mut dom, root, true);
                        let rejected = projected(&mut dom, root, false);
                        let label = format!(
                            "{family} {counts:?}, shared={shared}, inserted_first={inserted_first}, container={container}, tail={tail_kind}"
                        );
                        // The private fixture remains a combined D/I
                        // intermediate. Public comparison instead receives
                        // independently authored clean A and B with identical
                        // wrapper/shared-tail ownership. Rejecting a combined
                        // SDT D/I tree can relocate its live blank out of the
                        // control and is not a source-document constructor.
                        let mut sources = Vec::new();
                        for (side, count) in [("del", counts.0), ("ins", counts.1)] {
                            let content = (0..count)
                                .map(|index| {
                                    authored_paragraph(
                                        &text(family, side, index, shared),
                                        family,
                                        &props,
                                    )
                                })
                                .collect::<String>()
                                + tail;
                            let (source_dom, source_root, _) = document(&content, container);
                            let source_document =
                                dom.parse_xdocument(&source_dom.serialize_element(source_root));
                            sources.push(dom.root(source_document).unwrap());
                        }
                        let old_source = sources[0];
                        let new_source = sources[1];
                        let clean_rejected = projected(&mut dom, old_source, false);
                        let clean_accepted = projected(&mut dom, new_source, true);
                        let fold_boundary = container != "body" || tail_kind != "blank";
                        merge_replaced_in_container(&mut dom, target, "Editor", fold_boundary);
                        for (accept, expected) in [(false, &rejected), (true, &accepted)] {
                            let actual = projected(&mut dom, root, accept);
                            // This pass owns ordered payloads and run formats,
                            // table geometry, and stable repeat execution. Full
                            // authored pPr is checked below through its caller.
                            let runs = |view: &SourceView| {
                                view.formatted_characters
                                    .iter()
                                    .map(|(character, _, run)| (*character, run.clone()))
                                    .collect::<Vec<_>>()
                            };
                            if runs(&actual) != runs(expected)
                                || actual.nontext_payloads != expected.nontext_payloads
                                || actual.table_geometry != expected.table_geometry
                            {
                                failure_count += 1;
                                if failures.len() < 32 {
                                    failures.push(format!("private owned payload/format/geometry: accept={accept}; {label}; {}",compact_source_difference(&actual, expected)));
                                }
                            }
                        }
                        let once = dom.serialize_element(root);
                        merge_replaced_in_container(&mut dom, target, "Editor", fold_boundary);
                        if dom.serialize_element(root) != once {
                            failure_count += 1;
                            if failures.len() < 32 {
                                failures.push(format!("private idempotence: {label}"));
                            }
                        }
                        // Faithful mode promises complete source-property
                        // projections. Word mode deliberately transforms layout:
                        // M98b/file_167 parks MIX spacing on a following empty
                        // paragraph; M435/M88 retain original structural MIX
                        // properties and a deleted pilcrow. Those exact shapes
                        // have dedicated goldens, rather than a universal
                        // per-character paragraph-format equality contract.
                        for word_mode in [false, true] {
                            let old_input = dom.clone_subtree(old_source);
                            let new_input = dom.clone_subtree(new_source);
                            let old_body = dom.element(old_input, &W::body()).unwrap();
                            let new_body = dom.element(new_input, &W::body()).unwrap();
                            let mut settings = if word_mode {
                                WmlComparerSettings::default()
                            } else {
                                WmlComparerSettings::powertools_faithful()
                            };
                            settings.author_for_revisions = "Editor".to_string();
                            settings.date_time_for_revisions = DATE.to_string();
                            let redline = crate::comparer::compare_bodies_faithful(
                                &mut dom, old_input, new_input, old_body, new_body, &settings,
                            );
                            for (accept, expected) in
                                [(false, &clean_rejected), (true, &clean_accepted)]
                            {
                                let actual = projected(&mut dom, redline, accept);
                                let runs = |view: &SourceView| {
                                    view.formatted_characters
                                        .iter()
                                        .map(|(character, _, run)| (*character, run.clone()))
                                        .collect::<Vec<_>>()
                                };
                                let agrees = if word_mode {
                                    runs(&actual) == runs(expected)
                                        && actual.nontext_payloads == expected.nontext_payloads
                                        && actual.table_geometry == expected.table_geometry
                                } else {
                                    actual == *expected
                                };
                                if !agrees {
                                    failure_count += 1;
                                    if failures.len() < 32 {
                                        failures.push(format!("full caller source projection: word={word_mode}, accept={accept}; {label}; {}",compact_source_difference(&actual, expected)));
                                    }
                                }
                            }
                            if word_mode {
                                // Compare fresh copies of the identical clean
                                // sources: row/history mutation in the first
                                // call must not affect this repeat execution.
                                let old_repeat = dom.clone_subtree(old_source);
                                let new_repeat = dom.clone_subtree(new_source);
                                let old_body = dom.element(old_repeat, &W::body()).unwrap();
                                let new_body = dom.element(new_repeat, &W::body()).unwrap();
                                let repeated = crate::comparer::compare_bodies_faithful(
                                    &mut dom, old_repeat, new_repeat, old_body, new_body, &settings,
                                );
                                if dom.serialize_element(redline) != dom.serialize_element(repeated)
                                {
                                    failure_count += 1;
                                    if failures.len() < 32 {
                                        failures.push(format!("Word repeat execution: {label}"));
                                    }
                                }
                            }
                        }
                        cases += 1;
                    }
                }
            }
        }
    }
    assert_eq!(
        cases, 432,
        "all original boundary combinations remain exercised"
    );
    assert!(
        failure_count == 0,
        "{} source-contract failures across {cases} cases; first failures:\n{}",
        failure_count,
        failures
            .iter()
            .take(32)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn full_production_compare_restores_original_and_revised_run_and_paragraph_formatting() {
    for property in ["spacing", "indent", "run_font", "run_bold"] {
        let mut dom = Dom::new();
        let mut roots = Vec::new();
        for revised in [false, true] {
            let mut fragment = String::new();
            for i in 0..5 {
                let text = if revised && i == 2 {
                    "Shared completed report contains revised violet figures"
                } else {
                    "Shared completed report contains original quartz figures"
                };
                let before = if revised && property == "spacing" {
                    240
                } else {
                    120
                };
                let indent = if revised && property == "indent" {
                    720
                } else {
                    240
                };
                let font = if revised && property == "run_font" {
                    "Arial"
                } else {
                    "Carlito"
                };
                let bold = if revised && property == "run_bold" {
                    "<w:b/>"
                } else {
                    ""
                };
                fragment.push_str(&format!("<w:p><w:pPr><w:pStyle w:val=\"BodyText\"/><w:spacing w:before=\"{before}\" w:after=\"80\"/><w:ind w:left=\"{indent}\"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii=\"{font}\" w:hAnsi=\"{font}\"/>{bold}<w:color w:val=\"123456\"/></w:rPr><w:t>{text} {i}</w:t></w:r></w:p>"));
            }
            let doc = dom.parse_xdocument(&format!("<w:document xmlns:w=\"{}\"><w:body>{fragment}<w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr></w:body></w:document>", W::URI));
            roots.push(dom.root(doc).unwrap());
        }
        let expected_original = projected(&mut dom, roots[0], false);
        let expected_revised = projected(&mut dom, roots[1], true);
        let original_body = dom.element(roots[0], &W::body()).unwrap();
        let revised_body = dom.element(roots[1], &W::body()).unwrap();
        let redline = super::super::compare_bodies_faithful(
            &mut dom,
            roots[0],
            roots[1],
            original_body,
            revised_body,
            &WmlComparerSettings::default(),
        );
        assert_eq!(
            projected(&mut dom, redline, false),
            expected_original,
            "reject source format: {property}"
        );
        assert_eq!(
            projected(&mut dom, redline, true),
            expected_revised,
            "accept source format: {property}"
        );
    }
}

#[test]
fn public_cell_replacement_keeps_both_source_paragraph_properties_at_a_blank_tail() {
    for word_mode in [false, true] {
        for style in ["BodyText", "Heading1"] {
            let mut dom = Dom::new();
            let mut roots = Vec::new();
            for revised in [false, true] {
                let text = if revised {
                    "Revised violet harbor record0"
                } else {
                    "Original quartz meadow record0"
                };
                let before = if revised { 240 } else { 120 };
                let fragment = format!(
                    "<w:tbl><w:tblPr><w:tblW w:w='3600' w:type='dxa'/><w:tblLook w:val='04A0' w:firstRow='1' w:lastRow='0' w:firstColumn='1' w:lastColumn='0' w:noHBand='0' w:noVBand='1'/></w:tblPr><w:tblGrid><w:gridCol w:w='3600'/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w='3600' w:type='dxa'/></w:tcPr><w:p><w:pPr><w:pStyle w:val='{style}'/><w:spacing w:before='{before}' w:after='80'/></w:pPr><w:r><w:rPr><w:b/><w:color w:val='123456'/></w:rPr><w:t>{text}</w:t></w:r></w:p><w:p/></w:tc></w:tr></w:tbl>"
                );
                let anchors =
                    "<w:p><w:r><w:t>Stable surrounding contract anchor</w:t></w:r></w:p>".repeat(5);
                let doc = dom.parse_xdocument(&format!(
                        "<w:document xmlns:w='{}'><w:body>{anchors}{fragment}{anchors}<w:sectPr/></w:body></w:document>", W::URI
                    ));
                roots.push(dom.root(doc).unwrap());
            }
            let original = projected(&mut dom, roots[0], false);
            let revised = projected(&mut dom, roots[1], true);
            let bodies = roots
                .iter()
                .map(|&r| dom.element(r, &W::body()).unwrap())
                .collect::<Vec<_>>();
            let mut settings = if word_mode {
                WmlComparerSettings::default()
            } else {
                WmlComparerSettings::powertools_faithful()
            };
            settings.author_for_revisions = "Cell editor".to_string();
            settings.date_time_for_revisions = DATE.to_string();
            let redline = super::super::compare_bodies_faithful(
                &mut dom, roots[0], roots[1], bodies[0], bodies[1], &settings,
            );
            assert_eq!(
                projected(&mut dom, redline, false),
                original,
                "reject word={word_mode}, style={style}; redline XML: {}",
                dom.serialize_element(redline)
            );
            assert_eq!(
                projected(&mut dom, redline, true),
                revised,
                "accept word={word_mode}, style={style}; redline XML: {}",
                dom.serialize_element(redline)
            );
        }
    }
}

#[test]
fn live_blank_cell_tail_keeps_independent_replacement_paragraph_owners() {
    for inserted_first in [false, true] {
        for property in [
            "",
            "<w:spacing w:before='120' w:after='80'/>",
            "<w:pStyle w:val='Heading1'/>",
        ] {
            let deleted =
                revision_para("del", "Original quartz record", "ordinary", property, true);
            let inserted =
                revision_para("ins", "Revised violet record", "ordinary", property, true);
            let fragment = if inserted_first {
                inserted + &deleted
            } else {
                deleted + &inserted
            };
            let (mut dom, root, target) = document(&fragment, "tc");
            let original = projected(&mut dom, root, false);
            let revised = projected(&mut dom, root, true);
            let before = dom.serialize_element(root);
            merge_replaced_in_container(&mut dom, target, "Editor", true);
            assert_eq!(
                projected(&mut dom, root, false),
                original,
                "reject order={inserted_first}, props={property}"
            );
            assert_eq!(
                projected(&mut dom, root, true),
                revised,
                "accept order={inserted_first}, props={property}"
            );
            assert_eq!(
                dom.serialize_element(root),
                before,
                "independent cell paragraphs keep authored owners"
            );
        }
    }
}

#[test]
fn a_related_replacement_before_live_content_uses_a_paired_unmarked_pilcrow() {
    for properties in [
        "<w:pStyle w:val='BodyText'/>",
        "<w:spacing w:before='120' w:after='80'/>",
        "<w:pStyle w:val='Heading1'/><w:spacing w:before='120' w:after='80'/>",
    ] {
        let deleted = revision_para(
            "del",
            "Original quartz shared clause",
            "ordinary",
            properties,
            true,
        );
        let inserted = revision_para(
            "ins",
            "Revised violet shared clause",
            "ordinary",
            properties,
            true,
        )
        .replace("w:id=\"17\"", "w:id=\"18\"")
        .replace("w:id=\"31\"", "w:id=\"32\"");
        let anchor = "<w:p><w:r><w:t>Closing unchanged anchor</w:t></w:r></w:p>";
        let (mut dom, root, target) = document(&(deleted + &inserted + anchor), "body");
        let original = projected(&mut dom, root, false);
        let revised = projected(&mut dom, root, true);
        merge_replaced_in_container(&mut dom, target, "Editor", true);
        assert_eq!(
            projected(&mut dom, root, false),
            original,
            "reject {properties}"
        );
        assert_eq!(
            projected(&mut dom, root, true),
            revised,
            "accept {properties}"
        );
        let paras = dom.elements(target, Some(&W::p()));
        assert_eq!(paras.len(), 2, "Word keeps a mixed pair plus live anchor");
        assert!(para_is_mixed_revision(&dom, paras[0]));
        assert!(!para_mark_revision(&dom, paras[0], &W::ins()));
        assert!(!para_mark_revision(&dom, paras[0], &W::del()));
        assert_eq!(
            dom.descendants(root, Some(&W::p_pr_change())).len(),
            0,
            "equal authored properties need no invented history"
        );
    }
}

#[test]
fn multiple_original_paragraphs_before_a_live_blank_control_tail_keep_exact_phase_payloads() {
    for count in [2, 3, 8] {
        for inserted_first in [false, true] {
            for family in ["ordinary", "heading", "numbered"] {
                let first_props = properties(family, false);
                let mut deleted = String::new();
                for index in 0..count {
                    let other_props = format!(
                        "<w:pStyle w:val='BodyText'/><w:spacing w:after='{}'/>",
                        300 + index
                    );
                    deleted.push_str(
                        &revision_para(
                            "del",
                            &format!("Original clause {index}"),
                            "ordinary",
                            if index == 0 {
                                &first_props
                            } else {
                                &other_props
                            },
                            true,
                        )
                        .replace("w:id=\"17\"", &format!("w:id=\"{}\"", 100 + index))
                        .replace("w:id=\"31\"", &format!("w:id=\"{}\"", 200 + index)),
                    );
                }
                let inserted =
                    revision_para("ins", "Replacement clause", "ordinary", &first_props, true)
                        .replace("w:id=\"17\"", "w:id=\"901\"")
                        .replace("w:id=\"31\"", "w:id=\"902\"");
                let gap = if inserted_first {
                    inserted + &deleted
                } else {
                    deleted + &inserted
                };
                let (mut dom, root, target) = document(&(gap + "<w:p/>"), "sdtContent");
                let before = dom.elements(target, Some(&W::p()));
                let first_deleted_index = usize::from(inserted_first);
                let inserted_index = if inserted_first { 0 } else { count };
                let expected_first_properties = dom.clone_subtree(
                    dom.element(before[first_deleted_index], &W::p_pr())
                        .unwrap(),
                );
                let expected_first_mark =
                    dom.element(expected_first_properties, &W::r_pr()).unwrap();
                for deletion in dom.elements(expected_first_mark, Some(&W::del())) {
                    dom.remove(deletion);
                }
                if dom.nodes(expected_first_mark).is_empty() {
                    dom.remove(expected_first_mark);
                }
                let expected_first_properties = dom.serialize_element(expected_first_properties);
                let expected_inserted_body = dom
                    .elements(before[inserted_index], Some(&W::ins()))
                    .into_iter()
                    .map(|node| dom.serialize_element(node))
                    .collect::<Vec<_>>();
                let expected_deleted_bodies = before
                    [first_deleted_index..first_deleted_index + count]
                    .iter()
                    .map(|&paragraph| {
                        dom.elements(paragraph, Some(&W::del()))
                            .into_iter()
                            .map(|node| dom.serialize_element(node))
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>();
                let expected_later_properties = before
                    [first_deleted_index + 1..first_deleted_index + count]
                    .iter()
                    .map(|&paragraph| {
                        dom.serialize_element(dom.element(paragraph, &W::p_pr()).unwrap())
                    })
                    .collect::<Vec<_>>();
                merge_replaced_in_container(&mut dom, target, "Editor", true);
                let paragraphs = dom.elements(target, Some(&W::p()));
                assert_eq!(paragraphs.len(), count + 1);
                assert!(para_is_mixed_revision(&dom, paragraphs[0]));
                assert!(!para_mark_revision(&dom, paragraphs[0], &W::del()));
                assert_eq!(
                    dom.serialize_element(dom.element(paragraphs[0], &W::p_pr()).unwrap()),
                    expected_first_properties,
                    "matching authored paragraph geometry/style is unchanged"
                );
                for &other_original in &paragraphs[1..count] {
                    assert!(para_mark_revision(&dom, other_original, &W::del()));
                }
                assert!(paragraph_has_live_empty_pilcrow(&dom, paragraphs[count]));
                assert_eq!(
                    dom.elements(paragraphs[0], Some(&W::ins()))
                        .into_iter()
                        .map(|node| dom.serialize_element(node))
                        .collect::<Vec<_>>(),
                    expected_inserted_body
                );
                for index in 0..count {
                    assert_eq!(
                        dom.elements(paragraphs[index], Some(&W::del()))
                            .into_iter()
                            .map(|node| dom.serialize_element(node))
                            .collect::<Vec<_>>(),
                        expected_deleted_bodies[index]
                    );
                    if index > 0 {
                        assert_eq!(
                            dom.serialize_element(
                                dom.element(paragraphs[index], &W::p_pr()).unwrap()
                            ),
                            expected_later_properties[index - 1]
                        );
                    }
                }
                assert_eq!(
                    para_revision_body_text(&dom, paragraphs[0]),
                    "Replacement clause Original clause 0 "
                );
                assert!(dom.descendants(root, Some(&W::p_pr_change())).is_empty());
                let once = dom.serialize_element(root);
                merge_replaced_in_container(&mut dom, target, "Editor", true);
                assert_eq!(
                    dom.serialize_element(root),
                    once,
                    "phase is idempotent count={count}, inserted_first={inserted_first}, family={family}"
                );
            }
        }
    }
}

#[test]
fn multiple_revised_paragraphs_with_distinct_tail_properties_keep_word_deleted_carrier() {
    let props = properties("numbered", false);
    let other_props = "<w:pStyle w:val='Heading1'/><w:spacing w:after='990'/>";
    let first_inserted = revision_para("ins", "Leading revised clause", "ordinary", &props, true);
    let last_inserted = revision_para(
        "ins",
        "Distinct revised final clause",
        "ordinary",
        other_props,
        true,
    )
    .replace("w:id=\"17\"", "w:id=\"18\"")
    .replace("w:id=\"31\"", "w:id=\"32\"");
    let originals = (0..3)
        .map(|index| {
            revision_para(
                "del",
                &format!("Original clause {index}"),
                "ordinary",
                &props,
                true,
            )
            .replace("w:id=\"17\"", &format!("w:id=\"{}\"", 100 + index))
            .replace("w:id=\"31\"", &format!("w:id=\"{}\"", 200 + index))
        })
        .collect::<String>();
    let (mut dom, root, target) = document(
        &(first_inserted + &last_inserted + &originals + "<w:p/>"),
        "sdtContent",
    );
    let original = projected(&mut dom, root, false);
    let leading = dom.elements(target, Some(&W::p()))[0];
    let leading_properties = dom.serialize_element(dom.element(leading, &W::p_pr()).unwrap());
    merge_replaced_in_container(&mut dom, target, "Editor", true);
    assert_eq!(projected(&mut dom, root, false), original);
    let paragraphs = dom.elements(target, Some(&W::p()));
    assert_eq!(paragraphs.len(), 5);
    assert_eq!(
        dom.serialize_element(dom.element(paragraphs[0], &W::p_pr()).unwrap()),
        leading_properties
    );
    assert!(para_is_mixed_revision(&dom, paragraphs[1]));
    assert!(
        para_mark_revision(&dom, paragraphs[1], &W::del()),
        "unequal authored formats retain the existing Word carrier rule"
    );
    assert_eq!(
        para_revision_body_text(&dom, paragraphs[1])
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" "),
        "Distinct revised final clause Original clause 0"
    );
    for &other_original in &paragraphs[2..4] {
        assert!(para_mark_revision(&dom, other_original, &W::del()));
    }
    assert!(paragraph_has_live_empty_pilcrow(&dom, paragraphs[4]));
    assert!(dom.descendants(root, Some(&W::p_pr_change())).is_empty());
}

fn guarded_multi_original_complete_projection(dom: &mut Dom, root: NodeId, accept: bool) -> String {
    fn semantic(dom: &Dom, node: NodeId) -> String {
        let mut attrs = dom
            .attributes_iter(node)
            .filter(|(name, _)| !dom.is_namespace_declaration(name))
            .collect::<Vec<_>>();
        attrs.sort_by(|a, b| {
            (a.0.namespace_name(), a.0.local_name()).cmp(&(b.0.namespace_name(), b.0.local_name()))
        });
        let children = dom
            .nodes(node)
            .into_iter()
            .filter(|&child| {
                !((dom.name_is(child, &W::r_pr()) || dom.name_is(child, &W::p_pr()))
                    && !dom.has_attributes(child)
                    && dom.nodes(child).is_empty())
            })
            .map(|child| semantic(dom, child))
            .collect::<Vec<_>>();
        format!(
            "{:?}:{attrs:?}:{:?}:{}",
            dom.name(node),
            dom.text_value(node),
            children
                .into_iter()
                .map(|child| format!("{}:{child}", child.len()))
                .collect::<String>()
        )
    }
    let copy = dom.clone_subtree(root);
    let projected = if accept {
        crate::revision_processor::accept_revisions_document(dom, copy)
    } else {
        crate::revision_processor::reject_revisions_document(dom, copy)
    };
    remove_powertools_scratch_markup(dom, projected);
    semantic(dom, projected)
}

#[test]
fn multiple_original_blank_tail_checks_every_later_pilcrow_provenance() {
    let props = properties("numbered", false);
    for prior_index in [1, 2] {
        for (author, date) in [("Prior editor", DATE), ("Editor", "2000-01-01T00:00:00Z")] {
            let inserted = revision_para("ins", "Revised clause", "ordinary", &props, true);
            let originals = (0..3)
                .map(|index| {
                    let paragraph = revision_para(
                        "del",
                        &format!("Original clause {index}"),
                        "ordinary",
                        &props,
                        true,
                    )
                    .replace("w:id=\"17\"", &format!("w:id=\"{}\"", 100 + index))
                    .replace("w:id=\"31\"", &format!("w:id=\"{}\"", 200 + index));
                    if index == prior_index {
                        paragraph.replace(
                            &format!(
                                "<w:del w:id=\"{}\" w:author=\"Editor\" w:date=\"{DATE}\"/>",
                                200 + index
                            ),
                            &format!(
                                "<w:del w:id=\"{}\" w:author=\"{author}\" w:date=\"{date}\"/>",
                                200 + index
                            ),
                        )
                    } else {
                        paragraph
                    }
                })
                .collect::<String>();
            let (mut dom, root, target) =
                document(&(inserted + &originals + "<w:p/>"), "sdtContent");
            let original = projected(&mut dom, root, false);
            let before = dom.elements(target, Some(&W::p()));
            let prior_ppr = dom.element(before[1 + prior_index], &W::p_pr()).unwrap();
            let prior_mark = dom
                .element(dom.element(prior_ppr, &W::r_pr()).unwrap(), &W::del())
                .unwrap();
            assert_eq!(dom.attribute(prior_mark, &W::author()), Some(author));
            assert_eq!(dom.attribute(prior_mark, &W::date()), Some(date));
            let prior_body = dom.element(before[1 + prior_index], &W::del()).unwrap();
            assert_eq!(dom.attribute(prior_body, &W::author()), Some("Editor"));
            assert_eq!(dom.attribute(prior_body, &W::date()), Some(DATE));
            let intact = dom.serialize_element(root);
            assert!(
                !story_tail_has_paired_original_boundary(
                    &mut dom,
                    &before[1..4],
                    before[0],
                    &before[4..],
                    "Editor"
                ),
                "every later original mark must independently match author/date"
            );
            assert_eq!(
                dom.serialize_element(root),
                intact,
                "eligibility cannot mutate source payload or ownership"
            );
            let original_bodies = before[1..4]
                .iter()
                .map(|&paragraph| {
                    dom.elements(paragraph, Some(&W::del()))
                        .into_iter()
                        .map(|node| dom.serialize_element(node))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            let revised_body = dom
                .elements(before[0], Some(&W::ins()))
                .into_iter()
                .map(|node| dom.serialize_element(node))
                .collect::<Vec<_>>();
            let original_properties = before[1..4]
                .iter()
                .map(|&p| dom.serialize_element(dom.element(p, &W::p_pr()).unwrap()))
                .collect::<Vec<_>>();
            merge_replaced_in_container(&mut dom, target, "Editor", true);
            let paragraphs = dom.elements(target, Some(&W::p()));
            assert_eq!(
                paragraphs.len(),
                4,
                "prior index={prior_index}, author={author}, date={date}"
            );
            assert!(para_is_mixed_revision(&dom, paragraphs[0]));
            assert_eq!(
                para_revision_body_text(&dom, paragraphs[0]),
                "Revised clause Original clause 0 "
            );
            for index in 0..3 {
                let ppr = dom.element(paragraphs[index], &W::p_pr()).unwrap();
                assert_eq!(
                    dom.serialize_element(ppr),
                    original_properties[index],
                    "all original pilcrow properties and stamps survive"
                );
                assert!(para_mark_revision(&dom, paragraphs[index], &W::del()));
                assert_eq!(
                    dom.elements(paragraphs[index], Some(&W::del()))
                        .into_iter()
                        .map(|node| dom.serialize_element(node))
                        .collect::<Vec<_>>(),
                    original_bodies[index]
                );
                if index > 0 {
                    assert_eq!(
                        para_revision_body_text(&dom, paragraphs[index]),
                        format!("Original clause {index} ")
                    );
                }
            }
            assert!(paragraph_has_live_empty_pilcrow(&dom, paragraphs[3]));
            assert_eq!(projected(&mut dom, root, false), original);
            assert_eq!(
                dom.elements(paragraphs[0], Some(&W::ins()))
                    .into_iter()
                    .map(|node| dom.serialize_element(node))
                    .collect::<Vec<_>>(),
                revised_body
            );
            assert!(dom.descendants(root, Some(&W::p_pr_change())).is_empty());
            let once = dom.serialize_element(root);
            merge_replaced_in_container(&mut dom, target, "Editor", true);
            assert_eq!(
                dom.serialize_element(root),
                once,
                "foreign-history phase remains idempotent"
            );
        }
    }
}

#[test]
fn multiple_original_blank_tail_preserves_first_original_section_and_property_history() {
    let props = properties("numbered", false);
    for protected in [
        "<w:sectPr><w:type w:val='continuous'/><w:pgSz w:w='12240' w:h='15840'/></w:sectPr>",
        "<w:pPrChange w:id='701' w:author='Prior editor' w:date='2000-01-01T00:00:00Z'><w:pPr><w:pStyle w:val='Heading1'/><w:spacing w:after='990'/></w:pPr></w:pPrChange>",
        "<w:rPrChange w:id='702' w:author='Prior editor' w:date='2000-01-01T00:00:00Z'><w:rPr><w:b/><w:color w:val='CC3300'/></w:rPr></w:rPrChange>",
    ] {
        let decorate = |paragraph: String| {
            if protected.starts_with("<w:rPrChange") {
                paragraph.replacen(
                    "</w:rPr></w:pPr>",
                    &format!("{protected}</w:rPr></w:pPr>"),
                    1,
                )
            } else {
                paragraph.replacen("</w:pPr>", &format!("{protected}</w:pPr>"), 1)
            }
        };
        // Identical tracked properties still must not authorize deleting
        // an independent section boundary or previously authored history.
        let inserted = decorate(revision_para(
            "ins",
            "Revised clause",
            "ordinary",
            &props,
            true,
        ));
        let first = decorate(revision_para(
            "del",
            "Original first clause",
            "ordinary",
            &props,
            true,
        ));
        let last = revision_para("del", "Original last clause", "ordinary", &props, true)
            .replace("w:id=\"17\"", "w:id=\"18\"")
            .replace("w:id=\"31\"", "w:id=\"32\"");
        let (mut dom, root, target) =
            document(&(inserted + &first + &last + "<w:p/>"), "sdtContent");
        let original = projected(&mut dom, root, false);
        let before = dom.elements(target, Some(&W::p()));
        let first_properties = dom.serialize_element(dom.element(before[1], &W::p_pr()).unwrap());
        let last_properties = dom.serialize_element(dom.element(before[2], &W::p_pr()).unwrap());
        let intact = dom.serialize_element(root);
        assert!(!paired_replacement_has_identical_properties(
            &mut dom, before[1], before[0]
        ));
        assert!(
            !story_tail_has_paired_original_boundary(
                &mut dom,
                &before[1..3],
                before[0],
                &before[3..],
                "Editor"
            ),
            "section or history never authorizes paired boundary normalization"
        );
        assert_eq!(
            dom.serialize_element(root),
            intact,
            "negative eligibility retains complete text, stamps, section/history and geometry"
        );
        if protected.starts_with("<w:sectPr") {
            // The section disqualifies the new story-tail normalization,
            // but the established general Word I/D fold still joins into
            // the inserted carrier and strips its mark shell (M361).
            let expected_properties =
                dom.clone_subtree(dom.element(before[0], &W::p_pr()).unwrap());
            let mark_properties = dom.element(expected_properties, &W::r_pr()).unwrap();
            dom.remove(mark_properties);
            let expected_properties = dom.serialize_element(expected_properties);
            let inserted_body = dom.serialize_element(dom.element(before[0], &W::ins()).unwrap());
            let first_deleted_body =
                dom.serialize_element(dom.element(before[1], &W::del()).unwrap());
            let last_deleted_body =
                dom.serialize_element(dom.element(before[2], &W::del()).unwrap());
            let original_section = dom.serialize_element(
                dom.element(dom.element(before[1], &W::p_pr()).unwrap(), &W::sect_pr())
                    .unwrap(),
            );
            let body = dom.element(root, &W::body()).unwrap();
            let final_section = dom.serialize_element(dom.element(body, &W::sect_pr()).unwrap());
            merge_replaced_in_container(&mut dom, target, "Editor", true);
            let paragraphs = dom.elements(target, Some(&W::p()));
            assert_eq!(
                paragraphs.len(),
                3,
                "general section fold produces MIX,D,blank"
            );
            assert!(para_is_mixed_revision(&dom, paragraphs[0]));
            assert!(!para_mark_revision(&dom, paragraphs[0], &W::ins()));
            assert!(!para_mark_revision(&dom, paragraphs[0], &W::del()));
            let mixed_properties = dom.element(paragraphs[0], &W::p_pr()).unwrap();
            assert_eq!(
                dom.serialize_element(mixed_properties),
                expected_properties,
                "inserted carrier keeps complete authored section geometry and paragraph formatting"
            );
            assert_eq!(
                dom.serialize_element(dom.element(mixed_properties, &W::sect_pr()).unwrap()),
                original_section
            );
            assert_eq!(
                dom.descendants(target, Some(&W::sect_pr())).len(),
                1,
                "one authored paragraph section owner survives the general fold"
            );
            assert_eq!(
                dom.serialize_element(dom.element(body, &W::sect_pr()).unwrap()),
                final_section,
                "final body section retains independent ownership"
            );
            assert_eq!(
                dom.serialize_element(dom.element(paragraphs[0], &W::ins()).unwrap()),
                inserted_body
            );
            assert_eq!(
                dom.serialize_element(dom.element(paragraphs[0], &W::del()).unwrap()),
                first_deleted_body
            );
            assert_eq!(
                para_revision_body_text(&dom, paragraphs[0]),
                "Revised clause Original first clause "
            );
            assert!(para_mark_revision(&dom, paragraphs[1], &W::del()));
            assert_eq!(
                dom.serialize_element(dom.element(paragraphs[1], &W::p_pr()).unwrap()),
                last_properties
            );
            assert_eq!(
                dom.serialize_element(dom.element(paragraphs[1], &W::del()).unwrap()),
                last_deleted_body
            );
            assert_eq!(
                para_revision_body_text(&dom, paragraphs[1]),
                "Original last clause "
            );
            assert!(paragraph_has_live_empty_pilcrow(&dom, paragraphs[2]));
            assert_eq!(
                projected(&mut dom, root, false),
                original,
                "general section fold preserves original text and full paragraph/run formatting"
            );
            let once = dom.serialize_element(root);
            merge_replaced_in_container(&mut dom, target, "Editor", true);
            assert_eq!(
                dom.serialize_element(root),
                once,
                "general section fold remains idempotent"
            );
            continue;
        }
        let original_bodies = before[1..3]
            .iter()
            .map(|&paragraph| {
                dom.elements(paragraph, Some(&W::del()))
                    .into_iter()
                    .map(|node| dom.serialize_element(node))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let revised_body = dom
            .elements(before[0], Some(&W::ins()))
            .into_iter()
            .map(|node| dom.serialize_element(node))
            .collect::<Vec<_>>();
        merge_replaced_in_container(&mut dom, target, "Editor", true);
        let originals = dom
            .elements(target, Some(&W::p()))
            .into_iter()
            .filter(|&p| para_mark_revision(&dom, p, &W::del()))
            .collect::<Vec<_>>();
        assert_eq!(originals.len(), 2, "protected={protected}");
        assert_eq!(
            dom.serialize_element(dom.element(originals[0], &W::p_pr()).unwrap()),
            first_properties
        );
        assert_eq!(
            dom.serialize_element(dom.element(originals[1], &W::p_pr()).unwrap()),
            last_properties
        );
        assert_eq!(
            para_revision_body_text(&dom, originals[1]),
            "Original last clause "
        );
        assert_eq!(
            projected(&mut dom, root, false),
            original,
            "reject protected={protected}"
        );
        for index in 0..2 {
            assert_eq!(
                dom.elements(originals[index], Some(&W::del()))
                    .into_iter()
                    .map(|node| dom.serialize_element(node))
                    .collect::<Vec<_>>(),
                original_bodies[index]
            );
        }
        assert_eq!(
            dom.elements(originals[0], Some(&W::ins()))
                .into_iter()
                .map(|node| dom.serialize_element(node))
                .collect::<Vec<_>>(),
            revised_body
        );
        let once = dom.serialize_element(root);
        merge_replaced_in_container(&mut dom, target, "Editor", true);
        assert_eq!(
            dom.serialize_element(root),
            once,
            "history-preserving phase remains idempotent"
        );
    }
}

#[test]
fn multiple_revised_blank_tail_pairs_only_the_matching_final_revised_paragraph() {
    let matching = properties("numbered", false);
    for leading_count in [1, 2, 4] {
        let leading = (0..leading_count).map(|index| {
                let props = format!("<w:pStyle w:val='Heading1'/><w:spacing w:before='{}' w:after='990'/><w:ind w:left='720'/><w:jc w:val='right'/>", 240 + index);
                revision_para("ins", &format!("Leading revised clause {index}"), "ordinary", &props, true)
                    .replace("w:id=\"17\"", &format!("w:id=\"{}\"", 400 + index))
                    .replace("w:id=\"31\"", &format!("w:id=\"{}\"", 500 + index))
            }).collect::<String>();
        let final_inserted =
            revision_para("ins", "Final revised clause", "ordinary", &matching, true)
                .replace("w:id=\"17\"", "w:id=\"601\"")
                .replace("w:id=\"31\"", "w:id=\"602\"");
        let deleted = (0..3)
            .map(|index| {
                let later = format!(
                    "<w:pStyle w:val='BodyText'/><w:spacing w:after='{}'/><w:ind w:left='360'/>",
                    300 + index
                );
                revision_para(
                    "del",
                    &format!("Original clause {index}"),
                    "ordinary",
                    if index == 0 { &matching } else { &later },
                    true,
                )
                .replace("w:id=\"17\"", &format!("w:id=\"{}\"", 100 + index))
                .replace("w:id=\"31\"", &format!("w:id=\"{}\"", 200 + index))
            })
            .collect::<String>();
        let (mut dom, root, target) = document(
            &(leading + &final_inserted + &deleted + "<w:p/>"),
            "sdtContent",
        );
        let before = dom.elements(target, Some(&W::p()));
        let leading_xml = before[..leading_count]
            .iter()
            .map(|&p| dom.serialize_element(p))
            .collect::<Vec<_>>();
        let later_properties = before[leading_count + 2..leading_count + 4]
            .iter()
            .map(|&p| dom.serialize_element(dom.element(p, &W::p_pr()).unwrap()))
            .collect::<Vec<_>>();
        merge_replaced_in_container(&mut dom, target, "Editor", true);
        let paragraphs = dom.elements(target, Some(&W::p()));
        assert_eq!(paragraphs.len(), leading_count + 4);
        for index in 0..leading_count {
            assert_eq!(
                dom.serialize_element(paragraphs[index]),
                leading_xml[index],
                "leading revised paragraphs retain complete properties, payload and stamps"
            );
            assert!(para_mark_revision(&dom, paragraphs[index], &W::ins()));
        }
        assert!(para_is_mixed_revision(&dom, paragraphs[leading_count]));
        assert!(!para_mark_revision(
            &dom,
            paragraphs[leading_count],
            &W::del()
        ));
        assert!(!para_mark_revision(
            &dom,
            paragraphs[leading_count],
            &W::ins()
        ));
        assert_eq!(
            para_revision_body_text(&dom, paragraphs[leading_count]),
            "Final revised clause Original clause 0 "
        );
        for index in 0..2 {
            let p = paragraphs[leading_count + 1 + index];
            assert!(para_mark_revision(&dom, p, &W::del()));
            assert_eq!(
                dom.serialize_element(dom.element(p, &W::p_pr()).unwrap()),
                later_properties[index]
            );
            assert_eq!(
                para_revision_body_text(&dom, p),
                format!("Original clause {} ", index + 1)
            );
        }
        assert!(paragraph_has_live_empty_pilcrow(
            &dom,
            *paragraphs.last().unwrap()
        ));
        assert!(dom.descendants(root, Some(&W::p_pr_change())).is_empty());
        let once = dom.serialize_element(root);
        merge_replaced_in_container(&mut dom, target, "Editor", true);
        assert_eq!(
            dom.serialize_element(root),
            once,
            "multiple-I phase is idempotent count={leading_count}"
        );
    }
}

// A raw D/I phase fixture is not an untracked document: its deleted
// paragraph marks can make revision projection promote the shared blank
// out of sdtContent. Public source conservation therefore starts from
// literal clean source packages, with the blank authored inside the SDT.
#[test]
fn public_compare_multi_original_blank_tail_recovers_complete_clean_sources() {
    fn clean_paragraph(text: &str, props: &str) -> String {
        format!(
            "<w:p><w:pPr>{props}</w:pPr><w:r><w:rPr><w:b/><w:color w:val='123456'/><w:lang w:val='en-US'/></w:rPr><w:t>{text}</w:t></w:r></w:p>"
        )
    }
    fn package(body: &str, container: &str) -> Vec<u8> {
        let mut package = crate::opc::PartFs::open(include_bytes!(
            "../../../../../../tests/fixtures/relids/image_doc.docx"
        ))
        .unwrap();
        // Every referenced style is defined and has distinct inherited
        // formatting. Unresolved pStyle references are intentionally
        // removed by the public comparer and are not a faithful fixture.
        package.set_part("word/styles.xml", format!(
                "<w:styles xmlns:w='{}'><w:style w:type='paragraph' w:default='1' w:styleId='Normal'><w:name w:val='Normal'/></w:style><w:style w:type='paragraph' w:styleId='BodyText'><w:name w:val='Body Text'/><w:pPr><w:ind w:left='120'/><w:jc w:val='left'/></w:pPr><w:rPr><w:color w:val='334455'/></w:rPr></w:style><w:style w:type='paragraph' w:styleId='Heading1'><w:name w:val='heading 1'/><w:pPr><w:keepNext/><w:outlineLvl w:val='0'/></w:pPr><w:rPr><w:sz w:val='28'/></w:rPr></w:style></w:styles>", W::URI
            ).into_bytes());
        package.set_part("word/numbering.xml", format!(
                "<w:numbering xmlns:w='{}'><w:abstractNum w:abstractNumId='1'><w:multiLevelType w:val='singleLevel'/><w:lvl w:ilvl='0'><w:start w:val='1'/><w:numFmt w:val='decimal'/><w:lvlText w:val='%1.'/><w:lvlJc w:val='left'/><w:pPr><w:tabs><w:tab w:val='num' w:pos='720'/></w:tabs><w:ind w:left='720' w:hanging='360'/></w:pPr></w:lvl></w:abstractNum><w:num w:numId='9'><w:abstractNumId w:val='1'/></w:num></w:numbering>", W::URI
            ).into_bytes());
        package.add_content_type_override(
            "/word/numbering.xml",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml",
        );
        package.add_document_relationship(
            "word/document.xml",
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering",
            "numbering.xml",
        );
        let (dom, root, _) = document(body, container);
        package.set_part(
            "word/document.xml",
            dom.serialize_element(root).into_bytes(),
        );
        package.to_zip().unwrap()
    }
    fn complete_source(bytes: &[u8]) -> String {
        let package = crate::opc::PartFs::open(bytes).unwrap();
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&package.part_string("word/document.xml").unwrap());
        let root = dom.root(doc).unwrap();
        // IgnorePt14Namespace adds this compatibility declaration even
        // when no scratch markup remains. It has no document semantics;
        // retain every other compatibility token and every real attribute.
        assert!(
            dom.descendants_and_self(root, None)
                .into_iter()
                .all(|node| {
                    dom.name(node)
                        .is_none_or(|name| name.namespace_name() != PT::URI)
                        && dom
                            .attributes_iter(node)
                            .into_iter()
                            .all(|(name, _)| name.namespace_name() != PT::URI)
                }),
            "complete public projections contain no PowerTools scratch payload"
        );
        let ignorable = MC::ignorable();
        if dom.attribute(root, &ignorable) == Some("pt14") {
            dom.set_attribute_value(root, &ignorable, None);
        }
        guarded_multi_original_complete_projection(&mut dom, root, false)
    }
    for count in [2, 3, 8] {
        for leading_count in [0, 1, 2, 4] {
            for family in ["ordinary", "heading", "numbered"] {
                let matching = properties(family, false);
                // Disjoint alphabets give every changed text node an
                // unambiguous source owner rather than Equal ancestry.
                let original = (0..count).map(|index| {
                        let later = format!("<w:pStyle w:val='BodyText'/><w:spacing w:after='{}'/><w:ind w:left='360'/>", 300 + index);
                        let text = std::iter::repeat_n(char::from_u32(0x03b1 + index as u32).unwrap(), 8).collect::<String>();
                        clean_paragraph(&text, if index == 0 { &matching } else { &later })
                    }).collect::<String>() + "<w:p/>";
                let leading = (0..leading_count).map(|index| {
                        let props = format!("<w:pStyle w:val='Heading1'/><w:spacing w:before='{}' w:after='990'/><w:ind w:left='720'/><w:jc w:val='right'/>", 240 + index);
                        let text = std::iter::repeat_n(char::from_u32(0x4e00 + index as u32).unwrap(), 8).collect::<String>();
                        clean_paragraph(&text, &props)
                    }).collect::<String>();
                let revised = leading + &clean_paragraph("漢字漢字漢字漢字", &matching) + "<w:p/>";
                for (mode, settings, container) in [
                    // The faithful caller guarantees SDT ownership.
                    (
                        "PowerTools SDT",
                        WmlComparerSettings::powertools_faithful(),
                        "sdtContent",
                    ),
                    // Word deliberately flattens some completely revised
                    // controls (M390); its ordinary story has the same
                    // paragraph boundary and property contract.
                    ("Word story", WmlComparerSettings::default(), "body"),
                ] {
                    let a = package(&original, container);
                    let b = package(&revised, container);
                    let expected_original = complete_source(&a);
                    let expected_revised = complete_source(&b);
                    let compared = crate::document_comparer::compare_documents_with_settings(
                        &a, &b, &settings,
                    )
                    .unwrap();
                    let rejected = crate::document_comparer::reject_revisions(&compared).unwrap();
                    let accepted = crate::document_comparer::accept_revisions(&compared).unwrap();
                    assert_eq!(
                        complete_source(&rejected),
                        expected_original,
                        "reject {mode} count={count}, leading={leading_count}, family={family}"
                    );
                    assert_eq!(
                        complete_source(&accepted),
                        expected_revised,
                        "accept {mode} count={count}, leading={leading_count}, family={family}"
                    );
                }
            }
        }
    }
}

#[test]
fn a_single_replacement_before_a_live_blank_control_tail_keeps_a_paired_pilcrow() {
    for inserted_first in [false, true] {
        let props = properties("numbered", false);
        let deleted = revision_para("del", "Level 1", "ordinary", &props, true);
        let inserted = revision_para("ins", "test", "ordinary", &props, true)
            .replace("w:id=\"17\"", "w:id=\"18\"")
            .replace("w:id=\"31\"", "w:id=\"32\"");
        let gap = if inserted_first {
            inserted + &deleted
        } else {
            deleted + &inserted
        };
        let (mut dom, root, target) = document(&(gap + "<w:p/>"), "sdtContent");
        let original = projected(&mut dom, root, false);
        let revised = projected(&mut dom, root, true);
        merge_replaced_in_container(&mut dom, target, "Editor", true);
        assert_eq!(
            projected(&mut dom, root, false),
            original,
            "reject control, order={inserted_first}"
        );
        assert_eq!(
            projected(&mut dom, root, true),
            revised,
            "accept control, order={inserted_first}"
        );
        let paragraphs = dom.elements(target, Some(&W::p()));
        assert_eq!(paragraphs.len(), 2);
        assert!(para_is_mixed_revision(&dom, paragraphs[0]));
        assert!(!para_mark_revision(&dom, paragraphs[0], &W::ins()));
        assert!(!para_mark_revision(&dom, paragraphs[0], &W::del()));
    }
}

#[test]
fn a_sole_original_and_multiple_revised_paragraphs_keep_live_blank_tail_ownership() {
    fn count_projected_paragraphs(dom: &mut Dom, root: NodeId, accept: bool) -> usize {
        let copy = dom.clone_subtree(root);
        let projected = if accept {
            crate::revision_processor::accept_revisions_document(dom, copy)
        } else {
            crate::revision_processor::reject_revisions_document(dom, copy)
        };
        dom.descendants(projected, Some(&W::p())).len()
    }
    for count in [2, 3] {
        for inserted_first in [false, true] {
            for closing_props in [
                "<w:pStyle w:val='BodyText'/>",
                "<w:spacing w:before='120' w:after='80'/>",
                "<w:pStyle w:val='Heading1'/><w:spacing w:after='80'/>",
            ] {
                let deleted = revision_para(
                    "del",
                    "Original quartz contractual clause",
                    "ordinary",
                    closing_props,
                    true,
                );
                let inserted = (0..count)
                    .map(|i| {
                        let props = if i + 1 == count {
                            closing_props
                        } else {
                            "<w:spacing w:before='360' w:after='240'/>"
                        };
                        revision_para(
                            "ins",
                            &format!("Revised violet contractual clause {i}"),
                            "ordinary",
                            props,
                            true,
                        )
                        .replace("w:id=\"17\"", &format!("w:id=\"{}\"", 120 + i))
                        .replace("w:id=\"31\"", &format!("w:id=\"{}\"", 200 + i))
                    })
                    .collect::<String>();
                let gap = if inserted_first {
                    inserted + &deleted
                } else {
                    deleted + &inserted
                };
                let (mut dom, root, target) = document(&(gap + "<w:p/>"), "sdtContent");
                let original = projected(&mut dom, root, false);
                let revised = projected(&mut dom, root, true);
                assert_eq!(count_projected_paragraphs(&mut dom, root, false), 2);
                assert_eq!(count_projected_paragraphs(&mut dom, root, true), count + 1);
                merge_replaced_in_container(&mut dom, target, "Editor", true);
                assert_eq!(
                    projected(&mut dom, root, false),
                    original,
                    "reject count={count}, order={inserted_first}, props={closing_props}"
                );
                assert_eq!(
                    projected(&mut dom, root, true),
                    revised,
                    "accept count={count}, order={inserted_first}, props={closing_props}"
                );
                assert_eq!(
                    count_projected_paragraphs(&mut dom, root, false),
                    2,
                    "original paragraph and independent blank survive"
                );
                assert_eq!(
                    count_projected_paragraphs(&mut dom, root, true),
                    count + 1,
                    "every revised paragraph and independent blank survive"
                );
                let mixed = dom
                    .elements(target, Some(&W::p()))
                    .into_iter()
                    .filter(|&p| para_is_mixed_revision(&dom, p))
                    .collect::<Vec<_>>();
                assert_eq!(mixed.len(), 1, "Word terminal fusion shape remains");
                assert!(!para_mark_revision(&dom, mixed[0], &W::del()));
                let once = dom.serialize_element(root);
                merge_replaced_in_container(&mut dom, target, "Editor", true);
                assert_eq!(dom.serialize_element(root), once, "stable paired owner");
            }
        }
    }
}

#[test]
fn live_blank_cell_tail_with_inert_run_keeps_exact_pilcrow_owners() {
    for tail in [
        "<w:p><w:r/></w:p>",
        "<w:p><w:r><w:rPr><w:b/></w:rPr></w:r></w:p>",
        "<w:p><w:r><w:t/></w:r></w:p>",
    ] {
        for inserted_first in [false, true] {
            let props = "<w:pStyle w:val='Heading1'/><w:spacing w:before='120' w:after='80'/>";
            let deleted = revision_para("del", "Original quartz record", "ordinary", props, true);
            let inserted = revision_para(
                "ins",
                "Revised violet record",
                "ordinary",
                &props.replace("before='120'", "before='240'"),
                true,
            );
            let gap = if inserted_first {
                inserted + &deleted
            } else {
                deleted + &inserted
            };
            let (mut dom, root, target) = document(&(gap + tail), "tc");
            let original = projected(&mut dom, root, false);
            let revised = projected(&mut dom, root, true);
            let before = dom.serialize_element(root);
            merge_replaced_in_container(&mut dom, target, "Editor", true);
            assert_eq!(
                projected(&mut dom, root, false),
                original,
                "reject tail={tail}, order={inserted_first}; XML={}",
                dom.serialize_element(root)
            );
            assert_eq!(
                projected(&mut dom, root, true),
                revised,
                "accept tail={tail}, order={inserted_first}; XML={}",
                dom.serialize_element(root)
            );
            assert_eq!(
                dom.serialize_element(root),
                before,
                "an inert run does not erase the cell's live terminal owner: {tail}"
            );
        }
    }
}

#[test]
fn paired_property_comparison_never_grows_the_persistent_dom_or_changes_sources() {
    let mut dom = Dom::new();
    let mut pairs = Vec::new();
    for case in 0..32 {
        let same = case % 2 == 0;
        let old_space = 120 + case;
        let new_space = if same { old_space } else { old_space + 60 };
        let doc = dom.parse_xdocument(&format!(
                "<w:document xmlns:w='{}' xmlns:pt='{}'><w:body><w:p><w:pPr pt:Unid='only-original-bookkeeping'><w:pStyle w:val='BodyText'/><w:spacing w:before='{old_space}' w:after='80'/><w:rPr><w:b/><w:color w:val='123456'/><w:del w:id='1' w:author='Source editor' w:date='2026-01-02T03:04:05Z'/></w:rPr></w:pPr><w:del w:id='2'><w:r><w:delText>Original owned clause</w:delText></w:r></w:del></w:p><w:p><w:pPr><w:pStyle w:val='BodyText'/><w:spacing w:before='{new_space}' w:after='80'/><w:rPr><w:b/><w:color w:val='123456'/><w:ins w:id='3' w:author='Source editor' w:date='2026-01-02T03:04:05Z'/></w:rPr></w:pPr><w:ins w:id='4'><w:r><w:t>Revised owned clause</w:t></w:r></w:ins></w:p></w:body></w:document>", W::URI, PT::URI
            ));
        let root = dom.root(doc).unwrap();
        let paragraphs = dom.descendants(root, Some(&W::p()));
        pairs.push((root, paragraphs[0], paragraphs[1], same));
    }
    let sources = pairs
        .iter()
        .map(|&(root, _, _, _)| dom.serialize_element(root))
        .collect::<Vec<_>>();
    dom.shrink_arena_to_fit();
    let nodes = dom.node_count();
    let capacity = dom.node_capacity();
    for _ in 0..25 {
        for &(_, old, new, same) in &pairs {
            assert_eq!(
                paired_replacement_has_identical_properties(&mut dom, old, new),
                same,
                "complete authored spacing decides pairing"
            );
        }
    }
    assert_eq!(
        dom.node_count(),
        nodes,
        "800 pure property probes retain no scratch nodes"
    );
    assert_eq!(
        dom.node_capacity(),
        capacity,
        "scratch never reallocates the persistent source arena"
    );
    for ((root, _, _, _), source) in pairs.into_iter().zip(sources) {
        assert_eq!(
            dom.serialize_element(root),
            source,
            "source text, complete mark formatting and provenance stay immutable"
        );
    }
}

#[test]
fn empty_pilcrow_proof_retains_inert_formatting_and_declines_owned_payloads() {
    for (fragment, expected) in [
        ("<w:p/>", true),
        ("<w:p><w:r/></w:p>", true),
        (
            "<w:p><w:r><w:rPr><w:b/><w:color w:val='123456'/></w:rPr><w:t/></w:r></w:p>",
            true,
        ),
        (
            "<w:p><w:r><w:t xml:space='preserve'> </w:t></w:r></w:p>",
            false,
        ),
        ("<w:p><w:r><w:tab/></w:r></w:p>", false),
        ("<w:p><w:r><w:br/></w:r></w:p>", false),
        (
            "<w:p><w:r><w:fldChar w:fldCharType='begin'/></w:r></w:p>",
            false,
        ),
        (
            "<w:p><w:r><w:instrText>PAGE</w:instrText></w:r></w:p>",
            false,
        ),
        ("<w:p><w:r><w:drawing/></w:r></w:p>", false),
        ("<w:p><w:r><w:pict/></w:r></w:p>", false),
        ("<w:p><m:oMath/></w:p>", false),
        (
            "<w:p><w:r><w:rPr><w:rPrChange w:id='7'><w:rPr/></w:rPrChange></w:rPr></w:r></w:p>",
            false,
        ),
        (
            "<w:p><w:r><w:rPr><w:ins w:id='7'/></w:rPr></w:r></w:p>",
            false,
        ),
        (
            "<w:p><w:pPr><w:rPr><w:del w:id='7'/></w:rPr></w:pPr></w:p>",
            false,
        ),
    ] {
        let (dom, root, target) = document(fragment, "body");
        let p = dom.elements(target, Some(&W::p()))[0];
        let before = dom.serialize_element(root);
        assert_eq!(
            paragraph_has_live_empty_pilcrow(&dom, p),
            expected,
            "{fragment}"
        );
        assert_eq!(
            dom.serialize_element(root),
            before,
            "proof must retain exact source formatting and payloads"
        );
    }
}

#[test]
fn public_two_original_one_revised_windows_keep_complete_sources_after_all_word_phases() {
    for (container, tail) in [("body", "live"), ("sdtContent", "blank")] {
        for family in [
            "ordinary", "heading", "numbered", "field", "uniform", "long",
        ] {
            for anchored in [false, true] {
                for word_mode in [false, true] {
                    let mut dom = Dom::new();
                    let props = properties(
                        if family == "heading" {
                            "heading"
                        } else if family == "numbered" || family == "uniform" {
                            "numbered"
                        } else {
                            "ordinary"
                        },
                        false,
                    );
                    let mut roots = Vec::new();
                    for revised in [false, true] {
                        let kind = if revised { "ins" } else { "del" };
                        let count = if revised { 1 } else { 2 };
                        let mut gap = (0..count)
                            .map(|i| {
                                revision_para(
                                    kind,
                                    &text(family, kind, i, false),
                                    family,
                                    &props,
                                    true,
                                )
                                .replace("w:id=\"17\"", &format!("w:id=\"{}\"", 120 + i))
                                .replace("w:id=\"31\"", &format!("w:id=\"{}\"", 200 + i))
                            })
                            .collect::<String>();
                        gap.push_str(if tail == "live" {
                            "<w:p><w:r><w:t>Closing unchanged anchor</w:t></w:r></w:p>"
                        } else {
                            "<w:p/>"
                        });
                        let content = if container == "sdtContent" {
                            format!(
                                "<w:sdt><w:sdtPr><w:alias w:val='Contract clause'/><w:tag w:val='authored-owner'/><w:id w:val='42'/></w:sdtPr><w:sdtEndPr><w:color w:val='234567'/></w:sdtEndPr><w:sdtContent>{gap}</w:sdtContent></w:sdt>"
                            )
                        } else {
                            gap
                        };
                        let outside = if anchored {
                            "<w:p><w:r><w:t>Stable surrounding contractual anchor</w:t></w:r></w:p>"
                                .repeat(5)
                        } else {
                            String::new()
                        };
                        let doc = dom.parse_xdocument(&format!("<w:document xmlns:w='{}'><w:body>{outside}{content}{outside}<w:sectPr/></w:body></w:document>",W::URI));
                        let root = dom.root(doc).unwrap();
                        // Real source packages handed to the public comparer
                        // are clean. Derive them using the library's public
                        // source projection rather than supplying invented
                        // correlated paragraph-mark states to a private pass.
                        let source = if revised {
                            crate::revision_processor::accept_revisions_document(&mut dom, root)
                        } else {
                            crate::revision_processor::reject_revisions_document(&mut dom, root)
                        };
                        roots.push(source);
                    }
                    let original = projected(&mut dom, roots[0], false);
                    let revised = projected(&mut dom, roots[1], true);
                    let bodies = roots
                        .iter()
                        .map(|&root| dom.element(root, &W::body()).unwrap())
                        .collect::<Vec<_>>();
                    let mut settings = if word_mode {
                        WmlComparerSettings::default()
                    } else {
                        WmlComparerSettings::powertools_faithful()
                    };
                    settings.author_for_revisions = "Caller editor".to_string();
                    settings.date_time_for_revisions = DATE.to_string();
                    let redline = crate::comparer::compare_bodies_faithful(
                        &mut dom, roots[0], roots[1], bodies[0], bodies[1], &settings,
                    );
                    let label = format!(
                        "container={container}, tail={tail}, family={family}, anchored={anchored}, word={word_mode}"
                    );
                    assert_eq!(
                        projected(&mut dom, redline, false),
                        original,
                        "reject full caller: {label}; redline XML: {}",
                        dom.serialize_element(redline)
                    );
                    assert_eq!(
                        projected(&mut dom, redline, true),
                        revised,
                        "accept full caller: {label}; redline XML: {}",
                        dom.serialize_element(redline)
                    );
                }
            }
        }
    }
}

#[test]
fn word_m98b_spacing_and_history_match_the_complete_format_golden() {
    // file_167's Word oracle: MIX keeps a deleted pilcrow and no live
    // spacing; the trailing empty owns live spacing plus empty-old history.
    // This intentional Word layout transform differs from source-format
    // equality. Use fixed authored data and literal complete property
    // goldens, including configured revision provenance and schema order.
    let (mut dom, root, body) = document(
        "<w:p><w:pPr><w:pStyle w:val='Heading1'/><w:spacing w:before='360' w:after='80'/></w:pPr><w:ins w:id='10'><w:r><w:rPr><w:b/></w:rPr><w:t>Subsection Title</w:t></w:r></w:ins><w:del w:id='11'><w:r><w:rPr><w:i/></w:rPr><w:delText>24</w:delText></w:r></w:del></w:p><w:p/>",
        "body",
    );
    let settings = WmlComparerSettings {
        author_for_revisions: "Golden editor".to_string(),
        date_time_for_revisions: DATE.to_string(),
        ..WmlComparerSettings::default()
    };
    let mut next_id = 40;
    mixed_spacing_to_following_empty(&mut dom, root, &settings, &mut next_id);
    let (golden, _, golden_body) = document(
        &format!(
            "<w:p><w:pPr><w:pStyle w:val='Heading1'/><w:rPr><w:del w:id='41' w:author='Golden editor' w:date='{DATE}'/></w:rPr></w:pPr><w:ins w:id='10'><w:r><w:rPr><w:b/></w:rPr><w:t>Subsection Title</w:t></w:r></w:ins><w:del w:id='11'><w:r><w:rPr><w:i/></w:rPr><w:delText>24</w:delText></w:r></w:del></w:p><w:p><w:pPr><w:spacing w:before='360' w:after='80'/><w:pPrChange w:id='40' w:author='Golden editor' w:date='{DATE}'><w:pPr/></w:pPrChange></w:pPr></w:p>"
        ),
        "body",
    );
    assert_eq!(
        dom.serialize_element(body),
        golden.serialize_element(golden_body)
    );
    assert_eq!(next_id, 42);
    let once = dom.serialize_element(body);
    mixed_spacing_to_following_empty(&mut dom, root, &settings, &mut next_id);
    assert_eq!(dom.serialize_element(body), once);
    assert_eq!(next_id, 42);
}

#[test]
fn repeated_authored_prefixes_keep_deleted_paragraphs_before_their_unchanged_table() {
    fn owned(view: SourceView) -> (Vec<(char, String)>, Vec<String>, Vec<String>) {
        (
            view.formatted_characters
                .into_iter()
                .map(|(character, _, run)| (character, run))
                .collect(),
            view.nontext_payloads,
            view.table_geometry,
        )
    }
    let tail = "<w:tbl><w:tblPr><w:tblW w:w='1800' w:type='dxa'/><w:tblLook w:val='04A0' w:firstRow='1' w:lastRow='0' w:firstColumn='1' w:lastColumn='0' w:noHBand='0' w:noVBand='1'/></w:tblPr><w:tblGrid><w:gridCol w:w='1800'/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w='1800' w:type='dxa'/></w:tcPr><w:p><w:r><w:t>Unchanged cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl>";
    for (family, original_count, props) in [
        ("cover", 4, "<w:pStyle w:val='BodyText'/>"),
        ("wrap", 3, "<w:spacing w:before='120' w:after='80'/>"),
    ] {
        for container in ["body", "sdtContent"] {
            let mut dom = Dom::new();
            let mut sources = Vec::new();
            for (side, count) in [("del", original_count), ("ins", 2)] {
                let content = (0..count)
                    .map(|index| {
                        authored_paragraph(&text(family, side, index, true), family, props)
                    })
                    .collect::<String>()
                    + tail;
                let (source_dom, source_root, _) = document(&content, container);
                let source_document =
                    dom.parse_xdocument(&source_dom.serialize_element(source_root));
                sources.push(dom.root(source_document).unwrap());
            }
            let expected_original = projected(&mut dom, sources[0], false);
            let expected_revised = projected(&mut dom, sources[1], true);
            for word_mode in [false, true] {
                let mut settings = if word_mode {
                    WmlComparerSettings::default()
                } else {
                    WmlComparerSettings::powertools_faithful()
                };
                settings.author_for_revisions = "Order editor".into();
                settings.date_time_for_revisions = DATE.into();
                let mut results = Vec::new();
                for _ in 0..2 {
                    let old_input = dom.clone_subtree(sources[0]);
                    let new_input = dom.clone_subtree(sources[1]);
                    let old_body = dom.element(old_input, &W::body()).unwrap();
                    let new_body = dom.element(new_input, &W::body()).unwrap();
                    let compared = crate::comparer::compare_bodies_faithful(
                        &mut dom, old_input, new_input, old_body, new_body, &settings,
                    );
                    for (accept, expected) in
                        [(false, &expected_original), (true, &expected_revised)]
                    {
                        let actual = projected(&mut dom, compared, accept);
                        if !word_mode {
                            assert_eq!(
                                &actual, expected,
                                "faithful full source {family}/{container}/accept={accept}"
                            );
                        } else {
                            let expected_text = expected
                                .formatted_characters
                                .iter()
                                .map(|(character, _, _)| *character)
                                .collect::<String>();
                            let actual_text = actual
                                .formatted_characters
                                .iter()
                                .map(|(character, _, _)| *character)
                                .collect::<String>();
                            let expected_owned = (
                                expected
                                    .formatted_characters
                                    .iter()
                                    .map(|(character, _, run)| (*character, run.clone()))
                                    .collect::<Vec<_>>(),
                                expected.nontext_payloads.clone(),
                                expected.table_geometry.clone(),
                            );
                            assert!(
                                owned(actual) == expected_owned,
                                "Word original/revised ownership {family}/{container}/accept={accept}; expected text={expected_text:?}; actual text={actual_text:?}"
                            );
                        }
                    }
                    for kind in [W::ins(), W::del(), W::move_from(), W::move_to()] {
                        for revision in dom.descendants(compared, Some(&kind)) {
                            assert_eq!(dom.attribute(revision, &W::author()), Some("Order editor"));
                            assert_eq!(dom.attribute(revision, &W::date()), Some(DATE));
                        }
                    }
                    results.push(dom.serialize_element(compared));
                }
                assert_eq!(
                    results[0], results[1],
                    "repeat output {family}/{container}/word={word_mode}"
                );
            }
        }
    }
}

#[test]
fn unshared_wrap_gap_keeps_each_sources_preserved_terminal_space() {
    fn owned(view: SourceView) -> (Vec<(char, String)>, Vec<String>, Vec<String>) {
        (
            view.formatted_characters
                .into_iter()
                .map(|(character, _, run)| (character, run))
                .collect(),
            view.nontext_payloads,
            view.table_geometry,
        )
    }
    let tail = "";
    for (family, original_count, props) in [("wrap", 4, "")] {
        for container in ["body", "sdtContent", "tc"] {
            let mut dom = Dom::new();
            let mut sources = Vec::new();
            for (side, count) in [("del", original_count), ("ins", 2)] {
                let content = (0..count)
                    .map(|index| {
                        authored_paragraph(&text(family, side, index, false), family, props)
                    })
                    .collect::<String>()
                    + tail;
                let (source_dom, source_root, _) = document(&content, container);
                let source_document =
                    dom.parse_xdocument(&source_dom.serialize_element(source_root));
                sources.push(dom.root(source_document).unwrap());
            }
            let expected_original = projected(&mut dom, sources[0], false);
            let expected_revised = projected(&mut dom, sources[1], true);
            for word_mode in [false, true] {
                let mut settings = if word_mode {
                    WmlComparerSettings::default()
                } else {
                    WmlComparerSettings::powertools_faithful()
                };
                settings.author_for_revisions = "Order editor".into();
                settings.date_time_for_revisions = DATE.into();
                let mut results = Vec::new();
                for _ in 0..2 {
                    let old_input = dom.clone_subtree(sources[0]);
                    let new_input = dom.clone_subtree(sources[1]);
                    let old_body = dom.element(old_input, &W::body()).unwrap();
                    let new_body = dom.element(new_input, &W::body()).unwrap();
                    let compared = crate::comparer::compare_bodies_faithful(
                        &mut dom, old_input, new_input, old_body, new_body, &settings,
                    );
                    for (accept, expected) in
                        [(false, &expected_original), (true, &expected_revised)]
                    {
                        let actual = projected(&mut dom, compared, accept);
                        if !word_mode {
                            assert_eq!(
                                &actual, expected,
                                "faithful full source {family}/{container}/accept={accept}"
                            );
                        } else {
                            let expected_text = expected
                                .formatted_characters
                                .iter()
                                .map(|(character, _, _)| *character)
                                .collect::<String>();
                            let actual_text = actual
                                .formatted_characters
                                .iter()
                                .map(|(character, _, _)| *character)
                                .collect::<String>();
                            let expected_owned = (
                                expected
                                    .formatted_characters
                                    .iter()
                                    .map(|(character, _, run)| (*character, run.clone()))
                                    .collect::<Vec<_>>(),
                                expected.nontext_payloads.clone(),
                                expected.table_geometry.clone(),
                            );
                            assert!(
                                owned(actual) == expected_owned,
                                "Word original/revised ownership {family}/{container}/accept={accept}; expected text={expected_text:?}; actual text={actual_text:?}"
                            );
                        }
                    }
                    for kind in [W::ins(), W::del(), W::move_from(), W::move_to()] {
                        for revision in dom.descendants(compared, Some(&kind)) {
                            assert_eq!(dom.attribute(revision, &W::author()), Some("Order editor"));
                            assert_eq!(dom.attribute(revision, &W::date()), Some(DATE));
                        }
                    }
                    results.push(dom.serialize_element(compared));
                }
                assert_eq!(
                    results[0], results[1],
                    "repeat output {family}/{container}/word={word_mode}"
                );
            }
        }
    }
}

macro_rules! family_test {
    ($name:ident, $family:literal) => {
        #[test]
        fn $name() {
            exercise($family);
        }
    };
}
family_test!(ordinary_gap_source_views, "ordinary");
family_test!(heading_gap_source_views, "heading");
family_test!(numbered_gap_source_views, "numbered");
family_test!(uniform_list_gap_source_views, "uniform");
family_test!(alpha_label_gap_source_views, "label");
family_test!(title_page_gap_source_views, "cover");
family_test!(repeated_wrap_gap_source_views, "wrap");
family_test!(long_prose_gap_source_views, "long");
family_test!(empty_mark_gap_source_views, "empty");
family_test!(whitespace_gap_source_views, "whitespace");
family_test!(page_break_gap_source_views, "break");
family_test!(math_gap_source_views, "math");
family_test!(field_gap_source_views, "field");

#[derive(Debug, PartialEq, Eq)]
struct OrganicSourceSnapshot {
    owned: SourceView,
    annotations: Vec<String>,
    controls: Vec<String>,
    row_properties: Vec<String>,
    empty_paragraph_properties: Vec<String>,
    comments: Vec<String>,
}

/// One line per `w:comment` of a package, in `comments.xml` order:
/// author, date, initials and text, exactly as the part stores them.
/// No ids (projections renumber them) and no anchor resolution (the
/// `annotations` and `owned` fields pin that geometry already).
fn comment_summaries(package: &crate::opc::PartFs) -> Vec<String> {
    let Some(xml) = package.part_string("word/comments.xml") else {
        return Vec::new();
    };
    let mut dom = Dom::new();
    let comments = dom.parse_xdocument(&xml);
    let root = dom.root(comments).expect("comments root");
    dom.descendants(root, Some(&W::name("comment")))
        .into_iter()
        .map(|comment| {
            let attribute = |name: &XName| {
                dom.attribute(comment, name)
                    .map(str::to_owned)
                    .unwrap_or_default()
            };
            let text: String = dom
                .descendants(comment, Some(&W::t()))
                .into_iter()
                .map(|run| dom.value(run))
                .collect();
            format!(
                "author={:?} date={:?} initials={:?} text={:?}",
                attribute(&W::author()),
                attribute(&W::date()),
                attribute(&W::name("initials")),
                text,
            )
        })
        .collect()
}

fn organic_source_snapshot(bytes: &[u8]) -> OrganicSourceSnapshot {
    let package = crate::opc::PartFs::open(bytes).expect("memory package");
    let mut dom = Dom::new();
    let document =
        dom.parse_xdocument(&package.part_string("word/document.xml").expect("main part"));
    let root = dom.root(document).expect("document");
    remove_powertools_scratch_markup(&mut dom, root);
    let owned = projected(&mut dom, root, true);
    let mut annotations = Vec::new();
    let mut characters = 0usize;
    let mut ids = HashMap::<(String, String), usize>::new();
    for node in dom.descendants(root, None) {
        if dom.name_is(node, &W::t()) {
            characters += dom.value_str(node).chars().count();
        }
        let name = dom.name(node).expect("element");
        let family = match name.local_name() {
            "bookmarkStart" | "bookmarkEnd" => "bookmark",
            "commentRangeStart" | "commentRangeEnd" | "commentReference" => "comment",
            _ => continue,
        };
        if name.namespace_name() != W::URI {
            continue;
        }
        let original_id = dom.attribute(node, &W::id()).expect("range ID").to_owned();
        let next = ids.keys().filter(|(scope, _)| scope == family).count();
        let owner = *ids.entry((family.into(), original_id)).or_insert(next);
        let mut attrs = dom
            .attributes(node)
            .into_iter()
            .filter(|(name, _)| *name != W::id() && !dom.is_namespace_declaration(name))
            .collect::<Vec<_>>();
        attrs.sort_by(|a, b| {
            (a.0.namespace_name(), a.0.local_name()).cmp(&(b.0.namespace_name(), b.0.local_name()))
        });
        annotations.push(format!("{name:?}@{characters}/owner={owner}/{attrs:?}"));
    }
    // Independent ranges may exchange lexical order at the same offset.
    // Exact namespace, character boundary, owner, attributes and
    // multiplicity fully specify their authored spans.
    annotations.sort();
    let controls = dom
        .descendants(root, None)
        .into_iter()
        .filter(|&node| dom.name_is(node, &W::sdt_pr()) || dom.name_is(node, &W::sdt_end_pr()))
        .map(|node| dom.serialize_element(node))
        .collect();
    let row_properties = dom
        .descendants(root, Some(&W::tr_pr()))
        .into_iter()
        .map(|node| dom.serialize_element(node))
        .collect();
    let empty_paragraph_properties = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .filter(|&p| para_has_no_text(&dom, p))
        .map(|p| {
            dom.element(p, &W::p_pr())
                .map(|properties| dom.serialize_element(properties))
                .unwrap_or_default()
        })
        .collect();
    // The comments part as a comparable summary, in `comments.xml`
    // order. The full records (anchors, context, thread state) are
    // asserted through the public API by
    // tests/it_doc/comment_records_survive_projection.rs: this layer may
    // not reach the comments module, and `annotations` with `owned`
    // already
    // pin the anchor geometry and the surrounding text.
    let comments = comment_summaries(&package);
    OrganicSourceSnapshot {
        owned,
        annotations,
        controls,
        row_properties,
        empty_paragraph_properties,
        comments,
    }
}

fn organic_paragraph(text: &str, profile: usize, index: usize, revised: bool) -> String {
    let style = if profile == 1 && index.is_multiple_of(3) {
        "Heading1"
    } else if profile == 2 || (profile == 1 && index % 3 == 1) {
        "ListParagraph"
    } else {
        "BodyText"
    };
    let heading = if style == "Heading1" {
        "<w:keepNext/>"
    } else {
        ""
    };
    let outline = if style == "Heading1" {
        "<w:outlineLvl w:val='0'/>"
    } else {
        ""
    };
    let num = if style == "ListParagraph" {
        format!(
            "<w:numPr><w:ilvl w:val='{}'/><w:numId w:val='9'/></w:numPr>",
            usize::from(profile == 2 && index % 2 == 1)
        )
    } else {
        String::new()
    };
    let before = 120 + 40 * (index % 4) + usize::from(revised) * 20;
    let left = if revised { 540 } else { 360 };
    // Both sides author a semantic justification override. The shared
    // pPrChange projector legitimately omits schema-default left/start,
    // including faithful mode; testing redundant carrier spelling would
    // mistake that established projection for lost source formatting.
    let jc = if profile == 1 && index % 2 == 1 {
        "right"
    } else {
        "center"
    };
    let decoration = if revised { "<w:i/>" } else { "<w:b/>" };
    let color = if revised { "654321" } else { "123456" };
    format!(
        "<w:p><w:pPr><w:pStyle w:val='{style}'/>{heading}{num}<w:spacing w:before='{before}' w:after='80'/><w:ind w:left='{left}'/><w:jc w:val='{jc}'/>{outline}</w:pPr><w:r><w:rPr><w:rFonts w:ascii='Calibri' w:hAnsi='Calibri'/>{decoration}<w:color w:val='{color}'/><w:sz w:val='22'/><w:lang w:val='en-US'/></w:rPr><w:t xml:space='preserve'>{text}</w:t></w:r></w:p>"
    )
}

fn wholesale_boundary_package(fragment: &str) -> Vec<u8> {
    // Reuse the upstream memory package and actual style/numbering parts,
    // but author a whole story with no equal header or closing anchor.
    let base = organic_package("", "body", false, false);
    let mut package = crate::opc::PartFs::open(&base).expect("upstream memory package");
    package.set_part("word/document.xml", format!("<w:document xmlns:w='{}' xmlns:m='{}'><w:body>{fragment}<w:sectPr><w:pgSz w:w='12240' w:h='15840'/><w:pgMar w:top='720' w:right='900' w:bottom='720' w:left='900'/></w:sectPr></w:body></w:document>",W::URI,M::URI).into_bytes());
    package.to_zip().expect("literal whole-story package")
}

fn wholesale_boundary_paragraph(text: &str, properties: &str, side: &str) -> String {
    let color = if side == "original" {
        "123456"
    } else {
        "654321"
    };
    let emphasis = if side == "original" {
        "<w:b/>"
    } else {
        "<w:i/>"
    };
    format!(
        "<w:p><w:pPr>{properties}</w:pPr><w:r><w:rPr><w:rFonts w:ascii='Calibri' w:hAnsi='Calibri'/>{emphasis}<w:color w:val='{color}'/><w:sz w:val='22'/><w:lang w:val='en-US'/></w:rPr><w:t xml:space='preserve'>{text}</w:t></w:r></w:p>"
    )
}

// These packages intentionally omit organic_package's equal anchors. Their
// whole-story D/I replacement boundary reaches Word's final-pair guards.
// Full source formatting is checked in faithful mode. Word's independently
// owned chars/run formats/nontext/geometry remain exact; documented M88,
// M98b/M435 paragraph-property adoption is not a source-pPr round trip.
#[test]
fn whole_story_title_label_heading_and_list_boundaries_keep_source_ownership() {
    type BoundaryCase = (
        &'static str,
        usize,
        usize,
        &'static str,
        &'static str,
        &'static str,
        &'static str,
    );
    const LIST: &str = "<w:numPr><w:ilvl w:val='0'/><w:numId w:val='9'/></w:numPr>";
    const LIST_STYLE: &str = "<w:pStyle w:val='ListParagraph'/><w:numPr><w:ilvl w:val='0'/><w:numId w:val='9'/></w:numPr>";
    const HEADING: &str = "<w:pStyle w:val='Heading1'/>";
    const SPACING: &str = "<w:spacing w:before='120' w:after='240'/>";
    const CENTER: &str = "<w:jc w:val='center'/>";
    let cases: &[BoundaryCase] = &[
        (
            "sole-short-residual",
            1,
            3,
            "Ouch",
            "Violet revised contractual responsibilities survive forever",
            "",
            "",
        ),
        (
            "sole-two-word-residual",
            1,
            3,
            "Amber obligation",
            "Violet revised contractual responsibilities survive forever",
            "",
            "",
        ),
        ("sole-short-carrier", 1, 3, "Ouch", "Violet owners", "", ""),
        (
            "sole-two-insertions",
            1,
            2,
            "Ouch",
            "Violet revised contractual responsibilities survive forever",
            "",
            "",
        ),
        (
            "cover-three-paragraphs",
            1,
            3,
            "Amber original contractual obligations remain binding forever",
            "Prepared for violet counsel",
            "",
            "",
        ),
        (
            "cover-four-paragraphs",
            1,
            4,
            "Amber original contractual obligations remain binding forever",
            "Prepared for violet counsel",
            "",
            "",
        ),
        (
            "cover-email",
            1,
            4,
            "Amber original contractual obligations remain binding forever",
            "violet@example.test",
            "",
            "",
        ),
        (
            "cover-year",
            1,
            4,
            "Amber original contractual obligations remain binding forever",
            "Violet 2040",
            "",
            "",
        ),
        (
            "cover-agreement",
            1,
            4,
            "Amber original contractual obligations remain binding forever",
            "Violet agreement",
            "",
            "",
        ),
        (
            "cover-alpha-stub",
            1,
            4,
            "Amber original contractual obligations remain binding forever",
            "x",
            "",
            "",
        ),
        (
            "alpha-one-carrier",
            2,
            1,
            "Amber original unrelated manuscript narrative",
            "x",
            "",
            "",
        ),
        (
            "alpha-two-carriers",
            2,
            2,
            "Amber original unrelated manuscript narrative",
            "x",
            "",
            "",
        ),
        (
            "alpha-three-carriers",
            2,
            3,
            "Amber original unrelated manuscript narrative",
            "x",
            "",
            "",
        ),
        (
            "long-base-short-label",
            2,
            3,
            "Amber original scholarly manuscript explains unrelated customary duties clearly",
            "Violet",
            "",
            "",
        ),
        (
            "demo-intro-short-label",
            2,
            3,
            "This document demonstrates amber scholarly manuscript customary duties clearly",
            "Violet",
            "",
            "",
        ),
        (
            "ooxml-intro-five-labels",
            2,
            5,
            "This document demonstrates amber OOXML sample manuscript customary duties clearly",
            "Violet",
            "",
            "",
        ),
        (
            "ooxml-intro-four-labels",
            2,
            4,
            "This document demonstrates amber OOXML sample manuscript customary duties clearly",
            "Violet",
            "",
            "",
        ),
        (
            "heading-one-word",
            2,
            3,
            "Generalities",
            "Violet revised contractual responsibilities survive forever",
            HEADING,
            "",
        ),
        (
            "heading-two-words",
            2,
            3,
            "Amber generalities",
            "Violet revised contractual responsibilities survive forever",
            HEADING,
            "",
        ),
        (
            "heading-long-title",
            2,
            3,
            "Amber original scholarly manuscript headings explain contractual duties clearly",
            "Violet revised contractual responsibilities survive forever",
            HEADING,
            "",
        ),
        (
            "short-label-heading",
            2,
            3,
            "Generalities",
            "x",
            HEADING,
            "",
        ),
        (
            "single-revised-two-word-title",
            2,
            1,
            "Amber owners",
            "Violet counsel",
            "",
            "",
        ),
        (
            "single-revised-four-word-title",
            2,
            1,
            "Amber original manuscript owners",
            "Violet counsel",
            "",
            "",
        ),
        (
            "short-list-pair",
            2,
            2,
            "Amber owners",
            "Violet counsel",
            LIST_STYLE,
            LIST,
        ),
        (
            "short-list-carrier-four-words",
            2,
            2,
            "Amber owners",
            "Violet revised counsel owners",
            LIST_STYLE,
            LIST,
        ),
        (
            "uniform-list-three",
            2,
            3,
            "Amber owners",
            "test",
            LIST_STYLE,
            LIST,
        ),
        (
            "uniform-list-two",
            2,
            2,
            "Amber owners",
            "test",
            LIST_STYLE,
            LIST,
        ),
        (
            "uniform-list-long-old",
            2,
            3,
            "Amber original manuscript owners",
            "test",
            LIST_STYLE,
            LIST,
        ),
        (
            "heading-list-adoption",
            2,
            2,
            "Amber confidentiality obligations",
            "Violet counsel",
            HEADING,
            LIST_STYLE,
        ),
        (
            "spacing-center-adoption",
            2,
            2,
            "Amber original manuscript owners",
            "Violet counsel",
            SPACING,
            CENTER,
        ),
        (
            "demo-nine-insertions",
            2,
            9,
            "Amber Font Size Demo",
            "Violet revised contractual responsibilities survive forever",
            "",
            "",
        ),
        (
            "demo-ten-insertions",
            2,
            10,
            "Amber Font Size Demo",
            "Violet revised contractual responsibilities survive forever",
            "",
            "",
        ),
    ];
    let mut failures = Vec::new();
    let mut comparisons = 0usize;
    for &(label, old_count, new_count, first_old, last_new, old_props, new_props) in cases {
        let original_fragment = (0..old_count).map(|index| {
                let text = if index == 0 { first_old.to_string() } else { format!("Amber former manuscript owner clause {index} expressly preserves archival obligations") };
                wholesale_boundary_paragraph(&text,old_props,"original")
            }).collect::<String>();
        let revised_fragment = (0..new_count).map(|index| {
                let text = if index + 1 == new_count { last_new.to_string() } else if label.starts_with("uniform-list") { "test".to_string() } else { format!("Violet next instrument counsel provision {index} binds revised delivery responsibilities") };
                wholesale_boundary_paragraph(&text,new_props,"revised")
            }).collect::<String>();
        let a = wholesale_boundary_package(&original_fragment);
        let b = wholesale_boundary_package(&revised_fragment);
        let original = organic_source_snapshot(&a);
        let revised = organic_source_snapshot(&b);
        for word in [false, true] {
            let settings = WmlComparerSettings {
                author_for_revisions: "Whole story boundary editor".to_string(),
                date_time_for_revisions: DATE.to_string(),
                ..if word {
                    WmlComparerSettings::default()
                } else {
                    WmlComparerSettings::powertools_faithful()
                }
            };
            let compared =
                crate::document_comparer::compare_documents_with_settings(&a, &b, &settings)
                    .expect("real clean whole-story comparison");
            comparisons += 1;
            let package = crate::opc::PartFs::open(&compared).expect("compared memory package");
            let mut dom = Dom::new();
            let document =
                dom.parse_xdocument(&package.part_string("word/document.xml").expect("main part"));
            let root = dom.root(document).expect("document");
            for name in ["ins", "del", "rPrChange", "pPrChange"] {
                for node in dom.descendants(root, Some(&W::name(name))) {
                    assert_eq!(
                        dom.attribute(node, &W::author()),
                        Some("Whole story boundary editor"),
                        "{label}/word={word}/{name} author"
                    );
                    assert_eq!(
                        dom.attribute(node, &W::date()),
                        Some(DATE),
                        "{label}/word={word}/{name} date"
                    );
                    assert!(
                        dom.attribute(node, &W::id())
                            .is_some_and(|id| id.parse::<u32>().is_ok()),
                        "{label}/word={word}/{name} id"
                    );
                }
            }
            assert!(
                !dom.descendants(root, Some(&W::ins())).is_empty(),
                "new source is tracked {label}/word={word}"
            );
            assert!(
                !dom.descendants(root, Some(&W::del())).is_empty(),
                "old source is tracked {label}/word={word}"
            );
            for (accept, expected) in [(false, &original), (true, &revised)] {
                let projection = if accept {
                    crate::document_comparer::accept_revisions(&compared)
                } else {
                    crate::document_comparer::reject_revisions(&compared)
                }
                .expect("whole-story source projection");
                let actual = organic_source_snapshot(&projection);
                let matches = if word {
                    actual
                        .owned
                        .formatted_characters
                        .iter()
                        .map(|(c, _, r)| (c, r))
                        .eq(expected
                            .owned
                            .formatted_characters
                            .iter()
                            .map(|(c, _, r)| (c, r)))
                        && actual.owned.nontext_payloads == expected.owned.nontext_payloads
                        && actual.owned.table_geometry == expected.owned.table_geometry
                        && actual.annotations == expected.annotations
                        && actual.controls == expected.controls
                        && actual.row_properties == expected.row_properties
                        && actual.comments == expected.comments
                } else {
                    actual == *expected
                };
                if !matches {
                    failures.push(format!(
                        "{label}/word={word}/accept={accept}: {}",
                        compact_source_difference(&actual.owned, &expected.owned)
                    ));
                }
                let repeated = if accept {
                    crate::document_comparer::accept_revisions(&projection)
                } else {
                    crate::document_comparer::reject_revisions(&projection)
                }
                .expect("repeat source projection");
                assert_eq!(
                    organic_source_snapshot(&repeated),
                    actual,
                    "repeat {label}/word={word}/accept={accept}"
                );
            }
        }
    }
    assert_eq!(
        comparisons, 64,
        "all thirty-two literal boundaries in both modes"
    );
    assert!(
        failures.is_empty(),
        "{} whole-story ownership failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

fn terminal_layout_paragraph(kind: &str, properties: &str) -> String {
    let format = "<w:rPr><w:rFonts w:ascii='Calibri' w:hAnsi='Calibri'/><w:i/><w:color w:val='654321'/><w:sz w:val='22'/></w:rPr>";
    let body = match kind {
            "empty" => String::new(),
            "spaces" => format!("<w:r>{format}<w:t xml:space='preserve'>  </w:t></w:r>"),
            "field" => format!(
                "<w:r>{format}<w:fldChar w:fldCharType='begin'/></w:r><w:r>{format}<w:instrText xml:space='preserve'> PAGE </w:instrText></w:r><w:r>{format}<w:fldChar w:fldCharType='separate'/></w:r><w:r>{format}<w:t xml:space='preserve'>  </w:t></w:r><w:r>{format}<w:fldChar w:fldCharType='end'/></w:r>"
            ),
            "field-no-cache" => format!(
                "<w:r>{format}<w:fldChar w:fldCharType='begin'/></w:r><w:r>{format}<w:instrText xml:space='preserve'> PAGE </w:instrText></w:r><w:r>{format}<w:fldChar w:fldCharType='end'/></w:r>"
            ),
            "math" => {
                "<m:oMath><m:r><m:rPr><m:sty m:val='p'/></m:rPr><w:rPr><w:rFonts w:ascii='Cambria Math' w:hAnsi='Cambria Math'/></w:rPr><m:t>x+y</m:t></m:r></m:oMath>"
                    .to_string()
            }
            "page-break" => format!("<w:r>{format}<w:br w:type='page'/></w:r>"),
            "line-break" => format!("<w:r>{format}<w:br/></w:r>"),
            _ => panic!("terminal layout fixture kind"),
        };
    let ppr = if properties.is_empty() {
        String::new()
    } else {
        format!("<w:pPr>{properties}</w:pPr>")
    };
    format!("<w:p>{ppr}{body}</w:p>")
}

// M360 field residues and M431 layout breaks have source payload even
// without cached text. Their distinction must come from real atomization,
// not artificial mark/status flags. Each case uses a clean complete package
// and invokes the public comparison and projection pipeline.
#[test]
fn public_terminal_field_math_and_break_boundaries_preserve_all_source_owners() {
    type Case = (
        &'static str,
        &'static str,
        &'static str,
        usize,
        usize,
        usize,
        &'static str,
    );
    const OLD: &str = "Amber original confidential obligations";
    const LONG_OLD: &str = "Amber original confidential manuscript imposes former archival obligations upon bronze custodians throughout perpetuity";
    const HEADING: &str = "<w:pStyle w:val='Heading1'/>";
    const LIST: &str = "<w:pStyle w:val='ListParagraph'/><w:numPr><w:ilvl w:val='0'/><w:numId w:val='9'/></w:numPr>";
    const SPACING: &str = "<w:spacing w:before='120' w:after='240'/>";
    const CENTER: &str = "<w:jc w:val='center'/>";
    let mut cases = Vec::<Case>::new();
    for kind in [
        "empty",
        "spaces",
        "field",
        "field-no-cache",
        "math",
        "page-break",
        "line-break",
    ] {
        for properties in ["", HEADING, LIST, SPACING] {
            cases.push((kind, properties, "", 2, 2, 1, OLD));
        }
    }
    cases.extend([
        ("field", HEADING, CENTER, 2, 2, 1, OLD),
        ("field", HEADING, LIST, 2, 2, 1, OLD),
        ("field", LIST, SPACING, 2, 2, 1, OLD),
        ("field-no-cache", HEADING, CENTER, 2, 2, 1, OLD),
        ("spaces", SPACING, CENTER, 2, 2, 1, OLD),
        ("math", HEADING, LIST, 2, 2, 1, OLD),
        ("field", HEADING, "", 1, 2, 1, OLD),
        ("field", HEADING, "", 3, 2, 1, OLD),
        ("field", HEADING, "", 2, 1, 1, OLD),
        ("field", HEADING, "", 2, 3, 1, OLD),
        ("field", HEADING, "", 2, 5, 1, OLD),
        ("empty", HEADING, "", 2, 2, 2, OLD),
        ("empty", HEADING, "", 2, 2, 3, OLD),
        ("spaces", HEADING, "", 2, 2, 2, OLD),
        ("spaces", HEADING, "", 2, 2, 3, OLD),
        ("math", HEADING, "", 2, 2, 3, OLD),
        ("page-break", HEADING, "", 2, 2, 3, OLD),
    ]);
    // A thirteen-word first-original paragraph satisfies the M359 guard
    // that leaves whitespace/field carriers for the later merge pass.
    // Three original and five contentful revised paragraphs avoid the
    // local (<3) document-scale shortcut before the shell override.
    for kind in [
        "spaces",
        "field",
        "field-no-cache",
        "math",
        "page-break",
        "line-break",
    ] {
        for properties in ["", HEADING] {
            cases.push((kind, properties, "", 3, 5, 1, LONG_OLD));
        }
    }
    let mut failures = Vec::new();
    let mut comparisons = 0usize;
    for (
        index,
        &(
            kind,
            old_properties,
            new_properties,
            old_count,
            content_count,
            shell_count,
            first_original,
        ),
    ) in cases.iter().enumerate()
    {
        let old = (0..old_count)
            .map(|paragraph| {
                wholesale_boundary_paragraph(
                    if paragraph == 0 {
                        first_original
                    } else {
                        "Bronze previous archival manuscript duties"
                    },
                    old_properties,
                    "original",
                )
            })
            .collect::<String>();
        let new = (0..content_count)
            .map(|paragraph| {
                wholesale_boundary_paragraph(
                    if paragraph % 2 == 0 {
                        "Violet revised instrument counsel responsibilities"
                    } else {
                        "Cobalt next signed delivery provisions"
                    },
                    "",
                    "revised",
                )
            })
            .collect::<String>()
            + &terminal_layout_paragraph(kind, new_properties).repeat(shell_count);
        let a = wholesale_boundary_package(&old);
        let b = wholesale_boundary_package(&new);
        let original = organic_source_snapshot(&a);
        let revised = organic_source_snapshot(&b);
        for word in [false, true] {
            let label = format!(
                "layout case={index}/kind={kind}/oldprops={old_properties}/newprops={new_properties}/D={old_count}/contentI={content_count}/shellI={shell_count}/word={word}"
            );
            let settings = WmlComparerSettings {
                author_for_revisions: "Terminal layout owner editor".to_string(),
                date_time_for_revisions: DATE.to_string(),
                ..if word {
                    WmlComparerSettings::default()
                } else {
                    WmlComparerSettings::powertools_faithful()
                }
            };
            let compared =
                crate::document_comparer::compare_documents_with_settings(&a, &b, &settings)
                    .expect("public literal layout comparison");
            comparisons += 1;
            let package = crate::opc::PartFs::open(&compared).expect("compared memory package");
            let mut dom = Dom::new();
            let document =
                dom.parse_xdocument(&package.part_string("word/document.xml").expect("main part"));
            let root = dom.root(document).expect("document");
            for name in ["ins", "del", "pPrChange", "rPrChange"] {
                for revision in dom.descendants(root, Some(&W::name(name))) {
                    assert_eq!(
                        dom.attribute(revision, &W::author()),
                        Some("Terminal layout owner editor"),
                        "{label}/{name} provenance"
                    );
                    assert_eq!(
                        dom.attribute(revision, &W::date()),
                        Some(DATE),
                        "{label}/{name} date"
                    );
                    assert!(
                        dom.attribute(revision, &W::id())
                            .is_some_and(|id| id.parse::<u32>().is_ok()),
                        "{label}/{name} id"
                    );
                }
            }
            for (accept, expected) in [(false, &original), (true, &revised)] {
                let projection = if accept {
                    crate::document_comparer::accept_revisions(&compared)
                } else {
                    crate::document_comparer::reject_revisions(&compared)
                }
                .expect("public layout source projection");
                let actual = organic_source_snapshot(&projection);
                let matches = if word {
                    actual
                        .owned
                        .formatted_characters
                        .iter()
                        .map(|(c, _, r)| (c, r))
                        .eq(expected
                            .owned
                            .formatted_characters
                            .iter()
                            .map(|(c, _, r)| (c, r)))
                        && actual.owned.nontext_payloads == expected.owned.nontext_payloads
                        && actual.owned.table_geometry == expected.owned.table_geometry
                        && actual.annotations == expected.annotations
                        && actual.controls == expected.controls
                        && actual.row_properties == expected.row_properties
                        && actual.comments == expected.comments
                } else {
                    actual == *expected
                };
                if !matches {
                    failures.push(format!(
                            "{label}/accept={accept}: {}; emptyparagraphs={} actual_empty={:?} expected_empty={:?} actual_nontext={:?} expected_nontext={:?}",
                            compact_source_difference(&actual.owned, &expected.owned),
                            actual.empty_paragraph_properties
                                == expected.empty_paragraph_properties,
                            actual.empty_paragraph_properties, expected.empty_paragraph_properties,
                            actual.owned.nontext_payloads, expected.owned.nontext_payloads
                        ));
                }
                let repeat = if accept {
                    crate::document_comparer::accept_revisions(&projection)
                } else {
                    crate::document_comparer::reject_revisions(&projection)
                }
                .expect("repeat layout source projection");
                assert_eq!(
                    organic_source_snapshot(&repeat),
                    actual,
                    "repeat {label}/accept={accept}"
                );
            }
        }
    }
    assert_eq!(
        comparisons, 114,
        "all fifty-seven semantic layout boundaries in both complete modes"
    );
    assert!(
        failures.is_empty(),
        "{} exact terminal-owner failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// TOC-style complex fields legitimately cross paragraph boundaries. The
// unchanged field lead owns begin/instruction/separate; the original end
// belongs to its first replacement paragraph, while the revised end
// belongs to a later replacement paragraph. This is the caller input for
// the field-crossing fold guard, unlike balanced single-paragraph fields.
#[test]
fn public_cross_paragraph_field_ends_keep_exact_source_instruction_and_cache_owners() {
    let field_run = |body: &str| {
        format!(
            "<w:r><w:rPr><w:rFonts w:ascii='Calibri' w:hAnsi='Calibri'/><w:color w:val='345678'/><w:lang w:val='en-US'/></w:rPr>{body}</w:r>"
        )
    };
    let mut failures = Vec::new();
    let mut comparisons = 0;
    for nested in [false, true] {
        for related_head in [false, true] {
            for (old_count, new_count) in [(2usize, 2usize), (2, 3), (3, 2), (3, 3)] {
                for blank_tail in [false, true] {
                    let field_begin = field_run("<w:fldChar w:fldCharType='begin'/>")
                        + &field_run(
                            "<w:instrText xml:space='preserve'> TOC \\o &quot;1-3&quot; \\h </w:instrText>",
                        )
                        + &field_run("<w:fldChar w:fldCharType='separate'/>");
                    let inner_begin = if nested {
                        field_run("<w:fldChar w:fldCharType='begin'/>")
                            + &field_run(
                                "<w:instrText xml:space='preserve'> REF Archive \\h </w:instrText>",
                            )
                            + &field_run("<w:fldChar w:fldCharType='separate'/>")
                    } else {
                        String::new()
                    };
                    let common_prefix = format!(
                        "<w:p><w:pPr><w:pStyle w:val='Normal'/><w:jc w:val='center'/></w:pPr>{field_begin}{inner_begin}{}</w:p>",
                        field_run("<w:t>Unchanged contents field lead</w:t>")
                    );
                    let end = field_run("<w:fldChar w:fldCharType='end'/>");
                    let ends = if nested { end.clone() + &end } else { end };
                    let old = (0..old_count).map(|index| {
                            let text = if index == 0 {
                                if related_head { "TIFF source archive".to_string() } else { "Bronze source archive".to_string() }
                            } else { format!("Amber former manuscript obligations amber{index}") };
                            let paragraph = wholesale_boundary_paragraph(&text, "<w:pStyle w:val='Heading1'/><w:spacing w:before='120' w:after='240'/>", "original");
                            if index == 0 { paragraph.replacen("</w:p>", &format!("{ends}</w:p>"), 1) } else { paragraph }
                        }).collect::<String>();
                    let new = (0..new_count)
                        .map(|index| {
                            let text = if index == 0 {
                                if related_head {
                                    "TIFF revised exhibit".to_string()
                                } else {
                                    "Violet revised exhibit".to_string()
                                }
                            } else {
                                format!("Cobalt maritime instrument responsibilities cobalt{index}")
                            };
                            let paragraph = wholesale_boundary_paragraph(
                                &text,
                                "<w:pStyle w:val='BodyText'/><w:jc w:val='right'/>",
                                "revised",
                            );
                            if index + 1 == new_count {
                                paragraph.replacen("</w:p>", &format!("{ends}</w:p>"), 1)
                            } else {
                                paragraph
                            }
                        })
                        .collect::<String>();
                    let tail = if blank_tail { "<w:p/>" } else { "" };
                    let a = wholesale_boundary_package(&(common_prefix.clone() + &old + tail));
                    let b = wholesale_boundary_package(&(common_prefix + &new + tail));
                    let expected_a = organic_source_snapshot(&a);
                    let expected_b = organic_source_snapshot(&b);
                    for word in [false, true] {
                        comparisons += 1;
                        let label = format!(
                            "cross-field/nested={nested}/head={related_head}/{old_count}x{new_count}/blank={blank_tail}/word={word}"
                        );
                        let mut settings = if word {
                            crate::comparer::WmlComparerSettings::default()
                        } else {
                            crate::comparer::WmlComparerSettings::powertools_faithful()
                        };
                        settings.author_for_revisions = "Field boundary editor".into();
                        settings.date_time_for_revisions = DATE.into();
                        let compared = crate::document_comparer::compare_documents_with_settings(
                            &a, &b, &settings,
                        )
                        .unwrap();
                        let package = crate::opc::PartFs::open(&compared).unwrap();
                        let mut dom = Dom::new();
                        let document =
                            dom.parse_xdocument(&package.part_string("word/document.xml").unwrap());
                        let root = dom.root(document).unwrap();
                        for node in dom.descendants(root, None) {
                            if [W::ins(), W::del(), W::p_pr_change(), W::r_pr_change()]
                                .iter()
                                .any(|name| dom.name_is(node, name))
                            {
                                assert_eq!(
                                    dom.attribute(node, &W::author()),
                                    Some(settings.author_for_revisions.as_str()),
                                    "revision author {label}"
                                );
                                assert_eq!(
                                    dom.attribute(node, &W::date()),
                                    Some(DATE),
                                    "revision date {label}"
                                );
                                assert!(
                                    dom.attribute(node, &W::id())
                                        .is_some_and(|id| id.parse::<u32>().is_ok()),
                                    "numeric revision ID {label}"
                                );
                            }
                        }
                        for (accept, expected) in [(false, &expected_a), (true, &expected_b)] {
                            let projection = if accept {
                                crate::document_comparer::accept_revisions(&compared)
                            } else {
                                crate::document_comparer::reject_revisions(&compared)
                            }
                            .unwrap();
                            let actual = organic_source_snapshot(&projection);
                            let exact = if word {
                                actual
                                    .owned
                                    .formatted_characters
                                    .iter()
                                    .map(|(c, _, r)| (c, r))
                                    .eq(expected
                                        .owned
                                        .formatted_characters
                                        .iter()
                                        .map(|(c, _, r)| (c, r)))
                                    && actual.owned.nontext_payloads
                                        == expected.owned.nontext_payloads
                                    && actual.owned.table_geometry == expected.owned.table_geometry
                                    && actual.annotations == expected.annotations
                                    && actual.controls == expected.controls
                                    && actual.row_properties == expected.row_properties
                                    && actual.comments == expected.comments
                            } else {
                                actual == *expected
                            };
                            if !exact {
                                failures.push(format!("{label}/accept={accept}: {}; empty={:?}/{:?}; field payloads={:?}/{:?}", compact_source_difference(&actual.owned, &expected.owned), actual.empty_paragraph_properties, expected.empty_paragraph_properties, actual.owned.nontext_payloads, expected.owned.nontext_payloads));
                            }
                            let repeated = if accept {
                                crate::document_comparer::accept_revisions(&projection)
                            } else {
                                crate::document_comparer::reject_revisions(&projection)
                            }
                            .unwrap();
                            assert_eq!(
                                organic_source_snapshot(&repeated),
                                actual,
                                "idempotent exact field owners {label}/accept={accept}"
                            );
                        }
                    }
                }
            }
        }
    }
    assert_eq!(
        comparisons, 64,
        "all thirty-two legitimate field source pairs in both modes"
    );
    assert!(
        failures.is_empty(),
        "{} exact cross-field owner failures:\n{}",
        failures.len(),
        failures.into_iter().take(32).collect::<Vec<_>>().join("\n")
    );
}
fn organic_table(revised_geometry: bool, profile: usize) -> String {
    organic_table_with_rows(revised_geometry, revised_geometry, profile)
}

fn organic_table_with_rows(revised_geometry: bool, revised_rows: bool, profile: usize) -> String {
    let width = if revised_geometry { 2400 } else { 1800 };
    let rows = if revised_rows { 2 } else { 1 };
    let mut contents = String::new();
    for row in 0..rows {
        let cells = (0..2)
            .map(|column| {
                let text = format!("Stable table cell owner row{row} column{column}");
                format!(
                    "<w:tc><w:tcPr><w:tcW w:w='{width}' w:type='dxa'/></w:tcPr>{}</w:tc>",
                    organic_paragraph(&text, profile, row + column + 8, false)
                )
            })
            .collect::<String>();
        contents.push_str(&format!(
            "<w:tr><w:trPr><w:trHeight w:val='260' w:hRule='atLeast'/></w:trPr>{cells}</w:tr>"
        ));
    }
    format!(
        "<w:tbl><w:tblPr><w:tblW w:w='{}' w:type='dxa'/><w:tblLook w:val='04A0' w:firstRow='1' w:lastRow='0' w:firstColumn='1' w:lastColumn='0' w:noHBand='0' w:noVBand='1'/></w:tblPr><w:tblGrid><w:gridCol w:w='{width}'/><w:gridCol w:w='{width}'/></w:tblGrid>{contents}</w:tbl>",
        width * 2
    )
}

fn organic_package(fragment: &str, scope: &str, dense: bool, comments: bool) -> Vec<u8> {
    let bookmark_start = "<w:bookmarkStart w:id='6' w:name='StableOwnedBookmark'/>";
    let bookmark_end = "<w:bookmarkEnd w:id='6'/>";
    let comment_start = if comments {
        "<w:commentRangeStart w:id='0'/>"
    } else {
        ""
    };
    let comment_end = if comments {
        "<w:commentRangeEnd w:id='0'/><w:r><w:commentReference w:id='0'/></w:r>"
    } else {
        ""
    };
    let head = format!(
        "<w:p><w:pPr><w:pStyle w:val='BodyText'/></w:pPr>{bookmark_start}{comment_start}<w:r><w:rPr><w:rFonts w:ascii='Calibri' w:hAnsi='Calibri'/><w:sz w:val='22'/></w:rPr><w:t>Stable authored comment and bookmark anchor</w:t></w:r>{comment_end}{bookmark_end}</w:p>"
    );
    let field = "<w:p><w:pPr><w:pStyle w:val='BodyText'/></w:pPr><w:r><w:fldChar w:fldCharType='begin'/></w:r><w:r><w:instrText xml:space='preserve'> PAGE </w:instrText></w:r><w:r><w:fldChar w:fldCharType='separate'/></w:r><w:r><w:t>7</w:t></w:r><w:r><w:fldChar w:fldCharType='end'/></w:r><w:r><w:br/></w:r></w:p>";
    let anchors = if dense {
        (0..4).map(|i| organic_paragraph(&format!("Common contractual anchor clause {i} preserves signed deliverables for both parties permanently"), 0, i + 20, false)).collect::<String>()
    } else {
        String::new()
    };
    let closing = organic_paragraph("Inner scope closing anchor remains unchanged", 0, 30, false);
    let contents = format!("{head}{field}{anchors}{fragment}{closing}");
    let body = match scope {
        "body" => contents,
        "tc" => format!(
            "<w:tbl><w:tblPr><w:tblW w:w='7200' w:type='dxa'/><w:tblLook w:val='04A0' w:firstRow='1' w:lastRow='0' w:firstColumn='1' w:lastColumn='0' w:noHBand='0' w:noVBand='1'/></w:tblPr><w:tblGrid><w:gridCol w:w='7200'/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w='7200' w:type='dxa'/></w:tcPr>{contents}</w:tc></w:tr></w:tbl>{}",
            organic_paragraph("Outer story closing anchor remains unchanged", 0, 31, false)
        ),
        "sdtContent" => format!(
            "<w:sdt><w:sdtPr><w:alias w:val='Owned contractual clauses'/><w:tag w:val='organic-source-owner'/><w:id w:val='42'/><w:richText/></w:sdtPr><w:sdtEndPr><w:color w:val='234567'/></w:sdtEndPr><w:sdtContent>{contents}</w:sdtContent></w:sdt>{}",
            organic_paragraph("Outer story closing anchor remains unchanged", 0, 31, false)
        ),
        _ => panic!("scope"),
    };
    let mut package = crate::opc::PartFs::open(include_bytes!(
        "../../../../../../tests/fixtures/relids/image_doc.docx"
    ))
    .expect("bundled upstream package fixture");
    package.set_part("word/document.xml", format!("<w:document xmlns:w='{}'><w:body>{body}<w:sectPr><w:pgSz w:w='12240' w:h='15840'/><w:pgMar w:top='720' w:right='900' w:bottom='720' w:left='900'/></w:sectPr></w:body></w:document>", W::URI).into_bytes());
    package.set_part("word/styles.xml", format!("<w:styles xmlns:w='{}'><w:style w:type='paragraph' w:default='1' w:styleId='Normal'><w:name w:val='Normal'/></w:style><w:style w:type='paragraph' w:styleId='BodyText'><w:name w:val='Body Text'/><w:pPr><w:jc w:val='left'/></w:pPr></w:style><w:style w:type='paragraph' w:styleId='Heading1'><w:name w:val='heading 1'/><w:pPr><w:keepNext/><w:outlineLvl w:val='0'/></w:pPr></w:style><w:style w:type='paragraph' w:styleId='ListParagraph'><w:name w:val='List Paragraph'/><w:pPr><w:spacing w:after='40'/></w:pPr></w:style></w:styles>", W::URI).into_bytes());
    let levels = (0..2).map(|level| format!("<w:lvl w:ilvl='{level}'><w:start w:val='1'/><w:numFmt w:val='decimal'/><w:lvlText w:val='%{}.'/><w:lvlJc w:val='left'/></w:lvl>", level + 1)).collect::<String>();
    package.set_part("word/numbering.xml", format!("<w:numbering xmlns:w='{}'><w:abstractNum w:abstractNumId='1'><w:multiLevelType w:val='multilevel'/>{levels}</w:abstractNum><w:num w:numId='9'><w:abstractNumId w:val='1'/></w:num></w:numbering>", W::URI).into_bytes());
    package.add_content_type_override(
        "/word/numbering.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml",
    );
    package.add_document_relationship(
        "word/document.xml",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering",
        "numbering.xml",
    );
    if comments {
        package.set_part("word/comments.xml", format!("<w:comments xmlns:w='{}'><w:comment w:id='0' w:author='Source comment owner' w:initials='SO' w:date='2025-01-01T00:00:00Z'><w:p><w:r><w:t>Unchanged authored interpretation</w:t></w:r></w:p></w:comment></w:comments>", W::URI).into_bytes());
        package.add_content_type_override(
            "/word/comments.xml",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
        );
        package.add_document_relationship(
            "word/document.xml",
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
            "comments.xml",
        );
    }
    package.to_zip().expect("memory package")
}

/// These literal clean packages exercise heterogeneous organic production
/// paths rather than supplying private D/I intermediate states. Faithful
/// mode owns full source formats and metadata. Word's M98b spacing/M390
/// control transformations retain their existing exact goldens; its common
/// contract here is ordered chars/rPr, nontext, geometry and annotations.
#[test]
fn organic_heterogeneous_public_packages_preserve_owned_sources_across_modes() {
    let mut failures = Vec::new();
    let mut reported_boundaries = std::collections::HashSet::new();
    let mut failure_count = 0usize;
    let mut cases = 0usize;
    for family in [
        "prose",
        "list-labels",
        "cover",
        "wrap",
        "moved-clauses",
        "interposed-table",
        "changed-table",
    ] {
        for profile in 0..3 {
            let texts_a: Vec<String> = match family {
                    "list-labels" => vec!["Lvl 1 a".into(), "Lvl 1 b".into(), "Lvl 2 i".into(), "Lvl 2 ii".into(), "Lvl 3 z".into()],
                    "cover" => vec!["Original confidentiality obligations bind the parties permanently".into(), "Former mutual agreement remains enforceable".into()],
                    "wrap" => (0..4).map(|i| format!("Original wrap clause {i} ") + &"Let us tightly wrap this complete contractual passage ".repeat(5)).collect(),
                    _ => (0..5).map(|i| format!("Original clause {i} shall preserve signed records and deliver the completed report within thirty days after the request")).collect(),
                };
            let old = texts_a
                .iter()
                .enumerate()
                .map(|(i, text)| organic_paragraph(text, profile, i, false))
                .collect::<Vec<_>>();
            let new: Vec<String> = match family {
                    "moved-clauses" => [3, 0, 4, 1, 2].into_iter().map(|i| old[i].clone()).collect::<Vec<_>>(),
                    "list-labels" => ["a ", "b ", "3"].into_iter().enumerate().map(|(i, text)| organic_paragraph(text, profile, i + 4, true)).collect(),
                    "cover" => ["Revised engagement overview", "Scope of signed deliverables", "New approval arrangements", "Confidential project title", "Prepared for client January 2040"].into_iter().enumerate().map(|(i, text)| organic_paragraph(text, profile, i + 3, true)).collect(),
                    "wrap" => ["ONE", "b"].into_iter().enumerate().map(|(i, text)| organic_paragraph(text, profile, i + 2, true)).collect(),
                    _ => (0..4).map(|i| organic_paragraph(&format!("Revised clause {i} shall preserve signed records and deliver the approved report within forty days after the request"), profile, i + 2, true)).collect(),
                };
            let a_fragment = if family == "interposed-table" {
                old[..2].concat() + &organic_table(false, profile) + &old[2..].concat()
            } else if family == "changed-table" {
                old.concat() + &organic_table(false, profile)
            } else {
                old.concat()
            };
            let b_fragment = if family == "interposed-table" {
                new[..1].concat() + &organic_table(false, profile) + &new[1..].concat()
            } else if family == "changed-table" {
                new.concat() + &organic_table(true, profile)
            } else {
                new.concat()
            };
            // The legacy C# ReconstructElement copies one matched table's
            // tblPr/grid without table-level history. M-TBL supplies that
            // history only in Word mode. Faithful fixtures still change
            // row ownership and full paragraph/run formats, with identical
            // table width/grid; Word retains the actual geometry change.
            // Every mode retains its strict complete literal source oracle.
            let faithful_b_fragment = if family == "changed-table" {
                new.concat() + &organic_table_with_rows(false, true, profile)
            } else {
                b_fragment.clone()
            };
            for scope in ["body", "tc", "sdtContent"] {
                for dense in [false, true] {
                    let a = organic_package(&a_fragment, scope, dense, profile != 0);
                    let b = organic_package(&b_fragment, scope, dense, profile != 0);
                    let faithful_b =
                        organic_package(&faithful_b_fragment, scope, dense, profile != 0);
                    for reversed in [false, true] {
                        for word_mode in [false, true] {
                            let source_b = if word_mode { &b } else { &faithful_b };
                            let (a, b) = if reversed {
                                (source_b, &a)
                            } else {
                                (&a, source_b)
                            };
                            let original = organic_source_snapshot(a);
                            let revised = organic_source_snapshot(b);
                            for detect_moves in [false, true] {
                                cases += 1;
                                let mut settings = if word_mode {
                                    WmlComparerSettings::default()
                                } else {
                                    WmlComparerSettings::powertools_faithful()
                                };
                                settings.author_for_revisions = "Organic boundary editor".into();
                                settings.date_time_for_revisions = DATE.into();
                                settings.detect_moves = detect_moves;
                                let coordinate = format!(
                                    "{family}/profile={profile}/scope={scope}/dense={dense}/reversed={reversed}/word={word_mode}/moves={detect_moves}"
                                );
                                let compared =
                                    match crate::document_comparer::compare_documents_with_settings(
                                        a, b, &settings,
                                    ) {
                                        Ok(compared) => compared,
                                        Err(error) => {
                                            failure_count += 1;
                                            if failures.len() < 24 {
                                                failures.push(format!(
                                                    "{coordinate}: comparison error {error}"
                                                ));
                                            }
                                            continue;
                                        }
                                    };
                                let compared_package =
                                    crate::opc::PartFs::open(&compared).expect("compared package");
                                let mut compared_dom = Dom::new();
                                let compared_document = compared_dom.parse_xdocument(
                                    &compared_package
                                        .part_string("word/document.xml")
                                        .expect("compared main part"),
                                );
                                let compared_root = compared_dom
                                    .root(compared_document)
                                    .expect("compared document");
                                let provenance_valid = compared_dom
                                    .descendants(compared_root, None)
                                    .into_iter()
                                    .filter(|&node| {
                                        compared_dom.name(node).is_some_and(|name| {
                                            name.namespace_name() == W::URI
                                                && matches!(
                                                    name.local_name(),
                                                    "ins"
                                                        | "del"
                                                        | "moveFrom"
                                                        | "moveTo"
                                                        | "rPrChange"
                                                        | "pPrChange"
                                                        | "trPrChange"
                                                        | "tcPrChange"
                                                        | "tblPrChange"
                                                        | "sectPrChange"
                                                )
                                        })
                                    })
                                    .all(|node| {
                                        compared_dom.attribute(node, &W::author())
                                            == Some("Organic boundary editor")
                                            && compared_dom.attribute(node, &W::date())
                                                == Some(DATE)
                                            && compared_dom
                                                .attribute(node, &W::id())
                                                .is_some_and(|id| id.parse::<u32>().is_ok())
                                    });
                                if !provenance_valid {
                                    failure_count += 1;
                                    if failures.len() < 24 {
                                        failures.push(format!("{coordinate}: synthesized history has wrong configured provenance or ID"));
                                    }
                                }
                                for (accept, expected) in [(false, &original), (true, &revised)] {
                                    let projection = if accept {
                                        crate::document_comparer::accept_revisions(&compared)
                                    } else {
                                        crate::document_comparer::reject_revisions(&compared)
                                    }
                                    .expect("public projection");
                                    let actual = organic_source_snapshot(&projection);
                                    let matches = if word_mode {
                                        actual
                                            .owned
                                            .formatted_characters
                                            .iter()
                                            .map(|(c, _, r)| (c, r))
                                            .eq(expected
                                                .owned
                                                .formatted_characters
                                                .iter()
                                                .map(|(c, _, r)| (c, r)))
                                            && actual.owned.nontext_payloads
                                                == expected.owned.nontext_payloads
                                            && actual.owned.table_geometry
                                                == expected.owned.table_geometry
                                            && actual.annotations == expected.annotations
                                            && actual.row_properties == expected.row_properties
                                            && actual.comments == expected.comments
                                    } else {
                                        actual == *expected
                                    };
                                    if !matches {
                                        failure_count += 1;
                                        let boundary = format!(
                                            "{family}/profile={profile}/scope={scope}/word={word_mode}/accept={accept}"
                                        );
                                        if reported_boundaries.insert(boundary) {
                                            failures.push(format!("{coordinate}/accept={accept}: {}; annotations={}, controls={}, rows={}, empty-paragraphs={}, comments={}", compact_source_difference(&actual.owned, &expected.owned), actual.annotations == expected.annotations, actual.controls == expected.controls, actual.row_properties == expected.row_properties, actual.empty_paragraph_properties == expected.empty_paragraph_properties, actual.comments == expected.comments));
                                        }
                                    }
                                    let repeated = if accept {
                                        crate::document_comparer::accept_revisions(&projection)
                                    } else {
                                        crate::document_comparer::reject_revisions(&projection)
                                    }
                                    .expect("repeat public projection");
                                    if organic_source_snapshot(&repeated) != actual {
                                        failure_count += 1;
                                        if failures.len() < 24 {
                                            failures.push(format!("{coordinate}/accept={accept}: projection repeat changes source ownership"));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    assert_eq!(
        cases, 1008,
        "all heterogeneous family/property/scope/correlation/direction/move/mode boundaries executed"
    );
    assert_eq!(
        failure_count,
        0,
        "{failure_count} source contract failures across {cases} comparisons; first failures:\n{}",
        failures.join("\n")
    );
}

fn literal_legacy_table_geometry(
    table_width: u32,
    grid_width: u32,
    cell_width: u32,
    rows: usize,
) -> Vec<String> {
    let cells =
        format!("<w:tc><w:tcPr><w:tcW w:w='{cell_width}' w:type='dxa'/></w:tcPr><w:p/></w:tc>")
            .repeat(2);
    let mut dom = Dom::new();
    let document=dom.parse_xdocument(&format!("<w:tbl xmlns:w='{}'><w:tblPr><w:tblW w:w='{table_width}' w:type='dxa'/><w:tblLook w:val='04A0' w:firstRow='1' w:lastRow='0' w:firstColumn='1' w:lastColumn='0' w:noHBand='0' w:noVBand='1'/></w:tblPr><w:tblGrid><w:gridCol w:w='{grid_width}'/><w:gridCol w:w='{grid_width}'/></w:tblGrid>{}</w:tbl>",W::URI,format!("<w:tr>{cells}</w:tr>").repeat(rows)));
    let root = dom
        .root(document)
        .expect("independent literal table geometry oracle");
    dom.descendants(root, None)
        .into_iter()
        .filter(|&node| {
            [W::tbl_pr(), W::tbl_grid(), W::tc_pr()]
                .iter()
                .any(|name| dom.name_is(node, name))
        })
        .map(|node| dom.serialize_element(node))
        .collect()
}

#[test]
fn public_word_paired_table_width_and_grid_changes_restore_both_authored_geometries() {
    let a = organic_package(&organic_table(false, 0), "body", false, false);
    let b = organic_package(&organic_table(true, 0), "body", false, false);
    for reversed in [false, true] {
        let (a, b) = if reversed { (&b, &a) } else { (&a, &b) };
        let settings = WmlComparerSettings {
            author_for_revisions: "Paired table geometry editor".to_string(),
            date_time_for_revisions: DATE.to_string(),
            ..WmlComparerSettings::default()
        };
        let compared = crate::document_comparer::compare_documents_with_settings(a, b, &settings)
            .expect("clean public Word table comparison");
        let package = crate::opc::PartFs::open(&compared).expect("memory compared package");
        let mut dom = Dom::new();
        let document =
            dom.parse_xdocument(&package.part_string("word/document.xml").expect("main part"));
        let root = dom.root(document).expect("document");
        for name in ["tblPrChange", "tblGridChange"] {
            let history = dom.descendants(root, Some(&W::name(name)));
            assert_eq!(history.len(), 1, "one exact table history {name}");
            let change = history[0];
            assert!(
                dom.attribute(change, &W::id())
                    .is_some_and(|id| id.parse::<u32>().is_ok())
            );
            if name == "tblPrChange" {
                assert_eq!(
                    dom.attribute(change, &W::author()),
                    Some("Paired table geometry editor")
                );
                assert_eq!(dom.attribute(change, &W::date()), Some(DATE));
            } else {
                // CT_TblGridChange declares id only, not CT_TrackChange provenance.
                assert!(dom.attribute(change, &W::author()).is_none());
                assert!(dom.attribute(change, &W::date()).is_none());
            }
        }
        for accept in [false, true] {
            let expected = organic_source_snapshot(if accept { b } else { a });
            let projection = if accept {
                crate::document_comparer::accept_revisions(&compared)
            } else {
                crate::document_comparer::reject_revisions(&compared)
            }
            .expect("public Word table projection");
            let actual = organic_source_snapshot(&projection);
            assert!(
                actual == expected,
                "Word reversed={reversed}/accept={accept}: complete authored source: {}",
                compact_source_difference(&actual.owned, &expected.owned)
            );
            let repeated = if accept {
                crate::document_comparer::accept_revisions(&projection)
            } else {
                crate::document_comparer::reject_revisions(&projection)
            }
            .expect("repeat Word projection");
            assert_eq!(organic_source_snapshot(&repeated), actual);
        }
    }
}

// WmlComparer v10.0.0 ReconstructElement (6731-6751) copies the matched
// ancestor's table properties/grid without an old table-level history.
// PowerTools mode preserves this limitation: rejecting restores source
// rows/cell properties/text, but keeps the revised table width and grid.
// Word's M-TBL history is tested separately with exact A/B source oracles.
#[test]
fn public_faithful_paired_table_geometry_has_an_exact_legacy_omission_golden() {
    let a = organic_package(&organic_table(false, 0), "body", false, false);
    let b = organic_package(&organic_table(true, 0), "body", false, false);
    for reversed in [false, true] {
        let (a, b) = if reversed { (&b, &a) } else { (&a, &b) };
        let settings = WmlComparerSettings {
            author_for_revisions: "Paired table geometry editor".to_string(),
            date_time_for_revisions: DATE.to_string(),
            ..WmlComparerSettings::powertools_faithful()
        };
        let compared = crate::document_comparer::compare_documents_with_settings(a, b, &settings)
            .expect("clean public legacy table comparison");
        let package = crate::opc::PartFs::open(&compared).expect("memory package");
        let mut dom = Dom::new();
        let document =
            dom.parse_xdocument(&package.part_string("word/document.xml").expect("main part"));
        let root = dom.root(document).expect("document");
        for name in ["tblPrChange", "tblGridChange"] {
            assert!(
                dom.descendants(root, Some(&W::name(name))).is_empty(),
                "legacy table history stays absent"
            );
        }
        for accept in [false, true] {
            let mut expected = organic_source_snapshot(if accept { b } else { a });
            // Literal compatibility output: table-level geometry is B's;
            // source cell widths and row cardinality still follow projection.
            let (table_width, grid_width) = if reversed { (3600, 1800) } else { (4800, 2400) };
            let (cell_width, rows) = match (reversed, accept) {
                (false, false) | (true, true) => (1800, 1),
                (false, true) | (true, false) => (2400, 2),
            };
            expected.owned.table_geometry =
                literal_legacy_table_geometry(table_width, grid_width, cell_width, rows);
            let projection = if accept {
                crate::document_comparer::accept_revisions(&compared)
            } else {
                crate::document_comparer::reject_revisions(&compared)
            }
            .expect("legacy source projection");
            let actual = organic_source_snapshot(&projection);
            assert!(
                actual == expected,
                "legacy reversed={reversed}/accept={accept}: exact compatibility tree: {}",
                compact_source_difference(&actual.owned, &expected.owned)
            );
            let repeated = if accept {
                crate::document_comparer::accept_revisions(&projection)
            } else {
                crate::document_comparer::reject_revisions(&projection)
            }
            .expect("repeat legacy projection");
            assert_eq!(organic_source_snapshot(&repeated), actual);
        }
    }
}

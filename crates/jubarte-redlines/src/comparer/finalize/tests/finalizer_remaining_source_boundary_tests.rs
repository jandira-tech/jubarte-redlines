// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;

const DATE: &str = "2026-01-02T03:04:05Z";
const PROPS: &str =
    "<w:pStyle w:val='BodyText'/><w:spacing w:before='120' w:after='240'/><w:ind w:left='360'/>";
const FORMAT: &str = "<w:rPr><w:b/><w:color w:val='123456'/><w:lang w:val='en-US'/></w:rPr>";
const TABLE: &str = "<w:tbl><w:tblPr><w:tblW w:w='1800' w:type='dxa'/></w:tblPr><w:tblGrid><w:gridCol w:w='1800'/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w='1800' w:type='dxa'/></w:tcPr><w:p><w:r><w:t>Independent table payload</w:t></w:r></w:p></w:tc></w:tr></w:tbl>";

fn package(body: &str) -> (Dom, NodeId, NodeId) {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&format!(
        "<w:document xmlns:w='{}' xmlns:pt='{}'><w:body>{body}</w:body></w:document>",
        W::URI,
        PT::URI
    ));
    let root = dom.root(doc).expect("document");
    let body = dom.element(root, &W::body()).expect("body");
    (dom, root, body)
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
fn run(kind: &str, text: &str, author: &str, date: &str) -> String {
    let tag = if kind == "del" { "delText" } else { "t" };
    let run = format!("<w:r>{FORMAT}<w:{tag} xml:space='preserve'>{text}</w:{tag}></w:r>");
    if kind == "live" {
        run
    } else {
        format!("<w:{kind} w:id='17' w:author='{author}' w:date='{date}'>{run}</w:{kind}>")
    }
}
fn para(kind: &str, text: &str, props: &str) -> String {
    format!(
        "<w:p><w:pPr>{props}</w:pPr>{}</w:p>",
        run(kind, text, "Boundary editor", DATE)
    )
}
fn marked(
    kind: &str,
    text: &str,
    props: &str,
    mark_author: &str,
    mark_date: &str,
    body_author: &str,
    body_date: &str,
) -> String {
    format!(
        "<w:p><w:pPr>{props}<w:rPr><w:{kind} w:id='18' w:author='{mark_author}' w:date='{mark_date}'/></w:rPr></w:pPr>{}</w:p>",
        run(kind, text, body_author, body_date)
    )
}
fn assert_body(dom: &Dom, body: NodeId, expected: &str, label: &str) {
    let (want, _, wb) = package(expected);
    let actual = semantic(dom, body);
    let expected = semantic(&want, wb);
    if actual != expected {
        let first = actual
            .chars()
            .zip(expected.chars())
            .position(|(a, b)| a != b)
            .unwrap_or_else(|| actual.chars().count().min(expected.chars().count()));
        let context = |text: &str| {
            text.chars()
                .skip(first.saturating_sub(64))
                .take(240)
                .collect::<String>()
        };
        panic!(
            "{label}: complete source tree differs at character {first}; actual length {}, expected length {}; actual context {:?}; expected context {:?}",
            actual.len(),
            expected.len(),
            context(&actual),
            context(&expected)
        );
    }
}
fn words(prefix: &str, count: usize) -> String {
    (0..count)
        .map(|n| format!("{prefix}{n}"))
        .collect::<Vec<_>>()
        .join(" ")
}

// The public Word policy distinguishes local edits from wholesale gaps.
// These literals pin its recorded list/cover/wrap exceptions and numeric
// document-share boundaries; every decision is read-only on full source DOM.
#[test]
fn document_scale_folds_preserve_authored_sources_and_observed_word_exceptions() {
    let list = "<w:pStyle w:val='ListParagraph'/><w:numPr><w:ilvl w:val='0'/><w:numId w:val='7'/></w:numPr>";
    let numbered = "<w:numPr><w:ilvl w:val='0'/><w:numId w:val='7'/></w:numPr>";
    type SourceFoldCase = (
        String,
        Vec<(String, String)>,
        Vec<(String, String)>,
        usize,
        bool,
    );
    let mut cases: Vec<SourceFoldCase> = Vec::new();
    for fingerprint in [
        "Prepared by Dana",
        "dana@example.test",
        "March 2041",
        "January 2041",
        "February 2041",
        "April 2041",
        "June 2041",
        "July 2041",
        "August 2041",
        "September 2041",
        "October 2041",
        "November 2041",
        "December 2041",
        "2040",
        "Agreement",
    ] {
        for count in [3, 4] {
            let mut ins = (0..count - 1)
                .map(|n| (words(&format!("revised{n}"), 3), PROPS.to_string()))
                .collect::<Vec<_>>();
            ins.push((fingerprint.to_string(), PROPS.to_string()));
            let del = vec![
                (
                    "Original obligations bind the parties permanently".to_string(),
                    PROPS.to_string(),
                ),
                (
                    "Earlier signature obligations".to_string(),
                    PROPS.to_string(),
                ),
            ];
            cases.push((
                format!("cover/{fingerprint}/{count}"),
                ins,
                del,
                0,
                count == 3,
            ));
        }
    }
    for (iw, dw, filler, want, label) in [
        (6, 33, 0, true, "gap39"),
        (6, 34, 0, false, "gap40"),
        (9, 36, 29, false, "share-above60"),
        (9, 36, 30, true, "share60"),
        (9, 36, 31, true, "share-below60"),
        (10, 39, 0, true, "ratio-below4"),
        (10, 40, 0, false, "ratio4"),
    ] {
        let mut ins = Vec::new();
        let mut del = Vec::new();
        for n in 0..3 {
            let i = iw / 3 + usize::from(n < iw % 3);
            let d = dw / 3 + usize::from(n < dw % 3);
            ins.push((words(&format!("next{n}"), i), PROPS.to_string()));
            del.push((words(&format!("base{n}"), d), PROPS.to_string()));
        }
        cases.push((label.to_string(), ins, del, filler, want));
    }
    for obstruction in [
        "none",
        "no-last-number",
        "few-base-lists",
        "long-next",
        "long-base",
        "related",
        "balanced",
    ] {
        let mut ins = vec![
            ("Revised melon".to_string(), list.to_string()),
            ("Revised peach".to_string(), list.to_string()),
        ];
        let count = if obstruction == "balanced" { 3 } else { 4 };
        let mut del = (0..count)
            .map(|n| (format!("Original berry{n}"), numbered.to_string()))
            .collect::<Vec<_>>();
        match obstruction {
            "no-last-number" => ins[1].1 = PROPS.to_string(),
            "few-base-lists" => {
                for p in del.iter_mut().skip(1) {
                    p.1 = PROPS.to_string();
                }
            }
            "long-next" => ins[0].0 = words("longnext", 13),
            "long-base" => del[1].0 = words("longbase", 13),
            "related" => del[1].0 = "Revised melon".to_string(),
            _ => {}
        }
        cases.push((
            format!("short-lists/{obstruction}"),
            ins,
            del,
            0,
            obstruction != "none",
        ));
    }
    for (title, want) in [("Contents", false), ("Demonstration", true), ("Demo", true)] {
        cases.push((
            format!("label/{title}"),
            vec![
                ("ONE".into(), PROPS.into()),
                ("TWO".into(), PROPS.into()),
                ("A".into(), PROPS.into()),
                ("B".into(), PROPS.into()),
            ],
            vec![
                (title.into(), PROPS.into()),
                ("First chapter".into(), PROPS.into()),
                ("Second chapter".into(), PROPS.into()),
                ("Last chapter".into(), PROPS.into()),
            ],
            0,
            want,
        ));
    }
    for repeat in [true, false] {
        let text = if repeat {
            "Lets tightly wrap this phrase ".repeat(5)
        } else {
            words("unique", 25)
        };
        cases.push((
            format!("long-wrap/{repeat}"),
            vec![(text, PROPS.into())],
            (0..3)
                .map(|n| (words(&format!("basewrap{n}"), 6), PROPS.to_string()))
                .collect(),
            0,
            !repeat,
        ));
    }
    for (i, d, want) in [(10, 50, false), (9, 50, true), (10, 49, true)] {
        cases.push((
            format!("large/{i}/{d}"),
            (0..i)
                .map(|n| (words(&format!("newlarge{n}"), 3), PROPS.to_string()))
                .collect(),
            (0..d)
                .map(|n| (words(&format!("oldlarge{n}"), 6), PROPS.to_string()))
                .collect(),
            1000,
            want,
        ));
    }
    cases.push((
        "numbered uniform labels".into(),
        vec![("test".into(), numbered.into()); 4],
        vec![
            ("Num 1".into(), numbered.into()),
            ("Num 2".into(), numbered.into()),
            ("Num 3".into(), numbered.into()),
        ],
        0,
        true,
    ));
    cases.push((
        "list to short heading".into(),
        vec![("Citation".into(), list.into()); 4],
        vec![
            (
                "MUTUAL NONDISCLOSURE AGREEMENT".into(),
                "<w:pStyle w:val='Heading1'/>".into()
            );
            4
        ],
        0,
        true,
    ));
    cases.push((
        "demo introduction".into(),
        vec![
            ("Grapes".into(), PROPS.into()),
            ("Pears".into(), PROPS.into()),
            ("Melons".into(), PROPS.into()),
        ],
        vec![
            (
                "This document demonstrates practical original layout rules".into(),
                PROPS.into()
            );
            3
        ],
        0,
        true,
    ));
    cases.push((
        "nonboundary related paragraph".into(),
        vec![
            ("shared original phrase".into(), PROPS.into()),
            ("next unrelated".into(), PROPS.into()),
            ("next final".into(), PROPS.into()),
        ],
        vec![
            ("base unrelated phrase".into(), PROPS.into()),
            ("shared original phrase".into(), PROPS.into()),
            ("base final".into(), PROPS.into()),
        ],
        0,
        true,
    ));
    cases.push((
        "empty original separator".into(),
        vec![("new section content".into(), PROPS.into()); 3],
        vec![
            ("".into(), PROPS.into()),
            ("old remaining content".into(), PROPS.into()),
            ("old final content".into(), PROPS.into()),
        ],
        0,
        true,
    ));
    for (label, ins, del, filler, want) in cases {
        let mut xml = String::new();
        if filler > 0 {
            xml.push_str(&para("live", &words("unchanged", filler), PROPS));
        }
        let offset = usize::from(filler > 0);
        for (t, p) in &ins {
            xml.push_str(&para("ins", t, p));
        }
        for (t, p) in &del {
            xml.push_str(&para("del", t, p));
        }
        xml.push_str(TABLE);
        let (dom, root, body) = package(&xml);
        let ps = dom.elements(body, Some(&W::p()));
        let inss = &ps[offset..offset + ins.len()];
        let dels = &ps[offset + ins.len()..];
        let before = semantic(&dom, root);
        assert_eq!(
            should_fold_multi_del_at_document_scale(
                &dom,
                body,
                *inss.last().unwrap(),
                dels[0],
                inss,
                dels
            ),
            want,
            "{label}"
        );
        assert_eq!(semantic(&dom, root), before, "decision mutated {label}");
    }
}

#[test]
fn midstream_deleted_blocks_cross_only_payload_that_disappears_on_rejection() {
    for anchor in [
        "Tip: Revised usage",
        "Table of Contents",
        "1. Revised section",
        "Ordinary revised section",
    ] {
        for prefix_count in [39, 40, 41] {
            for del_count in [1, 2, 6, 7] {
                for barrier in [
                    "none",
                    "live",
                    "matched-table",
                    "inserted-table",
                    "mixed-row",
                ] {
                    let mut prefix = vec![para("ins", anchor, PROPS)];
                    for n in 1..prefix_count {
                        prefix.push(para("ins", &format!("Revised independent item {n}"), PROPS));
                    }
                    let row_ins = format!(
                        "<w:trPr><w:ins w:id='31' w:author='Boundary editor' w:date='{DATE}'/></w:trPr>"
                    );
                    let inserted_table = TABLE
                        .replacen("<w:tr>", &format!("<w:tr>{row_ins}"), 1)
                        .replace(
                            "<w:r><w:t>Independent table payload</w:t></w:r>",
                            &run("ins", "Independent table payload", "Boundary editor", DATE),
                        );
                    match barrier { "live"=>prefix[20]=para("live","Original live anchor",PROPS), "matched-table"=>prefix[20]=TABLE.into(), "inserted-table"=>prefix[20]=inserted_table.clone(), "mixed-row"=>prefix[20]=inserted_table.replacen("</w:trPr>",&format!("<w:del w:id='32' w:author='Original editor' w:date='{DATE}'/></w:trPr>"),1), _=>{} }
                    let dels=(0..del_count).map(|n|para("del",if n==0 {"Label"} else {"Original detailed paragraph that remains in the deleted source sequence"},PROPS)).collect::<Vec<_>>();
                    let xml = prefix.concat()
                        + &dels.concat()
                        + "<w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr>";
                    let (mut dom, root, body) = package(&xml);
                    splice_trailing_short_pure_dels_midstream(&mut dom, root);
                    let moves = anchor != "Ordinary revised section"
                        && prefix_count >= 40
                        && del_count <= 6
                        && matches!(barrier, "none" | "inserted-table");
                    let want = if !moves {
                        xml.clone()
                    } else if anchor.starts_with("1.") {
                        prefix[0].clone()
                            + &dels[0]
                            + &prefix[1..].concat()
                            + &dels[1..].concat()
                            + "<w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr>"
                    } else {
                        prefix[0].clone()
                            + &dels.concat()
                            + &prefix[1..].concat()
                            + "<w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr>"
                    };
                    let label = format!("{anchor}/{prefix_count}/{del_count}/{barrier}");
                    assert_body(&dom, body, &want, &label);
                    let once = semantic(&dom, root);
                    splice_trailing_short_pure_dels_midstream(&mut dom, root);
                    assert_eq!(semantic(&dom, root), once, "repeat {label}");
                }
            }
        }
    }
}

#[test]
fn paired_lifetimes_require_complete_matching_comparison_provenance() {
    for side in [0, 1] {
        for obstruction in [
            "none",
            "foreign-mark",
            "foreign-body",
            "body-date",
            "mark-date",
            "missing-mark",
            "opposite-mark",
            "missing-body",
            "missing-properties",
            "missing-mark-properties",
        ] {
            let mut ps = Vec::new();
            for (n, kind) in ["del", "ins"].into_iter().enumerate() {
                let bad = n == side;
                let ma = if bad && obstruction == "foreign-mark" {
                    "Legacy editor"
                } else {
                    "Boundary editor"
                };
                let ba = if bad && obstruction == "foreign-body" {
                    "Legacy editor"
                } else {
                    "Boundary editor"
                };
                let md = if bad && obstruction == "mark-date" {
                    "2025-01-02T03:04:05Z"
                } else {
                    DATE
                };
                let bd = if bad && obstruction == "body-date" {
                    "2025-01-02T03:04:05Z"
                } else {
                    DATE
                };
                let mut p = marked(
                    kind,
                    if n == 0 {
                        "Original authored text"
                    } else {
                        "Revised authored text"
                    },
                    PROPS,
                    ma,
                    md,
                    ba,
                    bd,
                );
                if bad {
                    match obstruction {
                        "missing-mark"=>p=p.replace(&format!("<w:{kind} w:id='18' w:author='{ma}' w:date='{md}'/>"),""),
                        "opposite-mark"=>p=p.replacen("</w:rPr></w:pPr>",&format!("<w:{} w:id='19' w:author='Boundary editor' w:date='{DATE}'/></w:rPr></w:pPr>",if kind=="del" {"ins"} else {"del"}),1),
                        "missing-body"=>p=p.replace(&run(kind,if n==0 {"Original authored text"} else {"Revised authored text"},ba,bd),&run("live","Surviving live source","Boundary editor",DATE)),
                        "missing-properties"=>{ let start=p.find("<w:pPr>").unwrap(); let end=p.find("</w:pPr>").unwrap() + "</w:pPr>".len(); p.replace_range(start..end,""); },
                        "missing-mark-properties"=>{ let start=p.find("<w:rPr>").unwrap(); let end=p.find("</w:rPr>").unwrap() + "</w:rPr>".len(); p.replace_range(start..end,""); }, _=>{} }
                }
                ps.push(p);
            }
            let (dom, root, body) = package(&(ps.concat() + TABLE));
            let p = dom.elements(body, Some(&W::p()));
            let before = semantic(&dom, root);
            assert_eq!(
                paired_replacement_marks_belong_to_comparer(&dom, p[0], p[1], "Boundary editor"),
                obstruction == "none",
                "{side}/{obstruction}"
            );
            assert_eq!(
                semantic(&dom, root),
                before,
                "source provenance must remain immutable"
            );
        }
    }
}

#[test]
fn paired_format_equality_is_complete_and_uses_no_production_arena_nodes() {
    for side in [0, 1] {
        for obstruction in [
            "none",
            "style",
            "spacing",
            "indent",
            "history",
            "section",
            "move-from",
            "move-to",
            "mark-history",
            "mark-bold",
            "missing-mark",
        ] {
            let mut ps = Vec::new();
            for (n, kind) in ["del", "ins"].into_iter().enumerate() {
                let props = if n == side {
                    match obstruction {
                        "style" => PROPS.replace("BodyText", "Heading1"),
                        "spacing" => PROPS.replace("240", "241"),
                        "indent" => PROPS.replace("360", "361"),
                        _ => PROPS.to_string(),
                    }
                } else {
                    PROPS.to_string()
                };
                let mut p = marked(
                    kind,
                    if n == 0 {
                        "Original source"
                    } else {
                        "Revised source"
                    },
                    &props,
                    "Boundary editor",
                    DATE,
                    "Boundary editor",
                    DATE,
                );
                if n == side {
                    match obstruction {
                        "history"=>p=p.replacen("</w:pPr>",&format!("<w:pPrChange w:id='29' w:author='Prior editor' w:date='{DATE}'><w:pPr><w:spacing w:after='80'/></w:pPr></w:pPrChange></w:pPr>"),1),
                        "section"=>p=p.replacen("</w:pPr>","<w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr></w:pPr>",1),
                        "move-from"|"move-to"=>p=p.replacen("</w:rPr></w:pPr>",&format!("<w:{} w:id='29' w:author='Prior editor' w:date='{DATE}'/></w:rPr></w:pPr>",if obstruction=="move-from" {"moveFrom"} else {"moveTo"}),1),
                        "mark-history"=>p=p.replacen("</w:rPr></w:pPr>",&format!("<w:rPrChange w:id='29' w:author='Prior editor' w:date='{DATE}'><w:rPr><w:i/></w:rPr></w:rPrChange></w:rPr></w:pPr>"),1),
                        "mark-bold"=>p=p.replacen("<w:rPr>","<w:rPr><w:b/>",1),
                        "missing-mark"=>p=p.replace(&format!("<w:{kind} w:id='18' w:author='Boundary editor' w:date='{DATE}'/>"),""), _=>{} }
                }
                ps.push(p);
            }
            let (mut dom, root, body) = package(&(ps.concat() + TABLE));
            let p = dom.elements(body, Some(&W::p()));
            let before = semantic(&dom, root);
            let count = dom.node_count();
            for _ in 0..3 {
                assert_eq!(
                    paired_replacement_has_identical_properties(&mut dom, p[0], p[1]),
                    obstruction == "none",
                    "{side}/{obstruction}"
                );
            }
            assert_eq!(
                semantic(&dom, root),
                before,
                "property comparison must not modify authored format/history"
            );
            assert_eq!(
                dom.node_count(),
                count,
                "property probes must not retain orphan DOM allocations"
            );
        }
    }
}

#[test]
fn two_authored_layout_blanks_are_restored_only_at_the_shared_table_boundary() {
    let blank1 = "<w:p><w:pPr><w:spacing w:after='80'/><w:rPr><w:b/></w:rPr></w:pPr><w:r><w:rPr><w:lang w:val='en-US'/></w:rPr></w:r></w:p>";
    let blank2 = "<w:p><w:pPr><w:pStyle w:val='BodyText'/><w:spacing w:after='120'/></w:pPr><w:r><w:rPr><w:i/></w:rPr></w:r></w:p>";
    for obstruction in [
        "none",
        "deleted-table",
        "inserted-table",
        "inserted-empty",
        "nonempty-separator",
        "no-separator",
        "no-table",
        "missing-source-title",
        "duplicate-source-title",
        "one-source-blank",
        "three-source-blanks",
        "source-blank-section",
        "long-original",
        "long-heading",
        "live-original",
        "live-revised",
        "empty-revised",
    ] {
        let source_title = para("live", "Revised authored title", PROPS);
        let mut source = source_title.clone() + blank1 + blank2 + TABLE;
        match obstruction {
                "missing-source-title"=>source=source.replace("Revised authored title","Other authored title"),
                "duplicate-source-title"=>source=source+&source_title,
                "one-source-blank"=>source=source_title.clone()+blank1+TABLE,
                "three-source-blanks"=>source=source_title.clone()+blank1+blank2+"<w:p/>"+TABLE,
                "source-blank-section"=>source=source.replace(blank1,"<w:p><w:pPr><w:spacing w:after='80'/><w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr></w:pPr></w:p>"), _=>{} }
        let ins = para(
            if obstruction == "live-revised" {
                "live"
            } else {
                "ins"
            },
            if obstruction == "empty-revised" {
                ""
            } else {
                "Revised authored title"
            },
            PROPS,
        );
        let del = para(
            if obstruction == "live-original" {
                "live"
            } else {
                "del"
            },
            if matches!(obstruction, "long-original" | "long-heading") {
                "Original authored heading has many independently retained words"
            } else {
                "Original title"
            },
            if obstruction == "long-heading" {
                "<w:pStyle w:val='Heading1'/>"
            } else {
                PROPS
            },
        );
        let empty = match obstruction {
            "inserted-empty" => marked(
                "ins",
                "",
                "",
                "Boundary editor",
                DATE,
                "Boundary editor",
                DATE,
            ),
            "nonempty-separator" => para("live", "Surviving original separator", PROPS),
            "no-separator" => String::new(),
            _ => "<w:p/>".to_string(),
        };
        let table = match obstruction {
            "deleted-table" | "inserted-table" => TABLE.replace(
                "<w:p><w:r><w:t>Independent table payload</w:t></w:r></w:p>",
                &marked(
                    if obstruction == "deleted-table" {
                        "del"
                    } else {
                        "ins"
                    },
                    "Independent table payload",
                    PROPS,
                    "Boundary editor",
                    DATE,
                    "Boundary editor",
                    DATE,
                ),
            ),
            "no-table" => para("live", "Independent authored tail", PROPS),
            _ => TABLE.to_string(),
        };
        let input = ins.clone() + &del + &empty + &table;
        let (mut dom, root, body) = package(&input);
        let source_doc = dom.parse_xdocument(&format!(
            "<w:document xmlns:w='{}'><w:body>{source}</w:body></w:document>",
            W::URI
        ));
        let source_root = dom.root(source_doc).unwrap();
        let source_body = dom.element(source_root, &W::body()).unwrap();
        let source_before = semantic(&dom, source_root);
        let templates = revised_two_blank_table_titles(&mut dom, source_body);
        let settings = WmlComparerSettings {
            author_for_revisions: "Boundary editor".into(),
            date_time_for_revisions: DATE.into(),
            ..WmlComparerSettings::default()
        };
        let mut id = 50;
        ensure_empty_pure_i_before_short_title_del(&mut dom, root, &templates, &settings, &mut id);
        let restores = matches!(obstruction, "none" | "inserted-table" | "long-heading");
        let expected = if restores {
            let mark =
                |n| format!("<w:ins w:id='{n}' w:author='Boundary editor' w:date='{DATE}'/>");
            let first = blank1.replacen("<w:rPr><w:b/>", &format!("<w:rPr>{}<w:b/>", mark(50)), 1);
            let second = blank2.replacen(
                "</w:pPr>",
                &format!("<w:rPr>{}</w:rPr></w:pPr>", mark(51)),
                1,
            );
            let old_blank = format!(
                "<w:p><w:pPr><w:rPr><w:del w:id='52' w:author='Boundary editor' w:date='{DATE}'/></w:rPr></w:pPr></w:p>"
            );
            ins + &first + &second + &del + &old_blank + &table
        } else {
            input
        };
        assert_body(&dom, body, &expected, obstruction);
        assert_eq!(id, if restores { 53 } else { 50 }, "{obstruction}");
        assert_eq!(
            semantic(&dom, source_root),
            source_before,
            "cloned source blanks remain authored and untracked"
        );
        let once = semantic(&dom, root);
        ensure_empty_pure_i_before_short_title_del(&mut dom, root, &templates, &settings, &mut id);
        assert_eq!(semantic(&dom, root), once, "repeat {obstruction}");
    }
}

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;

fn canonical(dom: &Dom, node: NodeId) -> String {
    if !dom.is_element(node) {
        return format!("text:{:?}", dom.text_value(node));
    }
    let mut attrs = dom
        .attributes_iter(node)
        .filter(|(name, _)| !dom.is_namespace_declaration(name))
        .map(|(name, value)| {
            (
                name.namespace_name().to_string(),
                name.local_name().to_string(),
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
            .map(|c| canonical(dom, c))
            .collect::<String>()
    )
}

type BodyHeuristic = fn(&mut Dom, NodeId);
type Configured = fn(&mut Dom, NodeId, &WmlComparerSettings, &mut u32);

/// The public body passes that take only the DOM and a root.
fn body_heuristics() -> Vec<(&'static str, BodyHeuristic)> {
    vec![
        (
            "unwrap_content_controls_in_pure_revisions",
            unwrap_content_controls_in_pure_revisions,
        ),
        (
            "fold_short_list_label_into_empty_pure_del",
            fold_short_list_label_into_empty_pure_del,
        ),
        (
            "strip_trailing_bare_empty_after_pure_i_dominant",
            strip_trailing_bare_empty_after_pure_i_dominant,
        ),
        (
            "promote_live_numpr_on_pure_d_from_pprchange",
            promote_live_numpr_on_pure_d_from_pprchange,
        ),
        (
            "strip_list_layout_from_mid_pure_del",
            strip_list_layout_from_mid_pure_del,
        ),
        (
            "free_mesh_shared_title_token_in_mix",
            free_mesh_shared_title_token_in_mix,
        ),
        (
            "strip_trailing_empty_pure_del_mark",
            strip_trailing_empty_pure_del_mark,
        ),
        (
            "strip_trailing_empty_pure_ins",
            strip_trailing_empty_pure_ins,
        ),
        (
            "relocate_title_page_last_empty_after_pure_dels",
            relocate_title_page_last_empty_after_pure_dels,
        ),
        (
            "strip_empty_pure_ins_before_trailing_pure_dels",
            strip_empty_pure_ins_before_trailing_pure_dels,
        ),
        (
            "fold_whitespace_pure_ins_into_following_pure_del",
            fold_whitespace_pure_ins_into_following_pure_del,
        ),
        (
            "rotate_ins_mark_del_only_paragraph",
            rotate_ins_mark_del_only_paragraph,
        ),
        (
            "restamp_stranded_del_mark_onto_del_only_paragraph",
            restamp_stranded_del_mark_onto_del_only_paragraph,
        ),
        (
            "split_head_short_title_long_del_mix",
            split_head_short_title_long_del_mix,
        ),
        (
            "fold_leading_ins_from_mix_into_preceding_pure_del",
            fold_leading_ins_from_mix_into_preceding_pure_del,
        ),
        (
            "strip_last_pure_del_mark_only_ppr",
            strip_last_pure_del_mark_only_ppr,
        ),
        (
            "cleanup_spacing_and_default_jc",
            cleanup_spacing_and_default_jc,
        ),
        (
            "last_pure_del_inherit_prev_jc",
            last_pure_del_inherit_prev_jc,
        ),
        (
            "strip_empty_pprchange_on_mix_with_live_jc",
            strip_empty_pprchange_on_mix_with_live_jc,
        ),
        ("strip_propertyless_ppr", strip_propertyless_ppr),
        (
            "align_word_table_and_comment_chrome",
            align_word_table_and_comment_chrome,
        ),
        (
            "promote_heading_spacing_from_pprchange_on_last_mix",
            promote_heading_spacing_from_pprchange_on_last_mix,
        ),
        (
            "strip_last_pure_del_mark_when_pprchange",
            strip_last_pure_del_mark_when_pprchange,
        ),
        (
            "interleave_list_cluster_after_coalesce",
            interleave_list_cluster_after_coalesce,
        ),
        ("ensure_default_page_size", ensure_default_page_size),
        (
            "splice_trailing_short_pure_dels_midstream",
            splice_trailing_short_pure_dels_midstream,
        ),
        (
            "split_digits_ins_from_mixed_title",
            split_digits_ins_from_mixed_title,
        ),
        (
            "peel_trailing_ins_from_mix_into_following_pure_del",
            peel_trailing_ins_from_mix_into_following_pure_del,
        ),
        (
            "restore_short_del_before_long_ins",
            restore_short_del_before_long_ins,
        ),
        (
            "peel_trailing_del_from_mix_into_following_pure_ins",
            peel_trailing_del_from_mix_into_following_pure_ins,
        ),
        (
            "restore_echoed_heading_deletions_before_table",
            restore_echoed_heading_deletions_before_table,
        ),
        (
            "strip_leading_del_echoing_prev_pure_i",
            strip_leading_del_echoing_prev_pure_i,
        ),
        ("free_mesh_wholesale_body_mix", free_mesh_wholesale_body_mix),
        ("free_mesh_bookended_ins_del", free_mesh_bookended_ins_del),
        (
            "peel_trailing_for_word_onto_next_mix",
            peel_trailing_for_word_onto_next_mix,
        ),
    ]
}

/// The public body passes that also take settings and an id counter.
fn configured_heuristics() -> Vec<(&'static str, Configured)> {
    vec![
        ("trailing_empty_spacing_to_pprchange", |d, r, s, id| {
            trailing_empty_spacing_to_pprchange(d, r, None, s, id);
        }),
        (
            "merged_closing_carrier_takes_source_properties",
            |d, r, s, id| {
                merged_closing_carrier_takes_source_properties(d, r, None, None, s, id);
            },
        ),
        ("closing_mark_takes_revised_properties", |d, r, s, id| {
            closing_mark_takes_revised_properties(d, r, None, None, (None, None), s, id);
        }),
        (
            "ensure_empty_pure_i_before_short_title_del",
            |d, r, s, id| {
                ensure_empty_pure_i_before_short_title_del(d, r, &HashMap::new(), s, id);
            },
        ),
        (
            "last_pure_del_spacing_to_pprchange",
            last_pure_del_spacing_to_pprchange,
        ),
        (
            "mixed_spacing_to_following_empty",
            mixed_spacing_to_following_empty,
        ),
        (
            "park_mixed_spacing_onto_trailing_pure_del",
            park_mixed_spacing_onto_trailing_pure_del,
        ),
        (
            "park_mixed_numpr_onto_trailing_empty_pure_del",
            park_mixed_numpr_onto_trailing_empty_pure_del,
        ),
        ("strip_unrecorded_word_defaults", |d, r, _, _| {
            strip_unrecorded_word_defaults(d, r, Some("276"));
        }),
        (
            "promote_live_jc_from_pprchange_on_body_mix",
            |d, r, _, _| promote_live_jc_from_pprchange_on_body_mix(d, r, &HashSet::new()),
        ),
        ("residual_short_label_zip_with_settings", |d, r, s, _| {
            residual_short_label_zip_with_settings(d, r, s);
        }),
        (
            "fold_midstream_demo_title_into_numbered_heading_with_source_properties",
            |d, r, s, id| {
                fold_midstream_demo_title_into_numbered_heading_with_source_properties(
                    d,
                    r,
                    s,
                    id,
                    &HashMap::new(),
                );
            },
        ),
        (
            "repair_borrowed_sentence_period",
            repair_borrowed_sentence_period,
        ),
        (
            "park_jc_on_first_short_title_mix_from_body",
            |d, r, s, id| {
                park_jc_on_first_short_title_mix_from_body(
                    d,
                    r,
                    &ParagraphPropertySources::default(),
                    s,
                    id,
                );
            },
        ),
        (
            "ensure_empty_pprchange_on_live_heading_spacing",
            ensure_empty_pprchange_on_live_heading_spacing,
        ),
        (
            "ensure_empty_pprchange_on_eq_with_live_jc",
            |d, r, s, id| {
                ensure_empty_pprchange_on_eq_with_live_jc(d, r, &HashSet::new(), s, id);
            },
        ),
    ]
}

// These are valid header/footer part roots accepted by the public DOM
// helpers, not body-pipeline fixtures. Body page-flow and final-pilcrow
// heuristics must not reinterpret their independent story boundaries.
#[test]
fn public_body_layout_heuristics_preserve_complete_independent_header_and_footer_parts() {
    let simple = body_heuristics();
    let configured = configured_heuristics();
    const FORMAT: &str = "<w:rPr><w:rFonts w:ascii='Georgia' w:hAnsi='Georgia'/><w:b/><w:color w:val='123456'/><w:sz w:val='20'/><w:lang w:val='en-US'/></w:rPr>";
    for story in ["hdr", "ftr"] {
        for tracked_formats in [false, true] {
            let date = "2026-01-02T03:04:05Z";
            let revision = |kind: &str, text: &str, id: u32| {
                format!(
                    "<w:{kind} w:id='{id}' w:author='Header editor' w:date='{date}'><w:r>{FORMAT}<w:{} xml:space='preserve'>{text}</w:{}></w:r></w:{kind}>",
                    if kind == "del" { "delText" } else { "t" },
                    if kind == "del" { "delText" } else { "t" }
                )
            };
            let props = "<w:pStyle w:val='Header'/><w:spacing w:before='120' w:after='60'/><w:ind w:left='360'/><w:jc w:val='right'/>";
            let ins = revision("ins", "Revised header title", 11);
            let del = revision("del", "Original header title", 12);
            let title = format!(
                "<w:p><w:pPr>{props}<w:rPr><w:ins w:id='13' w:author='Header editor' w:date='{date}'/></w:rPr></w:pPr>{ins}</w:p><w:p><w:pPr>{props}<w:rPr><w:del w:id='14' w:author='Header editor' w:date='{date}'/></w:rPr></w:pPr>{del}</w:p>"
            );
            let mixed = format!(
                "<w:p><w:pPr>{props}<w:pPrChange w:id='15' w:author='Prior editor' w:date='{date}'><w:pPr><w:spacing w:after='80'/><w:jc w:val='center'/></w:pPr></w:pPrChange></w:pPr>{}{}</w:p>",
                revision("ins", "Independent revised header continuation", 16),
                revision("del", "Independent original header continuation", 17)
            );
            let field = format!(
                "<w:p><w:pPr>{props}</w:pPr><w:bookmarkStart w:id='81' w:name='HeaderPage'/><w:r><w:fldChar w:fldCharType='begin'/></w:r><w:r><w:instrText xml:space='preserve'> PAGE </w:instrText></w:r><w:r><w:fldChar w:fldCharType='separate'/></w:r><w:r>{FORMAT}<w:t>17</w:t></w:r><w:r><w:fldChar w:fldCharType='end'/></w:r><w:bookmarkEnd w:id='81'/><w:hyperlink w:anchor='HeaderPage'><w:r>{FORMAT}<w:t>Owned page reference</w:t></w:r></w:hyperlink></w:p>"
            );
            let table = "<w:tbl><w:tblPr><w:tblW w:w='2400' w:type='dxa'/></w:tblPr><w:tblGrid><w:gridCol w:w='2400'/></w:tblGrid><w:tr><w:trPr><w:cantSplit/></w:trPr><w:tc><w:tcPr><w:tcW w:w='2400' w:type='dxa'/><w:shd w:fill='ABCDEF'/></w:tcPr><w:p><w:pPr><w:spacing w:after='40'/></w:pPr><w:r><w:rPr><w:i/></w:rPr><w:t>Independent header cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl>";
            let blanks = format!(
                "<w:p><w:pPr><w:spacing w:after='240'/><w:rPr><w:ins w:id='18' w:author='Header editor' w:date='{date}'/></w:rPr></w:pPr></w:p><w:p><w:pPr><w:spacing w:after='80'/><w:rPr><w:del w:id='19' w:author='Header editor' w:date='{date}'/></w:rPr></w:pPr></w:p><w:p/>"
            );
            let xml = format!(
                "<w:{story} xmlns:w='{}'>{title}{mixed}{field}{table}{blanks}</w:{story}>",
                W::URI
            );
            let mut dom = Dom::new();
            let document = dom.parse_xdocument(&xml);
            let root = dom.root(document).unwrap();
            let before = canonical(&dom, root);
            let count = dom.node_count();
            for (name, operation) in &simple {
                operation(&mut dom, root);
                assert_eq!(
                    canonical(&dom, root),
                    before,
                    "public {name} must preserve every {story} owner"
                );
                assert_eq!(
                    dom.node_count(),
                    count,
                    "{name} must not allocate body-only carriers in {story}"
                );
            }
            let settings = WmlComparerSettings {
                author_for_revisions: "Body comparer".to_string(),
                date_time_for_revisions: date.to_string(),
                detect_format_changes: tracked_formats,
                ..WmlComparerSettings::default()
            };
            let mut id = 100;
            for (name, operation) in &configured {
                operation(&mut dom, root, &settings, &mut id);
                assert_eq!(
                    canonical(&dom, root),
                    before,
                    "configured {name} must preserve every {story} owner"
                );
                assert_eq!(
                    dom.node_count(),
                    count,
                    "{name} must not allocate source history outside the body"
                );
                assert_eq!(
                    id, 100,
                    "{name} must not consume unrelated story revision IDs"
                );
            }
        }
    }
}

/// One element or text node per line, indented by depth, attributes
/// sorted: the form the pass snapshots diff.
fn outline(dom: &Dom, node: NodeId, depth: usize, out: &mut String) {
    let indent = "  ".repeat(depth);
    if !dom.is_element(node) {
        out.push_str(&format!("{indent}{:?}\n", dom.text_value(node)));
        return;
    }
    let short = |name: &XName| {
        if name.namespace_name() == W::URI {
            format!("w:{}", name.local_name())
        } else if name.namespace_name().is_empty() {
            name.local_name().to_string()
        } else {
            format!("{{{}}}{}", name.namespace_name(), name.local_name())
        }
    };
    let mut attrs = dom
        .attributes_iter(node)
        .filter(|(name, _)| !dom.is_namespace_declaration(name))
        .map(|(name, value)| format!("{}={value:?}", short(name)))
        .collect::<Vec<_>>();
    attrs.sort();
    let name = dom.name(node).map(|n| short(&n)).unwrap_or_default();
    out.push_str(&format!("{indent}{name} {}\n", attrs.join(" ")));
    for child in dom.nodes(node) {
        outline(dom, child, depth + 1, out);
    }
}

/// A body with the revision shapes the passes look for: pure inserted and
/// pure deleted titles, a mixed paragraph with a recorded property change,
/// a field, a table and trailing empty paragraphs.
fn synthetic_body() -> String {
    let date = "2026-01-02T03:04:05Z";
    let run = |kind: &str, text: &str, id: u32| {
        let tag = if kind == "del" { "delText" } else { "t" };
        format!(
            "<w:{kind} w:id='{id}' w:author='Editor' w:date='{date}'><w:r><w:rPr><w:b/></w:rPr><w:{tag} xml:space='preserve'>{text}</w:{tag}></w:r></w:{kind}>"
        )
    };
    let props = "<w:pStyle w:val='Heading1'/><w:spacing w:before='240' w:after='120' w:line='276'/><w:jc w:val='left'/>";
    format!(
        "<w:document xmlns:w='{}'><w:body>\
             <w:p><w:pPr>{props}<w:rPr><w:ins w:id='1' w:author='Editor' w:date='{date}'/></w:rPr></w:pPr>{}</w:p>\
             <w:p><w:pPr>{props}<w:rPr><w:del w:id='2' w:author='Editor' w:date='{date}'/></w:rPr></w:pPr>{}</w:p>\
             <w:p><w:pPr><w:spacing w:after='200'/><w:pPrChange w:id='3' w:author='Prior' w:date='{date}'><w:pPr><w:spacing w:after='200'/><w:jc w:val='center'/></w:pPr></w:pPrChange></w:pPr><w:r><w:t xml:space='preserve'>Kept text, </w:t></w:r>{}{}</w:p>\
             <w:p><w:numPr><w:ilvl w:val='0'/><w:numId w:val='1'/></w:numPr><w:r><w:t>List item</w:t></w:r>{}</w:p>\
             <w:p><w:r><w:fldChar w:fldCharType='begin'/></w:r><w:r><w:instrText xml:space='preserve'> PAGE </w:instrText></w:r><w:r><w:fldChar w:fldCharType='separate'/></w:r><w:r><w:t>1</w:t></w:r><w:r><w:fldChar w:fldCharType='end'/></w:r></w:p>\
             <w:tbl><w:tblPr><w:tblW w:w='2400' w:type='dxa'/></w:tblPr><w:tblGrid><w:gridCol w:w='2400'/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w='2400' w:type='dxa'/></w:tcPr><w:p>{}</w:p></w:tc></w:tr></w:tbl>\
             <w:p><w:pPr><w:spacing w:after='240'/><w:rPr><w:ins w:id='9' w:author='Editor' w:date='{date}'/></w:rPr></w:pPr></w:p>\
             <w:p><w:pPr><w:rPr><w:del w:id='10' w:author='Editor' w:date='{date}'/></w:rPr></w:pPr>{}</w:p>\
             <w:p/><w:sectPr><w:pgSz w:w='11906' w:h='16838'/></w:sectPr>\
             </w:body></w:document>",
        W::URI,
        run("ins", "Revised title", 4),
        run("del", "Original title", 5),
        run("ins", "added words", 6),
        run("del", "removed words", 7),
        run("ins", " grows", 8),
        run("del", "Cell text", 11),
        run("del", "Closing words", 12),
    )
}

/// The main document part of a fixture package, as XML text.
fn document_xml(docx: &[u8]) -> String {
    let pkg = crate::opc::PartFs::open(docx).expect("fixture opens");
    let main = pkg.main_document_part().expect("fixture has a main part");
    pkg.part_string(&main).expect("main part is text")
}

/// What every public body pass does to `xml`: `unchanged`, or a unified
/// diff of the outline before and after it (and the ids it took).
fn pass_report(xml: &str) -> String {
    let settings = WmlComparerSettings {
        author_for_revisions: "Snapshot".to_string(),
        date_time_for_revisions: "2026-01-02T03:04:05Z".to_string(),
        ..WmlComparerSettings::default()
    };
    let parsed = || {
        let mut dom = Dom::new();
        let document = dom.parse_xdocument(xml);
        let root = dom.root(document).expect("document element");
        (dom, root)
    };
    let before = {
        let (dom, root) = parsed();
        let mut out = String::new();
        outline(&dom, root, 0, &mut out);
        out
    };
    let mut report = String::new();
    let mut record = |name: &str, dom: &Dom, root: NodeId, ids: Option<(u32, u32)>| {
        let mut after = String::new();
        outline(dom, root, 0, &mut after);
        report.push_str(&format!("## {name}\n"));
        if let Some((from, to)) = ids.filter(|(from, to)| from != to) {
            report.push_str(&format!("ids {from} -> {to}\n"));
        }
        if after == before {
            report.push_str("unchanged\n");
        } else {
            let diff = similar::TextDiff::from_lines(&before, &after);
            report.push_str(
                &diff
                    .unified_diff()
                    .context_radius(1)
                    .header("before", "after")
                    .to_string(),
            );
        }
    };
    for (name, pass) in body_heuristics() {
        let (mut dom, root) = parsed();
        pass(&mut dom, root);
        record(name, &dom, root, None);
    }
    for (name, pass) in configured_heuristics() {
        let (mut dom, root) = parsed();
        let mut id = 1000;
        pass(&mut dom, root, &settings, &mut id);
        record(name, &dom, root, Some((1000, id)));
    }
    report
}

/// Snapshot of every public body pass on a small corpus: two of Word's
/// own redlines and a synthetic body. A pass whose output changes fails
/// here until the new snapshot is reviewed and blessed with
/// `JUBARTE_BLESS=1 cargo nextest run -E 'test(finalize_pass_snapshots)'`.
#[test]
fn finalize_pass_snapshots() {
    let corpus = [
        (
            "f1-word-redline",
            document_xml(include_bytes!(
                "../../../../../../tests/fixtures/f1/word-redline.docx"
            )),
        ),
        (
            "f4-word-redline",
            document_xml(include_bytes!(
                "../../../../../../tests/fixtures/f4/word-redline.docx"
            )),
        ),
        ("synthetic-body", synthetic_body()),
    ];
    // The committed snapshots stay beside the root package's tests;
    // this crate sits two directories below the repository root.
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/finalize-snapshots");
    let bless = std::env::var_os("JUBARTE_BLESS").is_some();
    for (name, xml) in corpus {
        let actual = format!(
            "# Every public comparer::finalize body pass on {name}: `unchanged`, or the\n\
                 # outline diff it makes. Regenerate after review with\n\
                 # JUBARTE_BLESS=1 cargo nextest run -E 'test(finalize_pass_snapshots)'\n{}",
            pass_report(&xml)
        );
        let path = dir.join(format!("{name}.txt"));
        if bless {
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(&path, &actual).unwrap();
            continue;
        }
        let expected = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("{} is missing; run with JUBARTE_BLESS=1", path.display()));
        if expected != actual {
            let diff = similar::TextDiff::from_lines(&expected, &actual)
                .unified_diff()
                .context_radius(2)
                .header("blessed", "now")
                .to_string();
            let shown: String = diff.chars().take(6000).collect();
            panic!(
                "{name}: a finalize pass changed its output. Review, then bless with \
                     JUBARTE_BLESS=1.\n{shown}"
            );
        }
    }
}

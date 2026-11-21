// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;

fn sheet() -> StyleSheet {
    StyleSheet {
        defaults: Defaults::word(),
        by_id: HashMap::new(),
        tables: HashMap::new(),
        theme: ThemeFonts::default(),
        latent: true,
        default_table: true,
    }
}

fn source(local: &str, paragraphs: &str) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let document = dom.parse_xdocument(&format!(
        "<w:{local} xmlns:w='{}'>{paragraphs}</w:{local}>",
        W::URI
    ));
    let root = dom.root(document).expect("literal chrome story");
    (dom, root)
}

fn collected_without_mutation(dom: &Dom, root: NodeId) -> Vec<TextRun> {
    let before = dom.serialize_element(root);
    let nodes = dom.node_count();
    let runs = collect_hf_runs(dom, root, &sheet(), 400.0, None);
    assert_eq!(dom.serialize_element(root), before);
    assert_eq!(dom.node_count(), nodes);
    let repeat = collect_hf_runs(dom, root, &sheet(), 400.0, None);
    assert_eq!(
        runs.iter().map(|r| r.text.as_str()).collect::<Vec<_>>(),
        repeat.iter().map(|r| r.text.as_str()).collect::<Vec<_>>()
    );
    assert_eq!(runs.len(), repeat.len());
    for (a, b) in runs.iter().zip(&repeat) {
        assert_eq!(
            (
                a.style.size,
                a.style.color,
                a.style.bold,
                a.frame_right,
                a.frame_center,
                a.rev,
                a.ends_line,
                a.para_gap
            ),
            (
                b.style.size,
                b.style.color,
                b.style.bold,
                b.frame_right,
                b.frame_center,
                b.rev,
                b.ends_line,
                b.para_gap
            )
        );
        assert!(a.field == b.field);
    }
    runs
}

#[test]
fn real_chrome_frames_keep_source_styles_while_waiting_for_empty_or_live_lines() {
    for local in ["hdr", "ftr"] {
        for align in ["right", "center"] {
            for destination in ["empty", "text", "end"] {
                let frame = format!(
                    "<w:p><w:pPr><w:framePr w:hAnchor='margin' w:vAnchor='text' w:xAlign='{align}'/></w:pPr><w:r><w:rPr><w:rFonts w:ascii='Calibri' w:hAnsi='Calibri'/><w:b/><w:color w:val='123456'/><w:sz w:val='40'/></w:rPr><w:t>Framed owner</w:t></w:r></w:p>"
                );
                let follow = match destination {
                    "empty" => "<w:p/>",
                    "text" => {
                        "<w:p><w:r><w:rPr><w:sz w:val='28'/></w:rPr><w:t>Following owner</w:t></w:r></w:p>"
                    }
                    _ => "",
                };
                let (dom, root) = source(local, &(frame + follow));
                let runs = collected_without_mutation(&dom, root);
                let expected = if destination == "text" {
                    vec!["Following owner", "Framed owner"]
                } else {
                    vec!["Framed owner"]
                };
                assert_eq!(
                    runs.iter().map(|r| r.text.as_str()).collect::<Vec<_>>(),
                    expected,
                    "{local}/{align}/{destination}"
                );
                let framed = runs.last().expect("authored frame text");
                assert_eq!(framed.style.size, 20.0);
                assert_eq!(
                    framed.style.color,
                    [18.0 / 255.0, 52.0 / 255.0, 86.0 / 255.0]
                );
                assert!(framed.style.bold);
                assert_eq!(
                    (framed.frame_right, framed.frame_center),
                    (align == "right", align == "center")
                );
                assert!(!framed.rev);
                assert!(framed.field == FieldKind::None);
                assert_eq!(framed.hf_para.is_some(), destination != "end");
                if destination == "text" {
                    assert_eq!(runs[0].style.size, 14.0);
                    assert!(!runs[0].frame_right && !runs[0].frame_center);
                }
            }
        }
    }
}

#[test]
fn uncached_chrome_fields_are_real_line_content_for_following_left_absolute_tabs() {
    for local in ["hdr", "ftr"] {
        for (name, kind) in [
            ("PAGE", FieldKind::Page),
            ("NUMPAGES", FieldKind::NumPages),
            ("NUMWORDS", FieldKind::NumWords),
        ] {
            for alignment in ["", " w:alignment='left'"] {
                let xml = format!(
                    "<w:p><w:r><w:fldChar w:fldCharType='begin'/></w:r><w:r><w:instrText xml:space='preserve'> {name} </w:instrText></w:r><w:r><w:fldChar w:fldCharType='separate'/></w:r><w:r><w:rPr><w:sz w:val='30'/></w:rPr><w:fldChar w:fldCharType='end'/></w:r><w:r>\n<w:rPr><w:rFonts w:ascii='Calibri'/><w:color w:val='234567'/><w:sz w:val='24'/></w:rPr>\n<w:ptab{alignment} w:relativeTo='margin'/>\n<w:t>Following label</w:t>\n</w:r></w:p>"
                );
                let (dom, root) = source(local, &xml);
                let runs = collected_without_mutation(&dom, root);
                assert_eq!(
                    runs.iter().map(|r| r.text.as_str()).collect::<Vec<_>>(),
                    ["", "\n", "Following label"],
                    "{local}/{name}/{alignment}"
                );
                assert!(runs[0].field == kind);
                assert_eq!(runs[0].style.size, 15.0);
                assert!(runs[1].field == FieldKind::None && runs[2].field == FieldKind::None);
                assert_eq!(runs[1].style.size, 12.0);
                assert_eq!(runs[2].style.size, 12.0);
                assert_eq!(
                    runs[2].style.color,
                    [35.0 / 255.0, 69.0 / 255.0, 103.0 / 255.0]
                );
                assert!(
                    runs.iter()
                        .all(|r| !r.rev && !r.frame_center && !r.frame_right)
                );
            }
        }
    }
}

#[test]
fn stacked_contextual_empty_chrome_lines_keep_their_authored_mark_sizes_and_gap_owners() {
    for local in ["hdr", "ftr"] {
        for contextual in [false, true] {
            for before in [80, 400] {
                let context = if contextual {
                    "<w:contextualSpacing/>"
                } else {
                    ""
                };
                let xml = format!(
                    "<w:p><w:pPr><w:spacing w:before='100' w:after='200'/>{context}<w:rPr><w:sz w:val='32'/></w:rPr></w:pPr></w:p><w:p><w:pPr><w:spacing w:before='{before}' w:after='60'/>{context}<w:rPr><w:sz w:val='24'/></w:rPr></w:pPr></w:p>"
                );
                let (dom, root) = source(local, &xml);
                let runs = collected_without_mutation(&dom, root);
                assert_eq!(
                    runs.iter().map(|r| r.text.as_str()).collect::<Vec<_>>(),
                    [HF_LINE_BREAK, HF_LINE_BREAK]
                );
                assert_eq!((runs[0].style.size, runs[1].style.size), (16.0, 12.0));
                assert_eq!(runs[0].para_gap, 15.0);
                let second = if contextual || before == 80 {
                    3.0
                } else {
                    13.0
                };
                assert_eq!(
                    runs[1].para_gap, second,
                    "{local}/context={contextual}/before={before}"
                );
                assert!(runs.iter().all(|r| r.hf_para.is_some() && !r.rev));
            }
        }
    }
}

#[test]
fn chrome_visible_source_excludes_original_move_streams_and_keeps_real_page_break_lines() {
    for local in ["hdr", "ftr"] {
        for deleted in ["del", "moveFrom"] {
            for preserve in [false, true] {
                for page in [false, true] {
                    let (dom, root) = source(
                        local,
                        &format!(
                            "<w:p><w:r><w:t xml:space='preserve'> </w:t></w:r><w:r><w:br{}/></w:r><w:{deleted} w:id='17' w:author='Original owner' w:date='2025-01-01T00:00:00Z'><w:r><w:delText>Original cache must not become a live header label</w:delText></w:r></w:{deleted}><w:ins w:id='18' w:author='Revised owner' w:date='2026-01-01T00:00:00Z'><w:r><w:t>Owned label</w:t></w:r></w:ins></w:p>",
                            if page { " w:type='page'" } else { "" }
                        ),
                    );
                    let before = dom.serialize_element(root);
                    let nodes = dom.node_count();
                    let shown = visible_text(&dom, root, RevMark::None, preserve);
                    assert_eq!(
                        shown,
                        if preserve {
                            " \nOwned label"
                        } else {
                            "\nOwned label"
                        }
                    );
                    assert_eq!(visible_text(&dom, root, RevMark::None, preserve), shown);
                    assert_eq!(dom.serialize_element(root), before);
                    assert_eq!(dom.node_count(), nodes);
                }
            }
        }
    }
}

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;

fn dom(fragment: &str) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let doc=dom.parse_xdocument(&format!(r#"<w:root xmlns:w="{}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:wpg="http://schemas.microsoft.com/office/word/2010/wordprocessingGroup" xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape" xmlns:wpc="http://schemas.microsoft.com/office/word/2010/wordprocessingCanvas" xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture">{fragment}</w:root>"#,W::URI));
    let root = dom.root(doc).unwrap();
    (dom, root)
}

fn fonts() -> Fonts<'static> {
    let mut fonts = Fonts::new();
    fonts.insert_embedded("Memory Mono", false, false, FaceId::MonoRegular.bytes());
    fonts
}
fn run(text: &str) -> TextRun {
    let mut style = Defaults::word().run;
    style.family = "Memory Mono".into();
    style.size = 12.0;
    TextRun::new(text, style)
}
fn text(lines: &[Vec<TextRun>]) -> Vec<String> {
    lines
        .iter()
        .map(|line| line.iter().map(|r| r.text.as_str()).collect())
        .collect()
}
fn mono_em() -> f32 {
    let raw = ttf_parser::Face::parse(FaceId::MonoRegular.bytes(), 0).unwrap();
    f32::from(
        raw.glyph_hor_advance(raw.glyph_index('1').unwrap())
            .unwrap(),
    ) * 12.0
        / f32::from(raw.units_per_em())
}

#[test]
fn character_wrapping_retains_each_page_reference_bookmark_and_note_owner() {
    let fonts = fonts();
    let width = 2.1 * mono_em();
    for owner in ["plain", "pageref", "ref", "footnote"] {
        let mut source = run("1234");
        match owner {
            "pageref" => source.pageref = Some("OwnedPage".into()),
            "ref" => source.ref_name = Some("OwnedReference".into()),
            "footnote" => source.footnote_id = Some("OwnedNote".into()),
            _ => {}
        }
        let lines = wrap_runs_segment(
            &fonts,
            std::slice::from_ref(&source),
            width,
            width,
            false,
            None,
            LineFit {
                char_break: true,
                ..LineFit::default()
            },
        );
        assert_eq!(text(&lines), ["12", "34"]);
        assert_eq!(
            lines
                .iter()
                .flatten()
                .map(|r| r.text.as_str())
                .collect::<String>(),
            source.text
        );
        let expected_runs = if owner == "plain" { 2 } else { 4 };
        assert_eq!(lines.iter().flatten().count(), expected_runs);
        for r in lines.iter().flatten() {
            assert_eq!(
                (&r.pageref, &r.ref_name, &r.footnote_id),
                (&source.pageref, &source.ref_name, &source.footnote_id)
            );
            assert!(style_eq(&r.style, &source.style));
            assert!(r.field == FieldKind::None);
        }
    }
}

#[test]
fn wrapping_does_not_merge_different_field_kinds_or_comment_marker_ownership() {
    let fonts = fonts();
    for owner in ["field", "comment-marker", "character-style", "styleref"] {
        let a = run("12");
        let mut b = run("34");
        match owner {
            "field" => b.field = FieldKind::Page,
            "comment-marker" => b.comment_marker = true,
            "character-style" => b.char_style = Some(std::rc::Rc::from("OwnedChar")),
            "styleref" => {
                b.styleref = Some(std::rc::Rc::new(StyleRef {
                    id: "OwnedHeading".into(),
                    last: false,
                }));
            }
            _ => unreachable!(),
        }
        let lines = wrap_runs_segment(
            &fonts,
            &[a.clone(), b.clone()],
            100.0,
            100.0,
            false,
            None,
            LineFit::default(),
        );
        assert_eq!(text(&lines), ["1234"]);
        assert_eq!(lines[0].len(), 2, "owner={owner}");
        for (actual, source) in lines[0].iter().zip([&a, &b]) {
            assert_eq!(actual.text, source.text);
            assert!(actual.field == source.field);
            assert_eq!(actual.comment_marker, source.comment_marker);
            assert_eq!(actual.char_style, source.char_style);
            assert_eq!(actual.styleref, source.styleref);
            assert!(style_eq(&actual.style, &source.style));
        }
    }
}

#[test]
fn closing_punctuation_hangs_with_its_letter_but_cannot_exceed_a_narrower_than_glyph_line() {
    let fonts = fonts();
    let em = mono_em();
    for (width, expected) in [
        (1.1 * em, vec!["1’", "2’"]),
        (0.5 * em, vec!["1", "’", "2", "’"]),
    ] {
        let source = run("1’2’");
        let lines = wrap_runs_segment(
            &fonts,
            std::slice::from_ref(&source),
            width,
            width,
            false,
            None,
            LineFit {
                char_break: true,
                ..LineFit::default()
            },
        );
        assert_eq!(text(&lines), expected);
        assert_eq!(
            lines
                .iter()
                .flatten()
                .map(|r| r.text.as_str())
                .collect::<String>(),
            source.text
        );
        assert!(
            lines
                .iter()
                .flatten()
                .all(|r| style_eq(&r.style, &source.style))
        );
    }
}

#[test]
fn hanging_spaces_keep_their_authored_span_while_legacy_narrow_cells_stack_them() {
    let fonts = fonts();
    let width = 0.5 * mono_em();
    for (hang, expected) in [(false, vec!["1", "  ", "2"]), (true, vec!["1  ", "2"])] {
        let source = run("1  2");
        let lines = wrap_runs_segment(
            &fonts,
            std::slice::from_ref(&source),
            width,
            width,
            false,
            None,
            LineFit {
                char_break: true,
                hang_spaces: hang,
                ..LineFit::default()
            },
        );
        assert_eq!(text(&lines), expected);
        assert_eq!(
            lines
                .iter()
                .flatten()
                .map(|r| r.text.as_str())
                .collect::<String>(),
            source.text
        );
    }
}

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
fn word_mode(test: impl FnOnce()) {
    struct Restore(RevisionStyle);
    impl Drop for Restore {
        fn drop(&mut self) {
            REVISIONS.with(|v| v.set(self.0));
        }
    }
    let _restore = Restore(REVISIONS.with(|v| v.replace(RevisionStyle::Word)));
    test();
}

#[test]
fn revised_header_fields_keep_owned_cache_or_empty_placeholders_and_complete_run_style() {
    word_mode(|| {
        let sheet = sheet();
        for (tag, kind) in [
            ("PAGE", FieldKind::Page),
            ("NUMPAGES", FieldKind::NumPages),
            ("NUMWORDS", FieldKind::NumWords),
        ] {
            for (wrapper, deleted) in [("ins", false), ("del", true)] {
                for cache in ["", "IV"] {
                    let text_tag = if deleted { "delText" } else { "t" };
                    let instr_tag = if deleted { "delInstrText" } else { "instrText" };
                    let cached = if cache.is_empty() {
                        String::new()
                    } else {
                        format!(
                            "<w:r><w:rPr><w:color w:val='008000'/><w:b/><w:sz w:val='24'/></w:rPr><w:{text_tag}>{cache}</w:{text_tag}></w:r>"
                        )
                    };
                    let source = format!(
                        "<w:p><w:{wrapper} w:id='1' w:author='Owner' w:date='2001-02-03T04:05:06Z'><w:r><w:rPr><w:color w:val='008000'/><w:b/><w:sz w:val='24'/></w:rPr><w:fldChar w:fldCharType='begin'/><w:{instr_tag}> {tag} </w:{instr_tag}><w:fldChar w:fldCharType='separate'/></w:r>{cached}<w:r><w:rPr><w:color w:val='008000'/><w:b/><w:sz w:val='24'/></w:rPr><w:fldChar w:fldCharType='end'/></w:r></w:{wrapper}></w:p>"
                    );
                    let (dom, root) = dom(&source);
                    let unchanged = dom.serialize_element(root);
                    let mut runs = Vec::new();
                    collect_hf_rev(
                        &dom,
                        root,
                        &sheet.defaults.run,
                        &sheet,
                        &mut FieldScan::default(),
                        &mut runs,
                        RevMark::None,
                    );
                    assert_eq!(runs.len(), 1, "field {tag}/{wrapper}/cache={cache:?}");
                    let r = &runs[0];
                    assert_eq!(r.text, cache);
                    assert!(r.field == kind);
                    assert!(r.rev);
                    assert_eq!(r.style.color, [0.0, 128.0 / 255.0, 0.0]);
                    assert!(r.style.bold);
                    assert_eq!(r.style.size, 12.0);
                    assert_eq!(r.style.strike, deleted);
                    assert!(!r.style.underline);
                    assert!(r.pageref.is_none() && r.ref_name.is_none() && r.styleref.is_none());
                    assert_eq!(dom.serialize_element(root), unchanged);
                }
            }
        }
    });
}

#[test]
fn revised_field_runs_skip_drawing_metadata_and_binary_field_data_without_leaking_text() {
    word_mode(|| {
        let sheet = sheet();
        let (dom, root) = dom(
            "<w:p><w:ins w:id='1' w:author='Owner' w:date='2001-02-03T04:05:06Z'><w:r><w:fldChar w:fldCharType='begin'><w:fldData>BASE64_METADATA</w:fldData></w:fldChar><w:instrText> UNHANDLED </w:instrText><w:drawing><wp:docPr id='4' name='NOT_VISIBLE'/></w:drawing><w:fldChar w:fldCharType='end'/></w:r><w:r><w:t>Owned visible text</w:t></w:r></w:ins></w:p>",
        );
        let unchanged = dom.serialize_element(root);
        let mut runs = Vec::new();
        collect_hf_rev(
            &dom,
            root,
            &sheet.defaults.run,
            &sheet,
            &mut FieldScan::default(),
            &mut runs,
            RevMark::None,
        );
        assert_eq!(
            runs.iter().map(|r| r.text.as_str()).collect::<String>(),
            "Owned visible text"
        );
        assert_eq!(runs.len(), 1);
        assert!(runs[0].rev && runs[0].style.underline);
        assert_eq!(runs[0].style.color, HF_REV_COLOR);
        assert!(runs[0].field == FieldKind::None);
        assert_eq!(dom.serialize_element(root), unchanged);
    });
}

#[test]
fn revised_left_position_tabs_after_field_cache_keep_field_scan_and_source_style() {
    word_mode(|| {
        let sheet = sheet();
        let (dom, root) = dom(
            "<w:p><w:ins w:id='1' w:author='Owner' w:date='2001-02-03T04:05:06Z'><w:r><w:fldChar w:fldCharType='begin'/><w:instrText> UNHANDLED </w:instrText><w:fldChar w:fldCharType='separate'/></w:r><w:r><w:rPr><w:b/></w:rPr><w:t>One</w:t><w:ptab w:alignment='left' w:relativeTo='margin' w:leader='none'/><w:t>Two</w:t></w:r><w:r><w:fldChar w:fldCharType='end'/></w:r></w:ins></w:p>",
        );
        let unchanged = dom.serialize_element(root);
        let mut runs = Vec::new();
        collect_hf_rev(
            &dom,
            root,
            &sheet.defaults.run,
            &sheet,
            &mut FieldScan::default(),
            &mut runs,
            RevMark::None,
        );
        assert_eq!(
            runs.iter().map(|r| r.text.as_str()).collect::<String>(),
            "One\nTwo"
        );
        assert_eq!(runs.len(), 3);
        assert!(
            runs.iter()
                .all(|r| r.rev && r.style.bold && r.style.underline && r.field == FieldKind::None)
        );
        assert_eq!(dom.serialize_element(root), unchanged);
    });
}

fn grouped_shape(text: &str, vert: &str) -> String {
    format!(
        "<wps:wsp><wps:spPr><a:xfrm flipH='1' flipV='1'><a:off x='127000' y='63500'/><a:ext cx='127000' cy='127000'/></a:xfrm><a:solidFill><a:srgbClr val='00FF00'/></a:solidFill><a:prstGeom prst='rect'/></wps:spPr><wps:txbx><w:txbxContent><w:p><w:pPr><w:ind w:left='180'/><w:spacing w:before='40'/></w:pPr><w:r><w:rPr><w:b/><w:color w:val='008000'/></w:rPr><w:t>{text}</w:t></w:r></w:p></w:txbxContent></wps:txbx><wps:bodyPr vert='{vert}' anchor='ctr' lIns='25400' tIns='12700' rIns='38100' bIns='50800'/></wps:wsp>"
    )
}

#[test]
fn drawing_canvas_and_explicit_group_frames_keep_identical_child_geometry_and_owned_text() {
    let sheet = sheet();
    for canvas in [false, true] {
        for paragraphs in [false, true] {
            for vert in ["horz", "vert"] {
                let child = grouped_shape("Owned label", vert);
                let group = if canvas {
                    format!("<wpc:wpc>{child}</wpc:wpc>")
                } else {
                    format!(
                        "<wpg:wgp><wpg:grpSpPr><a:xfrm><a:off x='0' y='0'/><a:ext cx='508000' cy='254000'/><a:chOff x='0' y='0'/><a:chExt cx='508000' cy='254000'/></a:xfrm></wpg:grpSpPr>{child}</wpg:wgp>"
                    )
                };
                let (dom, root) = dom(&format!(
                    "<w:p><w:r><w:drawing><wp:inline><wp:extent cx='508000' cy='254000'/><wp:docPr id='1' name='Owned group'/><a:graphic><a:graphicData>{group}</a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>"
                ));
                let drawing = first_named(&dom, root, "drawing").unwrap();
                let unchanged = dom.serialize_element(root);
                let ctx = GroupText {
                    theme: &sheet.theme,
                    base: &sheet.defaults.run,
                    sheet: paragraphs.then_some(&sheet),
                };
                let children = group_children(&dom, drawing, &ctx);
                assert_eq!(children.len(), 1);
                let child = &children[0];
                assert_eq!(child.frac, [0.25, 0.25, 0.25, 0.5]);
                let shape = &child.shape;
                assert_eq!(
                    (shape.w, shape.h, shape.text_dx, shape.text_dy),
                    (10.0, 10.0, 9.0, 2.0)
                );
                assert_eq!(shape.fill, Some([0.0, 1.0, 0.0]));
                assert!(shape.line.is_none() && !shape.stroke && shape.flip_h && shape.flip_v);
                assert_eq!(shape.insets, [2.0, 1.0, 3.0, 4.0]);
                assert!(matches!(shape.text_anchor, TextAnchor::Center));
                assert_eq!(
                    shape
                        .runs
                        .iter()
                        .map(|r| r.text.as_str())
                        .collect::<String>(),
                    "Owned label"
                );
                assert!(
                    shape
                        .runs
                        .iter()
                        .all(|r| r.style.bold && r.style.color == [0.0, 128.0 / 255.0, 0.0])
                );
                let stacked = paragraphs && vert == "horz";
                assert_eq!(shape.paras.len(), usize::from(stacked));
                assert!(shape.tables.is_empty());
                if stacked {
                    assert_eq!(
                        shape.paras[0]
                            .0
                            .iter()
                            .map(|r| r.text.as_str())
                            .collect::<String>(),
                        "Owned label"
                    );
                    assert_eq!(shape.paras[0].1.indent_left, 9.0);
                }
                assert!(group_pictures(&dom, drawing).is_empty());
                assert_eq!(dom.serialize_element(root), unchanged);
            }
        }
    }
}

#[test]
fn group_picture_members_keep_source_order_and_their_own_transform_boxes() {
    let (dom, root) = dom(
        "<w:drawing><wp:inline><wp:extent cx='508000' cy='254000'/><a:graphic><a:graphicData><wpc:wpc><pic:pic><pic:spPr><a:xfrm><a:off x='0' y='0'/><a:ext cx='127000' cy='63500'/></a:xfrm></pic:spPr></pic:pic><pic:pic><pic:spPr><a:xfrm><a:off x='254000' y='127000'/><a:ext cx='254000' cy='127000'/></a:xfrm></pic:spPr></pic:pic></wpc:wpc></a:graphicData></a:graphic></wp:inline></w:drawing>",
    );
    let drawing = first_named(&dom, root, "drawing").unwrap();
    let unchanged = dom.serialize_element(root);
    let nodes = descendants_local(&dom, drawing, "pic");
    let pictures = group_pictures(&dom, drawing);
    assert_eq!(
        pictures,
        vec![
            ([0.0, 0.0, 0.25, 0.25], nodes[0]),
            ([0.5, 0.5, 0.5, 0.5], nodes[1])
        ]
    );
    assert_eq!(dom.serialize_element(root), unchanged);
}

#[test]
fn grouped_empty_text_keeps_flat_source_without_inventing_stacked_paragraphs() {
    let sheet = sheet();
    let (dom, root) = dom(&format!(
        "<w:drawing><wp:inline><wp:extent cx='508000' cy='254000'/><a:graphic><a:graphicData><wpc:wpc>{}</wpc:wpc></a:graphicData></a:graphic></wp:inline></w:drawing>",
        grouped_shape(" ", "horz")
    ));
    let drawing = first_named(&dom, root, "drawing").unwrap();
    let ctx = GroupText {
        theme: &sheet.theme,
        base: &sheet.defaults.run,
        sheet: Some(&sheet),
    };
    let unchanged = dom.serialize_element(root);
    let children = group_children(&dom, drawing, &ctx);
    assert_eq!(children.len(), 1);
    assert!(children[0].shape.paras.is_empty());
    assert_eq!(
        children[0]
            .shape
            .runs
            .iter()
            .map(|r| r.text.as_str())
            .collect::<String>(),
        " "
    );
    assert_eq!(children[0].frac, [0.25, 0.25, 0.25, 0.5]);
    assert_eq!(dom.serialize_element(root), unchanged);
}

#[test]
fn intentionally_incomplete_custom_beziers_do_not_drop_later_valid_path_commands() {
    for broken in [
        "<a:cubicBezTo><a:pt x='1' y='2'/><a:pt x='3' y='4'/></a:cubicBezTo>",
        "<a:quadBezTo><a:pt x='1' y='2'/></a:quadBezTo>",
    ] {
        let (dom, root) = dom(&format!(
            "<wps:wsp><wps:spPr><a:xfrm><a:ext cx='254000' cy='127000'/></a:xfrm><a:custGeom><a:avLst/><a:gdLst/><a:pathLst><a:path w='20' h='10' fill='none' stroke='true'><a:moveTo><a:pt x='0' y='0'/></a:moveTo>{broken}<a:lnTo><a:pt x='20' y='10'/></a:lnTo></a:path></a:pathLst></a:custGeom></wps:spPr></wps:wsp>"
        ));
        let unchanged = dom.serialize_element(root);
        let custom = cust_geom(&dom, root).unwrap();
        assert!(custom.av.is_empty() && custom.gd.is_empty());
        assert_eq!(custom.paths.len(), 1);
        assert_eq!(
            (
                custom.paths[0].w,
                custom.paths[0].h,
                custom.paths[0].fill,
                custom.paths[0].stroke
            ),
            (20.0, 10.0, preset_geom::Fill::None, true)
        );
        assert_eq!(custom.paths[0].cmds.len(), 2);
        assert!(matches!(&custom.paths[0].cmds[0],preset_geom::OwnedCmd::M(x,y) if x=="0"&&y=="0"));
        assert!(
            matches!(&custom.paths[0].cmds[1],preset_geom::OwnedCmd::L(x,y) if x=="20"&&y=="10")
        );
        let paths = preset_geom::evaluate_custom(&custom, 40.0, 20.0);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].subpaths.len(), 1);
        assert_eq!(paths[0].subpaths[0].pts, vec![(0.0, 0.0), (40.0, 20.0)]);
        assert!(!paths[0].subpaths[0].closed);
        assert_eq!(dom.serialize_element(root), unchanged);
    }
}

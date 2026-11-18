// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;

fn dom(fragment: &str) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&format!(r#"<w:root xmlns:w="{}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:wp14="http://schemas.microsoft.com/office/word/2010/wordprocessingDrawing" xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">{fragment}</w:root>"#, W::URI));
    let root = dom.root(doc).unwrap();
    (dom, root)
}
fn fonts() -> Fonts<'static> {
    let mut fonts = Fonts::new();
    fonts.insert_embedded("Memory Mono", false, false, FaceId::MonoRegular.bytes());
    fonts
}
fn para(text: &str) -> CellPara {
    let mut run = Defaults::word().run;
    run.family = "Memory Mono".into();
    run.size = 12.0;
    let mut style = Defaults::word().para;
    style.before = 2.0;
    style.after = 3.0;
    style.line_exact = Some(20.0);
    style.indent_left = 0.0;
    style.indent_right = 0.0;
    style.indent_first = 0.0;
    CellPara {
        runs: vec![TextRun::new(text, run)],
        images: Vec::new(),
        boxes: Vec::new(),
        style,
        bookmarks: vec!["OwnedCell".into()],
        blank_bookmarks: vec!["EmptyCell".into()],
        continued: false,
        vertical: false,
        squeeze: Squeeze::NONE,
        hang_spaces: false,
    }
}
fn picture(w: f32, h: f32) -> LaidImage {
    // Parse real shape defaults, then use the production image wrapper.
    let (dom, root) = dom(
        "<w:p><w:r><w:drawing><wp:inline><wp:extent cx='254000' cy='254000'/><a:graphic><a:graphicData><wps:wsp><wps:spPr><a:prstGeom prst='rect'/><a:solidFill><a:srgbClr val='00FF00'/></a:solidFill><a:ln><a:noFill/></a:ln></wps:spPr></wps:wsp></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>",
    );
    let p = dom.element(root, &W::p()).unwrap();
    let mut boxes = collect_textboxes(None, &dom, p, &Defaults::word().run, &ThemeFonts::default());
    assert_eq!(boxes.len(), 1);
    let mut b = boxes.remove(0);
    b.w = w;
    b.h = h;
    text_box_image(b, [0.0; 4])
}
fn slot(h_frame: &str, v_frame: &str, col_x: i32, drop: i32, extra: &str, wrap: &str) -> ImageSlot {
    let h_position = if extra.contains("pctPosHOffset") {
        extra.to_string()
    } else {
        format!("<wp:posOffset>{col_x}</wp:posOffset>")
    };
    let v_position = if extra.contains("pctPosVOffset") {
        extra.to_string()
    } else {
        format!("<wp:posOffset>{drop}</wp:posOffset>")
    };
    let (dom, root) = dom(&format!(
        "<w:drawing><wp:anchor distR='50800' distB='63500'><wp:positionH relativeFrom='{h_frame}'>{h_position}</wp:positionH><wp:positionV relativeFrom='{v_frame}'>{v_position}</wp:positionV>{wrap}</wp:anchor></w:drawing>"
    ));
    let before = dom.serialize_element(root);
    let parsed = drawing_slot(&dom, dom.element(root, &W::drawing()).unwrap());
    assert_eq!(dom.serialize_element(root), before);
    parsed
}
fn assert_source(para: &CellPara, text: &str) {
    assert_eq!(
        para.runs
            .iter()
            .map(|r| r.text.as_str())
            .collect::<String>(),
        text
    );
    assert_eq!(para.bookmarks, ["OwnedCell"]);
    assert_eq!(para.blank_bookmarks, ["EmptyCell"]);
    assert_eq!(
        (para.style.before, para.style.after, para.style.line_exact),
        (2.0, 3.0, Some(20.0))
    );
}

#[test]
fn square_cell_float_room_and_vertical_reach_have_independent_literal_thresholds() {
    let fonts = fonts();
    for (col, drop, room, expected, height) in [
        (25400, 127000, 60.0, Some((26.0, 45.0)), 45.0),
        (25400, 127000, 44.0, Some((26.0, 45.0)), 45.0),
        (25400, 127000, 43.0, None, 65.0),
        (25400, 241300, 60.0, Some((26.0, 54.0)), 54.0),
        (25400, 254000, 60.0, None, 55.0),
        (25400, -127000, 60.0, Some((26.0, 35.0)), 35.0),
    ] {
        let mut para = para("AB");
        let mut image = picture(20.0, 30.0);
        image.slot = slot(
            "column",
            "paragraph",
            col,
            drop,
            "",
            "<wp:wrapSquare wrapText='bothSides'/>",
        );
        para.images.push(image);
        assert_eq!(cell_side_float(&fonts, &para, room), expected);
        assert_eq!(cell_para_height(&fonts, &para, room), height);
        assert_source(&para, "AB");
        assert_eq!(cell_image_wh(&para.images[0]), (20.0, 30.0));
    }
}

#[test]
fn page_and_percentage_positions_exclude_cell_owned_float_without_losing_extent() {
    for (h, v, extra, owned) in [
        ("column", "paragraph", "", true),
        ("page", "paragraph", "", false),
        ("column", "page", "", false),
        (
            "column",
            "paragraph",
            "<wp14:pctPosHOffset>25000</wp14:pctPosHOffset>",
            false,
        ),
        (
            "column",
            "paragraph",
            "<wp14:pctPosVOffset>25000</wp14:pctPosVOffset>",
            false,
        ),
    ] {
        let mut image = picture(20.0, 30.0);
        image.slot = slot(
            h,
            v,
            25400,
            127000,
            extra,
            "<wp:wrapSquare wrapText='bothSides'/>",
        );
        assert_eq!(cell_holds_image(&image), owned);
        assert_eq!(cell_image_wh(&image), (20.0, 30.0));
        let mut para = para("AB");
        para.images.push(image);
        assert_eq!(
            cell_side_float(&fonts(), &para, 60.0),
            owned.then_some((26.0, 45.0))
        );
        assert_source(&para, "AB");
    }
    let inline = picture(20.0, 30.0);
    assert!(cell_holds_image(&inline));
}

#[test]
fn cell_overlay_wrap_and_behind_flags_preserve_text_without_claiming_room() {
    let fonts = fonts();
    for (wrap, behind, expected_side, height, takes_room) in [
        (
            "<wp:wrapSquare wrapText='bothSides'/>",
            false,
            Some((26.0, 45.0)),
            45.0,
            true,
        ),
        (
            "<wp:wrapSquare wrapText='bothSides'/>",
            true,
            None,
            25.0,
            false,
        ),
        ("<wp:wrapNone/>", false, None, 25.0, false),
        ("<wp:wrapTopAndBottom/>", false, None, 65.0, true),
    ] {
        let mut para = para("AB");
        let mut image = picture(20.0, 30.0);
        image.slot = slot("column", "paragraph", 25400, 127000, "", wrap);
        image.behind = behind;
        para.images.push(image);
        let (align, offset, drop, room) = cell_image_place(&para.images[0], Align::Right);
        assert!(align == Align::Left);
        assert_eq!((offset, drop, room), (Some(2.0), 10.0, takes_room));
        assert_eq!(cell_side_float(&fonts, &para, 60.0), expected_side);
        assert_eq!(cell_para_height(&fonts, &para, 60.0), height);
        assert_source(&para, "AB");
    }
}

#[test]
fn leading_cell_icon_requires_first_source_position_and_both_icon_dimensions() {
    for (w, h, lead_chars, align, text, expected) in [
        (48.0, 24.0, 0, Align::Left, "AB", true),
        (49.0, 24.0, 0, Align::Left, "AB", false),
        (48.0, 25.0, 0, Align::Left, "AB", false),
        (48.0, 24.0, 1, Align::Left, "AB", false),
        (48.0, 24.0, 0, Align::Center, "AB", false),
        (48.0, 24.0, 0, Align::Right, "AB", false),
        (48.0, 24.0, 0, Align::Left, " ", false),
    ] {
        let mut para = para(text);
        para.style.align = align;
        let mut image = picture(w, h);
        image.lead_chars = lead_chars;
        para.images.push(image);
        assert_eq!(cell_lead_picture(&para).is_some(), expected);
        if expected {
            assert!(std::ptr::eq(
                cell_lead_picture(&para).unwrap(),
                &para.images[0]
            ));
        }
        assert_eq!(cell_image_wh(&para.images[0]), (w, h));
        assert_source(&para, text);
    }
}

fn raw_mono_single(size: f32) -> f32 {
    let face = ttf_parser::Face::parse(FaceId::MonoRegular.bytes(), 0).unwrap();
    f32::from(face.ascender() - face.descender() + face.line_gap()) * size
        / f32::from(face.units_per_em())
}
fn close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 0.001,
        "actual={actual}, expected={expected}"
    );
}

#[test]
fn cell_list_marker_lifts_once_while_exact_and_minimum_spacing_retain_authored_boxes() {
    let fonts = fonts();
    let mut source = para("Body");
    let mut marker = source.runs[0].with_text("1. ");
    marker.list_marker = true;
    marker.style.size = 24.0;
    source.runs.insert(0, marker);
    let raw = ttf_parser::Face::parse(FaceId::MonoRegular.bytes(), 0).unwrap();
    let body_line = raw_mono_single(12.0);
    let marker_up = raw_mono_single(24.0)
        - f32::from(raw.descender()).abs() * 24.0 / f32::from(raw.units_per_em());
    let body_descent = f32::from(raw.descender()).abs() * 12.0 / f32::from(raw.units_per_em());
    let natural = body_line.max(marker_up + body_descent);
    for (exact, minimum, multiple, expected) in [
        (None, None, 1.0, natural),
        (None, None, 1.5, body_line * 1.5 + natural - body_line),
        (Some(20.0), None, 1.5, 20.0),
        (None, Some(100.0), 1.5, 100.0),
        (None, Some(1.0), 1.5, natural),
    ] {
        source.style.line_exact = exact;
        source.style.line_at_least = minimum;
        source.style.line_mult = multiple;
        let (size, height) = cell_runs_line_box(&fonts, &source.runs, &source.style);
        assert_eq!(size, 24.0);
        close(height, expected);
        assert_eq!(source.runs[0].text, "1. ");
        assert!(source.runs[0].list_marker);
        assert_eq!(source.runs[1].text, "Body");
        assert!(!source.runs[1].list_marker);
        assert_eq!(source.bookmarks, ["OwnedCell"]);
    }
}

#[test]
fn picture_only_auto_multiple_uses_mark_face_and_declines_exact_minimum_or_unit_spacing() {
    let fonts = fonts();
    let mut source = para(" ");
    source.images.push(picture(20.0, 30.0));
    for explicit_mark in [false, true] {
        source.style.mark_run = explicit_mark.then(|| {
            let mut style = source.runs[0].style.clone();
            style.size = 24.0;
            std::rc::Rc::new(style)
        });
        for (exact, minimum, multiple, expected) in [
            (
                None,
                None,
                1.5,
                0.5 * raw_mono_single(if explicit_mark { 24.0 } else { 12.0 }),
            ),
            (Some(20.0), None, 1.5, 0.0),
            (None, Some(40.0), 1.5, 0.0),
            (None, None, 1.0, 0.0),
            (None, None, 0.8, 0.0),
        ] {
            source.style.line_exact = exact;
            source.style.line_at_least = minimum;
            source.style.line_mult = multiple;
            assert!(cell_para_is_image_only(&source));
            close(picture_line_leading(&fonts, &source), expected);
            close(cell_para_text_h(&fonts, &source, 60.0), expected);
            close(cell_para_height(&fonts, &source, 60.0), 35.0 + expected);
            assert_eq!(cell_image_wh(&source.images[0]), (20.0, 30.0));
            assert_eq!(source.runs[0].text, " ");
            assert_eq!(source.bookmarks, ["OwnedCell"]);
        }
    }
}

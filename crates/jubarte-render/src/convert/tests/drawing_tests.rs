// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;

fn letter_ctx() -> PlaceCtx {
    PlaceCtx {
        page_w: 612.0,
        page_h: 792.0,
        margin_l: 72.0,
        margin_r: 72.0,
        margin_t: 72.0,
        margin_b: 72.0,
        column_x: 72.0,
        para_top: 700.0,
        line_top: 688.0,
        cursor_x: 90.0,
    }
}

#[test]
fn resolve_anchor_page_offset_is_from_page_origin() {
    let p = resolve_anchor(
        &letter_ctx(),
        &AnchorSpec {
            w: 144.0,
            h: 72.0,
            h_from: "page",
            h_align: Align::Left,
            h_off: Some(72.0),
            v_from: "page",
            v_align: Align::Left,
            v_off: Some(72.0),
            wrap: WrapMode::None,
        },
    );
    assert!((p.x - 72.0).abs() < 0.01, "x={}", p.x);
    assert!((p.y - (792.0 - 72.0 - 72.0)).abs() < 0.01, "y={}", p.y);
    assert!((p.w - 144.0).abs() < 0.01);
    assert!((p.h - 72.0).abs() < 0.01);
}

#[test]
fn resolve_anchor_margin_right_align_sits_at_right_margin() {
    let p = resolve_anchor(
        &letter_ctx(),
        &AnchorSpec {
            w: 144.0,
            h: 50.0,
            h_from: "margin",
            h_align: Align::Right,
            h_off: None,
            v_from: "margin",
            v_align: Align::Left,
            v_off: Some(0.0),
            wrap: WrapMode::Square {
                dist_l: 9.0,
                dist_r: 9.0,
                dist_t: 0.0,
                dist_b: 0.0,
            },
        },
    );
    assert!((p.x - 396.0).abs() < 0.01, "612-72-144=396, got x={}", p.x);
    assert!((p.y - (792.0 - 72.0 - 50.0)).abs() < 0.01, "y={}", p.y);
    assert_eq!(
        p.wrap,
        WrapMode::Square {
            dist_l: 9.0,
            dist_r: 9.0,
            dist_t: 0.0,
            dist_b: 0.0,
        }
    );
}

#[test]
fn resolve_anchor_paragraph_offset_uses_para_top_not_page() {
    let p = resolve_anchor(
        &letter_ctx(),
        &AnchorSpec {
            w: 100.0,
            h: 40.0,
            h_from: "column",
            h_align: Align::Left,
            h_off: Some(0.0),
            v_from: "paragraph",
            v_align: Align::Left,
            v_off: Some(0.0),
            wrap: WrapMode::None,
        },
    );
    assert!((p.x - 72.0).abs() < 0.01, "x={}", p.x);
    assert!(
        (p.y - 660.0).abs() < 0.01,
        "para_top 700 - h 40 = 660, got y={}",
        p.y
    );
}

fn spec(
    h_from: &'static str,
    h_align: Align,
    v_from: &'static str,
    v_align: Align,
) -> AnchorSpec<'static> {
    AnchorSpec {
        w: 144.0,
        h: 72.0,
        h_from,
        h_align,
        h_off: None,
        v_from,
        v_align,
        v_off: None,
        wrap: WrapMode::None,
    }
}

#[test]
fn resolve_anchor_margin_areas_are_their_own_frames() {
    // ST_RelFromH leftMargin / rightMargin are the margin *areas*.
    let ctx = letter_ctx();
    let right = resolve_anchor(
        &ctx,
        &spec("rightMargin", Align::Right, "page", Align::Left),
    );
    assert!(
        (right.x - (612.0 - 144.0)).abs() < 0.01,
        "flush with the page edge: x={}",
        right.x
    );
    let left_off = resolve_anchor(
        &ctx,
        &AnchorSpec {
            h_off: Some(10.0),
            ..spec("rightMargin", Align::Left, "page", Align::Left)
        },
    );
    assert!(
        (left_off.x - (540.0 + 10.0)).abs() < 0.01,
        "offset from the right-margin edge: x={}",
        left_off.x
    );
    let lm = resolve_anchor(&ctx, &spec("leftMargin", Align::Left, "page", Align::Left));
    assert!(
        lm.x.abs() < 0.01,
        "leftMargin starts at the page edge: x={}",
        lm.x
    );
}

#[test]
fn resolve_anchor_vertical_alignment_uses_its_frame() {
    let ctx = letter_ctx();
    let y = |v_from, v_align| resolve_anchor(&ctx, &spec("margin", Align::Left, v_from, v_align)).y;
    assert!(
        (y("page", Align::Left) - (792.0 - 72.0)).abs() < 0.01,
        "page top"
    );
    assert!(y("page", Align::Right).abs() < 0.01, "page bottom");
    assert!(
        (y("page", Align::Center) - 360.0).abs() < 0.01,
        "page centre"
    );
    assert!(
        (y("margin", Align::Center) - (72.0 + (648.0 - 72.0) / 2.0)).abs() < 0.01,
        "margin centre"
    );
    assert!(
        (y("margin", Align::Right) - 72.0).abs() < 0.01,
        "margin bottom"
    );
    assert!(
        (y("paragraph", Align::Left) - (700.0 - 72.0)).abs() < 0.01,
        "paragraph top"
    );
    assert!(
        (y("line", Align::Left) - (688.0 - 72.0)).abs() < 0.01,
        "line top"
    );
    assert!(
        (y("topMargin", Align::Right) - 720.0).abs() < 0.01,
        "top-margin bottom"
    );
    assert!(
        y("bottomMargin", Align::Right).abs() < 0.01,
        "bottom-margin bottom"
    );
}

#[test]
fn resolve_anchor_keeps_explicit_offsets_unclamped() {
    // posOffset is signed and may run off the page; Word clips at paint.
    let ctx = letter_ctx();
    let low = resolve_anchor(
        &ctx,
        &AnchorSpec {
            v_off: Some(780.0),
            ..spec("page", Align::Left, "page", Align::Left)
        },
    );
    assert!((low.y - (792.0 - 780.0 - 72.0)).abs() < 0.01, "y={}", low.y);
    let above = resolve_anchor(
        &ctx,
        &AnchorSpec {
            v_off: Some(-20.0),
            ..spec("page", Align::Left, "margin", Align::Left)
        },
    );
    assert!(
        (above.y - (720.0 + 20.0 - 72.0)).abs() < 0.01,
        "y={}",
        above.y
    );
    let bottom = resolve_anchor(
        &ctx,
        &AnchorSpec {
            v_off: Some(10.0),
            ..spec("page", Align::Left, "bottomMargin", Align::Left)
        },
    );
    assert!(
        (bottom.y - (72.0 - 10.0 - 72.0)).abs() < 0.01,
        "offsets run down from the bottom-margin top: y={}",
        bottom.y
    );
}

#[test]
fn resolve_anchor_character_uses_cursor_x() {
    let p = resolve_anchor(
        &letter_ctx(),
        &AnchorSpec {
            w: 20.0,
            h: 20.0,
            h_from: "character",
            h_align: Align::Left,
            h_off: Some(0.0),
            v_from: "line",
            v_align: Align::Left,
            v_off: Some(0.0),
            wrap: WrapMode::None,
        },
    );
    assert!((p.x - 90.0).abs() < 0.01, "x={}", p.x);
    assert!((p.y - (688.0 - 20.0)).abs() < 0.01, "y={}", p.y);
    assert_eq!(p.wrap, WrapMode::None);
}

#[test]
fn word_device_pt_does_not_snap_thirteen_or_twenty_six_after_mini_429() {
    // Word Quartz 26pt/13pt is 25.92/12.96 but mini 429 ITT-neg
    // table_bookmark −0.070 / file_134 −0.059. Keep 26/13 unsnapped.
    assert!((word_device_pt(13.0) - 13.0).abs() < 0.001);
    assert!((word_device_pt(26.0) - 26.0).abs() < 0.001);
    assert!((word_device_pt(11.0) - 11.04).abs() < 0.001);
    assert!((word_device_pt(14.0) - 14.0).abs() < 0.001);
}

const DRAWING_XML: &str = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<w:body><w:p>
<w:r><w:t>Hello</w:t></w:r>
<w:r><w:drawing>
  <wp:anchor>
    <wp:positionH relativeFrom="margin"><wp:align>left</wp:align></wp:positionH>
    <wp:positionV relativeFrom="margin"><wp:align>top</wp:align></wp:positionV>
    <wp:posOffset>123456789</wp:posOffset>
    <wp:extent cx="137160" cy="137160"/>
    <wp:wrapSquare wrapText="bothSides"/>
  </wp:anchor>
</w:drawing></w:r>
</w:p></w:body></w:document>"#;

fn load() -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(DRAWING_XML);
    let root = dom.root(doc).expect("root");
    (dom, root)
}

#[test]
fn drawing_align_and_posoffset_are_not_runs() {
    let (dom, root) = load();
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
    let joined: String = runs.iter().map(|r| r.text.as_str()).collect();
    assert_eq!(joined, "Hello");
    assert!(!joined.contains("left"), "{joined}");
    assert!(!joined.contains("123456789"), "{joined}");
}

#[test]
fn unnamespaced_extent_cx_cy_are_read() {
    let (dom, root) = load();
    let drawing = dom
        .descendants(root, Some(&W::drawing()))
        .into_iter()
        .next()
        .expect("drawing");
    let (w, h) = drawing_extent_pt(&dom, drawing);
    assert!((w - 10.8).abs() < 0.05, "w={w}");
    assert!((h - 10.8).abs() < 0.05, "h={h}");
}

#[test]
fn wrap_square_anchor_is_float() {
    let (dom, root) = load();
    let drawing = dom
        .descendants(root, Some(&W::drawing()))
        .into_iter()
        .next()
        .expect("drawing");
    assert!(matches!(
        drawing_slot(&dom, drawing),
        ImageSlot::Float {
            align: Align::Left,
            ..
        }
    ));
}

#[test]
fn wrap_tight_anchor_is_square() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor distL="114300" distR="114300">
    <wp:positionH relativeFrom="margin"><wp:align>right</wp:align></wp:positionH>
    <wp:positionV relativeFrom="margin"><wp:align>top</wp:align></wp:positionV>
    <wp:extent cx="1828800" cy="1828800"/>
    <wp:wrapTight wrapText="bothSides"/>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let drawing = dom
        .descendants(root, Some(&W::drawing()))
        .into_iter()
        .next()
        .expect("drawing");
    match drawing_slot(&dom, drawing) {
        ImageSlot::Float {
            wrap_square,
            wrap_top_bottom,
            ..
        } => {
            assert!(wrap_square, "wrapTight ≈ Square");
            assert!(!wrap_top_bottom);
        }
        _ => panic!("wrapTight must be a float, not Flow"),
    }
}

#[test]
fn wrap_top_and_bottom_anchor_is_float_not_flow() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor>
    <wp:positionH relativeFrom="margin"><wp:align>right</wp:align></wp:positionH>
    <wp:positionV relativeFrom="margin"><wp:align>top</wp:align></wp:positionV>
    <wp:extent cx="1828800" cy="914400"/>
    <wp:wrapTopAndBottom/>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let drawing = dom
        .descendants(root, Some(&W::drawing()))
        .into_iter()
        .next()
        .expect("drawing");
    match drawing_slot(&dom, drawing) {
        ImageSlot::Float {
            wrap_square,
            wrap_top_bottom,
            align,
            ..
        } => {
            assert!(!wrap_square);
            assert!(wrap_top_bottom);
            assert!(matches!(align, Align::Right));
        }
        _ => panic!("wrapTopAndBottom must overlay, not consume Flow"),
    }
}

#[test]
fn vml_imagedata_absolute_slot_is_not_flow() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:v="urn:schemas-microsoft-com:vml" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<w:body><w:p><w:r><w:pict>
  <v:shape style="position:absolute;margin-left:187.95pt;margin-top:15.9pt;width:72pt;height:36pt;mso-position-horizontal-relative:page;mso-position-vertical-relative:page">
    <v:imagedata r:id="rIdImg"/>
  </v:shape>
</w:pict></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let pict = dom
        .descendants(root, Some(&W::pict()))
        .into_iter()
        .next()
        .expect("pict");
    match vml_absolute_slot(&dom, pict) {
        Some(ImageSlot::Float {
            page_x: Some(x),
            page_y: Some(y),
            ..
        }) => {
            assert!((x - 187.95).abs() < 0.05, "page_x={x}");
            assert!((y - 15.9).abs() < 0.05, "page_y={y}");
        }
        _ => panic!("expected page-origin float, not Flow"),
    }
}

/// image_out_of_folder: DeepL parks the banner PNG in `wp:wrapSquare`
/// and the same words in a sibling VML `w:pict`. Word prints the PNG
/// only; the pict is editor chrome. The file's pict is page-relative
/// in both axes (an absent relative is VML's `text`).
const DEEPL_SIBLING_XML: &str = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
 xmlns:v="urn:schemas-microsoft-com:vml">
<w:body><w:p>
<w:r>
<w:drawing>
  <wp:anchor>
    <wp:positionH relativeFrom="page"><wp:posOffset>0</wp:posOffset></wp:positionH>
    <wp:positionV relativeFrom="page"><wp:posOffset>0</wp:posOffset></wp:positionV>
    <wp:extent cx="10690522" cy="807396"/>
    <wp:wrapSquare wrapText="bothSides"/>
    <a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture">
      <a:blip r:embed="rId1"/>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing>
<w:pict>
  <v:shape style="position:absolute;margin-left:187.95pt;margin-top:15.9pt;width:477.9pt;height:42.8pt;mso-position-horizontal-relative:page;mso-position-vertical-relative:page" filled="f" stroked="f">
  <v:textbox>
    <w:txbxContent><w:p><w:r><w:t>Subscribe to DeepL Pro</w:t></w:r></w:p></w:txbxContent>
  </v:textbox>
  </v:shape>
</w:pict>
</w:r>
</w:p></w:body></w:document>"#;

#[test]
fn wrap_square_picture_overlays_sibling_vml_textbox() {
    // Word Quartz paints the VML "Subscribe to DeepL Pro" as page-origin
    // overlay (margin-left 187.95pt / margin-top 15.9pt). Skipping it
    // dropped the vector copy; flowing it (ITT 41) shoved Quantum down.
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(DEEPL_SIBLING_XML);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    let overlay = boxes
        .iter()
        .find(|b| b.runs.iter().any(|r| r.text.contains("Subscribe to DeepL")));
    let overlay = overlay.expect("sibling VML txbx must overlay, not skip");
    match overlay.slot {
        ImageSlot::Float {
            page_x: Some(x),
            page_y: Some(y),
            ..
        } => {
            assert!((x - 187.95).abs() < 0.05, "page_x={x}");
            assert!((y - 15.9).abs() < 0.05, "page_y={y}");
        }
        _ => panic!("expected page-origin float overlay"),
    }
    assert!(!overlay.stroke, "DeepL v:shape stroked=f");
    assert!(
        overlay.text_dx.abs() < 0.05,
        "mini 417 ITT-neg unindented lIns=7.2; keep pad=4 path text_dx=0; text_dx={}",
        overlay.text_dx
    );
}

#[test]
fn vml_textbox_center_relative_to_text_is_not_page_origin() {
    // xml 3.4 ckpt 2 / file_104: mso-position-horizontal:center
    // relative to text is ImageSlot center, not page_x=0.
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:v="urn:schemas-microsoft-com:vml" xmlns:w10="urn:schemas-microsoft-com:office:word">
<w:body><w:p>
<w:r><w:t>Title</w:t></w:r>
<w:r><w:pict>
  <v:shape style="position:absolute;left:0;margin-left:0;margin-top:0;width:186.35pt;height:110.6pt;z-index:251659264;visibility:visible;mso-wrap-style:square;mso-wrap-distance-left:9pt;mso-wrap-distance-right:9pt;mso-position-horizontal:center;mso-position-horizontal-relative:text;mso-position-vertical:absolute;mso-position-vertical-relative:text">
  <v:textbox>
    <w:txbxContent><w:p><w:r><w:t>hello</w:t></w:r></w:p></w:txbxContent>
  </v:textbox>
  <w10:wrap type="square"/>
  </v:shape>
</w:pict></w:r>
</w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    let box_ = boxes
        .iter()
        .find(|b| b.runs.iter().any(|r| r.text.contains("hello")))
        .expect("vml hello txbx");
    match box_.slot {
        ImageSlot::Float {
            align,
            page_x,
            wrap_square,
            para_y,
            ..
        } => {
            assert!(
                matches!(align, Align::Center),
                "mso-position-horizontal:center"
            );
            assert!(
                page_x.is_none(),
                "text-relative center is not page origin; page_x={page_x:?}"
            );
            assert!(wrap_square, "w10:wrap type=square");
            assert!(
                para_y.is_some(),
                "mso-position-vertical-relative:text; para_y={para_y:?}"
            );
        }
        _ => panic!("expected float slot, got flow w={}", box_.w),
    }
}

/// The VML shape's wrap is its `w10:wrap`, never `mso-wrap-style`, and
/// its side distances default to 9pt (Word probes of 3cccdeb956's
/// header logo: no `w10:wrap` paints the text over the logo,
/// `mso-wrap-style:none` with `w10:wrap square` wraps 9pt clear).
#[test]
fn vml_wrap_comes_from_w10_wrap_with_9pt_sides() {
    let slot = |shape_style: &str, wrap: &str| {
        let xml = format!(
            r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:v="urn:schemas-microsoft-com:vml" xmlns:w10="urn:schemas-microsoft-com:office:word">
<w:body><w:p><w:r><w:pict>
  <v:shape style="position:absolute;margin-left:0;margin-top:0;width:100pt;height:50pt;{shape_style}">
  <v:textbox><w:txbxContent><w:p><w:r><w:t>hello</w:t></w:r></w:p></w:txbxContent></v:textbox>
  {wrap}
  </v:shape>
</w:pict></w:r></w:p></w:body></w:document>"#
        );
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        match boxes.first().expect("vml txbx").slot {
            ImageSlot::Float {
                wrap_square,
                dist_l,
                dist_r,
                dist_t,
                dist_b,
                ..
            } => (wrap_square, [dist_l, dist_r, dist_t, dist_b]),
            ImageSlot::Flow => panic!("expected a float"),
        }
    };
    let (square, _) = slot("mso-wrap-style:square", "");
    assert!(!square, "mso-wrap-style alone wraps no text");
    let (square, dist) = slot("mso-wrap-style:none", r#"<w10:wrap type="square"/>"#);
    assert!(square, "w10:wrap square wraps");
    assert_eq!(dist, [9.0, 9.0, 0.0, 0.0], "VML's default distances");
    let (_, dist) = slot(
        "mso-wrap-distance-left:0;mso-wrap-distance-right:0",
        r#"<w10:wrap type="square"/>"#,
    );
    assert_eq!(dist, [0.0, 0.0, 0.0, 0.0], "explicit distances stay");
}

#[test]
fn standalone_vml_pict_textbox_is_kept() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:v="urn:schemas-microsoft-com:vml">
<w:body><w:p>
<w:r><w:pict>
  <v:textbox>
    <w:txbxContent><w:p><w:r><w:t>Datum plane</w:t></w:r></w:p></w:txbxContent>
  </v:textbox>
</w:pict></w:r>
</w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(
        boxes.len(),
        1,
        "pict-only txbx must still paint; n={}",
        boxes.len()
    );
    let text: String = boxes[0].runs.iter().map(|r| r.text.as_str()).collect();
    assert_eq!(text, "Datum plane");
}

// `wps` is declared: a Choice on an undeclared prefix is passed over
// for the Fallback, and Word cannot read such a file at all (#1876).
const TXBX_XML: &str = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape"
 xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006">
<w:body><w:p>
<w:r>
<mc:AlternateContent>
  <mc:Choice Requires="wps">
    <w:drawing>
      <wp:anchor>
        <wp:positionH relativeFrom="column"><wp:align>center</wp:align></wp:positionH>
        <wp:extent cx="2374265" cy="1403985"/>
        <wp:wrapNone/>
        <w:txbxContent><w:p><w:r><w:t>Datum plane</w:t></w:r></w:p></w:txbxContent>
      </wp:anchor>
    </w:drawing>
  </mc:Choice>
  <mc:Fallback>
    <w:pict>
      <w:txbxContent><w:p><w:r><w:t>Datum plane</w:t></w:r></w:p></w:txbxContent>
    </w:pict>
  </mc:Fallback>
</mc:AlternateContent>
</w:r>
</w:p></w:body></w:document>"#;

#[test]
fn txbx_content_is_a_box_not_a_paragraph_run() {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(TXBX_XML);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
    let joined: String = runs.iter().map(|r| r.text.as_str()).collect();
    assert_eq!(joined, "", "txbx text is not body text; got {joined}");
    assert!(!joined.contains("center"));
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1, "Choice+Fallback must not duplicate");
    let text: String = boxes[0].runs.iter().map(|r| r.text.as_str()).collect();
    assert_eq!(text, "Datum plane");
    assert!((boxes[0].w - 186.95).abs() < 0.2, "w={}", boxes[0].w);
    assert!(matches!(
        boxes[0].slot,
        ImageSlot::Float {
            align: Align::Center,
            ..
        }
    ));
}

#[test]
fn inline_empty_rectangle_is_skipped() {
    // Strict01 Rectangle 3: inline wsp, no txbx. Must not become a flow box.
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
<w:body><w:p>
<w:r><w:drawing>
  <wp:inline>
    <wp:extent cx="5104000" cy="2122000"/>
    <a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingShape"/>
    </a:graphic>
  </wp:inline>
</w:drawing></w:r>
</w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1, "inline noFill extent must reserve flow");
    assert!(boxes[0].reserve_only, "must not stroke or fill Rectangle 3");
    assert!(
        (boxes[0].h - 167.09).abs() < 0.5,
        "167pt hole; h={}",
        boxes[0].h
    );
}

#[test]
fn chart_drawing_reserves_flow_space() {
    // 5486400×3200400 EMU = 432×252 pt (Strict01 page-1 chart).
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
<w:body><w:p>
<w:r><w:drawing>
  <wp:inline>
    <wp:extent cx="5486400" cy="3200400"/>
    <a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/chart"/>
    </a:graphic>
  </wp:inline>
</w:drawing></w:r>
</w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1, "chart must emit a placeholder box");
    assert!(boxes[0].runs.iter().all(|r| r.text.trim().is_empty()));
    assert!((boxes[0].w - 432.0).abs() < 0.5, "w={}", boxes[0].w);
    assert!((boxes[0].h - 252.0).abs() < 0.5, "h={}", boxes[0].h);
    assert!(matches!(boxes[0].slot, ImageSlot::Flow));
}

#[test]
fn bent_connector_shape_is_collected() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="2149522" cy="1207827"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="bentConnector3"/>
        <a:ln><a:solidFill><a:srgbClr val="4F81BD"/></a:solidFill></a:ln>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(
        boxes.len(),
        1,
        "bent connector must be collected; n={}",
        boxes.len()
    );
    assert!(
        matches!(boxes[0].geom, ShapeGeom::BentConnector),
        "geom must be BentConnector"
    );
    assert!(boxes[0].fill.is_some(), "line color must paint");
    assert!(!boxes[0].tail_end, "fixture has no tailEnd");
}

#[test]
fn ellipse_prst_is_not_a_box() {
    // plan Step 7 / case34: unknown prstGeom must not collapse to a
    // rectangle. Word's ellipse is an oval in the extent.
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="900000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="ellipse"/>
        <a:solidFill><a:srgbClr val="FF0000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=ellipse must not collapse to Box; Word paints an oval"
    );
}

#[test]
fn parallelogram_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="900000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="parallelogram"/>
        <a:solidFill><a:srgbClr val="00FF00"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=parallelogram must not collapse to Box"
    );
}

#[test]
fn plus_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="plus"/>
        <a:solidFill><a:srgbClr val="0000FF"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=plus must not collapse to Box"
    );
}

#[test]
fn pentagon_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="pentagon"/>
        <a:solidFill><a:srgbClr val="FF00FF"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=pentagon must not collapse to Box"
    );
}

#[test]
fn octagon_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="octagon"/>
        <a:solidFill><a:srgbClr val="00FFFF"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=octagon must not collapse to Box"
    );
}

#[test]
fn star4_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="star4"/>
        <a:solidFill><a:srgbClr val="FFFF00"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=star4 must not collapse to Box"
    );
}

#[test]
fn star5_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="star5"/>
        <a:solidFill><a:srgbClr val="BF8F00"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=star5 must not collapse to Box"
    );
}

#[test]
fn rt_triangle_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="900000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="rtTriangle"/>
        <a:solidFill><a:srgbClr val="FF8800"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=rtTriangle must not collapse to Box"
    );
}

#[test]
fn heart_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="heart"/>
        <a:solidFill><a:srgbClr val="FF0000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=heart must not collapse to Box"
    );
}

#[test]
fn donut_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="donut"/>
        <a:solidFill><a:srgbClr val="00AAFF"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=donut must not collapse to Box"
    );
}

#[test]
fn frame_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="frame"/>
        <a:solidFill><a:srgbClr val="333333"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=frame must not collapse to Box"
    );
}

#[test]
fn flow_chart_terminator_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="900000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="flowChartTerminator"/>
        <a:solidFill><a:srgbClr val="0070C0"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=flowChartTerminator must not collapse to Box"
    );
}

#[test]
fn heptagon_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="heptagon"/>
        <a:solidFill><a:srgbClr val="7030A0"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=heptagon must not collapse to Box"
    );
}

#[test]
fn star6_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="star6"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=star6 must not collapse to Box"
    );
}

#[test]
fn star7_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="star7"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=star7 must not collapse to Box"
    );
}

#[test]
fn star8_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="star8"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=star8 must not collapse to Box"
    );
}

#[test]
fn star10_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="star10"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=star10 must not collapse to Box"
    );
}

#[test]
fn star12_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="star12"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=star12 must not collapse to Box"
    );
}

#[test]
fn star16_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="star16"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=star16 must not collapse to Box"
    );
}

#[test]
fn star24_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="star24"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=star24 must not collapse to Box"
    );
}

#[test]
fn star32_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="star32"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=star32 must not collapse to Box"
    );
}

#[test]
fn flow_chart_document_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="flowChartDocument"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=flowChartDocument must not collapse to Box"
    );
}

#[test]
fn flow_chart_offpage_connector_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="flowChartOffpageConnector"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=flowChartOffpageConnector must not collapse to Box"
    );
}

#[test]
fn flow_chart_delay_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="flowChartDelay"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=flowChartDelay must not collapse to Box"
    );
}

#[test]
fn flow_chart_manual_input_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="flowChartManualInput"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=flowChartManualInput must not collapse to Box"
    );
}

#[test]
fn flow_chart_punched_card_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="flowChartPunchedCard"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=flowChartPunchedCard must not collapse to Box"
    );
}

#[test]
fn flow_chart_preparation_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="flowChartPreparation"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=flowChartPreparation must not collapse to Box"
    );
}

#[test]
fn flow_chart_extract_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="flowChartExtract"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=flowChartExtract must not collapse to Box"
    );
}

#[test]
fn flow_chart_merge_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="flowChartMerge"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=flowChartMerge must not collapse to Box"
    );
}

#[test]
fn flow_chart_collate_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="flowChartCollate"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=flowChartCollate must not collapse to Box"
    );
}

#[test]
fn double_wave_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="doubleWave"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=doubleWave must not collapse to Box"
    );
}

#[test]
fn flow_chart_display_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="flowChartDisplay"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=flowChartDisplay must not collapse to Box"
    );
}

#[test]
fn flow_chart_input_output_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="flowChartInputOutput"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=flowChartInputOutput must not collapse to Box"
    );
}

#[test]
fn flow_chart_manual_operation_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="flowChartManualOperation"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=flowChartManualOperation must not collapse to Box"
    );
}

#[test]
fn flow_chart_sort_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="flowChartSort"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=flowChartSort must not collapse to Box"
    );
}

#[test]
fn flow_chart_offline_storage_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="flowChartOfflineStorage"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=flowChartOfflineStorage must not collapse to Box"
    );
}

#[test]
fn flow_chart_online_storage_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="flowChartOnlineStorage"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=flowChartOnlineStorage must not collapse to Box"
    );
}

#[test]
fn flow_chart_connector_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="flowChartConnector"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=flowChartConnector must not collapse to Box"
    );
    assert!(
        matches!(boxes[0].geom, ShapeGeom::Ellipse),
        "prst=flowChartConnector is an ellipse"
    );
}

#[test]
fn flow_chart_punched_tape_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="flowChartPunchedTape"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=flowChartPunchedTape must not collapse to Box"
    );
}

#[test]
fn flow_chart_alternate_process_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="flowChartAlternateProcess"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=flowChartAlternateProcess must not collapse to Box"
    );
    assert!(
        matches!(boxes[0].geom, ShapeGeom::RoundRect),
        "prst=flowChartAlternateProcess is a roundRect (ssd6 ≈ default adj)"
    );
}

#[test]
fn flow_chart_or_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="flowChartOr"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=flowChartOr must not collapse to Box"
    );
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Ellipse),
        "prst=flowChartOr is a circle plus a cross, not a bare ellipse"
    );
}

#[test]
fn flow_chart_summing_junction_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="flowChartSummingJunction"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=flowChartSummingJunction must not collapse to Box"
    );
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Ellipse),
        "prst=flowChartSummingJunction is a circle plus an X, not a bare ellipse"
    );
}

#[test]
fn flow_chart_internal_storage_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="flowChartInternalStorage"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=flowChartInternalStorage must not collapse to Box"
    );
}

#[test]
fn flow_chart_predefined_process_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="flowChartPredefinedProcess"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=flowChartPredefinedProcess must not collapse to Box"
    );
}

#[test]
fn flow_chart_magnetic_disk_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="flowChartMagneticDisk"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=flowChartMagneticDisk must not collapse to Box"
    );
}

#[test]
fn flow_chart_magnetic_drum_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="flowChartMagneticDrum"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=flowChartMagneticDrum must not collapse to Box"
    );
}

#[test]
fn flow_chart_magnetic_tape_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="flowChartMagneticTape"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=flowChartMagneticTape must not collapse to Box"
    );
}

#[test]
fn flow_chart_multidocument_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="flowChartMultidocument"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=flowChartMultidocument must not collapse to Box"
    );
}

#[test]
fn cube_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="cube"/>
        <a:solidFill><a:srgbClr val="5B9BD5"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=cube must not collapse to Box"
    );
}

#[test]
fn folded_corner_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="foldedCorner"/>
        <a:solidFill><a:srgbClr val="ED7D31"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=foldedCorner must not collapse to Box"
    );
}

#[test]
fn can_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="can"/>
        <a:solidFill><a:srgbClr val="70AD47"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=can must not collapse to Box"
    );
}

#[test]
fn cloud_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="cloud"/>
        <a:solidFill><a:srgbClr val="5B9BD5"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=cloud must not collapse to Box"
    );
}

#[test]
fn pie_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="pie"/>
        <a:solidFill><a:srgbClr val="ED7D31"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=pie must not collapse to Box"
    );
}

#[test]
fn left_right_arrow_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="900000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="leftRightArrow"/>
        <a:solidFill><a:srgbClr val="4472C4"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=leftRightArrow must not collapse to Box"
    );
}

#[test]
fn quad_arrow_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="quadArrow"/>
        <a:solidFill><a:srgbClr val="4472C4"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=quadArrow must not collapse to Box"
    );
}

#[test]
fn lightning_bolt_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="lightningBolt"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=lightningBolt must not collapse to Box"
    );
}

#[test]
fn sun_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="sun"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=sun must not collapse to Box"
    );
}

#[test]
fn moon_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="moon"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=moon must not collapse to Box"
    );
}

#[test]
fn circular_arrow_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="circularArrow"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=circularArrow must not collapse to Box"
    );
}

#[test]
fn gear6_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="gear6"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=gear6 must not collapse to Box"
    );
}

#[test]
fn smiley_face_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="smileyFace"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=smileyFace must not collapse to Box"
    );
}

#[test]
fn gear9_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="gear9"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=gear9 must not collapse to Box"
    );
}

#[test]
fn teardrop_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="teardrop"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=teardrop must not collapse to Box"
    );
}

#[test]
fn no_smoking_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="noSmoking"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=noSmoking must not collapse to Box"
    );
}

#[test]
fn plaque_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="plaque"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=plaque must not collapse to Box"
    );
}

#[test]
fn left_circular_arrow_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="leftCircularArrow"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=leftCircularArrow must not collapse to Box"
    );
}

#[test]
fn block_arc_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="blockArc"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=blockArc must not collapse to Box"
    );
}

#[test]
fn chord_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="chord"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=chord must not collapse to Box"
    );
}

#[test]
fn bevel_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="bevel"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=bevel must not collapse to Box"
    );
}

#[test]
fn arc_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="arc"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=arc must not collapse to Box"
    );
}

#[test]
fn left_bracket_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="leftBracket"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=leftBracket must not collapse to Box"
    );
}

#[test]
fn right_bracket_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="rightBracket"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=rightBracket must not collapse to Box"
    );
}

#[test]
fn left_brace_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="leftBrace"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=leftBrace must not collapse to Box"
    );
}

#[test]
fn right_brace_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="rightBrace"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=rightBrace must not collapse to Box"
    );
}

#[test]
fn brace_pair_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="bracePair"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=bracePair must not collapse to Box"
    );
}

#[test]
fn bracket_pair_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="bracketPair"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=bracketPair must not collapse to Box"
    );
}

#[test]
fn snip1_rect_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="snip1Rect"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=snip1Rect must not collapse to Box"
    );
}

#[test]
fn round1_rect_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="round1Rect"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=round1Rect must not collapse to Box"
    );
}

#[test]
fn snip2_same_rect_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="snip2SameRect"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=snip2SameRect must not collapse to Box"
    );
}

#[test]
fn round2_same_rect_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="round2SameRect"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=round2SameRect must not collapse to Box"
    );
}

#[test]
fn snip2_diag_rect_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="snip2DiagRect"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=snip2DiagRect must not collapse to Box"
    );
}

#[test]
fn round2_diag_rect_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="round2DiagRect"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=round2DiagRect must not collapse to Box"
    );
}

#[test]
fn ribbon_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="ribbon"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=ribbon must not collapse to Box"
    );
}

#[test]
fn ribbon2_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="ribbon2"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=ribbon2 must not collapse to Box"
    );
}

#[test]
fn left_right_circular_arrow_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="leftRightCircularArrow"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=leftRightCircularArrow must not collapse to Box"
    );
}

#[test]
fn wave_prst_is_not_a_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="wave"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        !matches!(boxes[0].geom, ShapeGeom::Box),
        "prst=wave must not collapse to Box"
    );
}

#[test]
fn circle_prst_maps_to_ellipse() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="circle"/>
        <a:solidFill><a:srgbClr val="70AD47"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        matches!(boxes[0].geom, ShapeGeom::Ellipse),
        "prst=circle must map to Ellipse, not Box"
    );
}

#[test]
fn bent_connector_reads_triangle_tail_end() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="2149522" cy="1207827"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="bentConnector3"/>
        <a:ln><a:solidFill><a:srgbClr val="4F81BD"/></a:solidFill>
          <a:tailEnd type="triangle"/>
        </a:ln>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1);
    assert!(
        boxes[0].tail_end,
        "a:tailEnd type=triangle must set tail_end"
    );
}

#[test]
fn arrowhead_triangle_points_past_the_stroke_end() {
    let tri = arrowhead_triangle((0.0, 10.0), (10.0, 10.0));
    let tip = tri[1];
    assert!(
        tip.0 > 10.0 && (tip.1 - 10.0).abs() < 0.02,
        "rightward tailEnd tip past x=10; {tri:?}"
    );
}

#[test]
fn wave_underline_segments_have_dy() {
    let segs = wave_underline_segments(100.0, 50.0, 20.0);
    assert!(segs.len() >= 4, "{segs:?}");
    assert!(
        segs.iter().any(|s| (s.3 - s.1).abs() > 0.35),
        "wave must not be a horizontal rule; {segs:?}"
    );
    let x1 = segs.first().map(|s| s.0).unwrap();
    let x2 = segs.last().map(|s| s.2).unwrap();
    assert!(
        (x1 - 100.0).abs() < 0.05 && (x2 - 120.0).abs() < 0.05,
        "{segs:?}"
    );
}

#[test]
fn bent_connector_points_match_ooxml_default() {
    let pts = bent_connector_points(262.7, 608.25, 164.25, 95.35);
    assert!((pts[1].0 - (262.7 + 164.25 * 0.5)).abs() < 0.02, "{pts:?}");
    assert!((pts[0].1 - (608.25 + 95.35)).abs() < 0.02, "top {pts:?}");
    assert!((pts[2].1 - 608.25).abs() < 0.02, "bottom {pts:?}");
    assert!((pts[3].0 - (262.7 + 164.25)).abs() < 0.02, "end {pts:?}");
}

#[test]
fn curved_connector_flip_v_starts_at_bottom() {
    let x = 10.0;
    let y = 20.0;
    let dw = 100.0;
    let dh = 40.0;
    let curve = curved_connector_cubics(x, y, dw, dh, false, true);
    assert!(
        (curve.start.0 - x).abs() < 0.02 && (curve.start.1 - y).abs() < 0.02,
        "flipV start is bottom-left; {:?}",
        curve.start
    );
    let end = curve.segments[1][2];
    assert!(
        (end.0 - (x + dw)).abs() < 0.02 && (end.1 - (y + dh)).abs() < 0.02,
        "flipV end is top-right; {end:?}"
    );
    let [(c1x, _), (c2x, c2y), (ex, ey)] = curve.segments[0];
    assert!(
        (c1x - (x + dw * 0.25)).abs() < 0.02
            && (c2x - ex).abs() < 0.02
            && (c2y - (y + dh * 0.25)).abs() < 0.02
            && (ey - (y + dh * 0.5)).abs() < 0.02
            && (c2y - ey).abs() > 1.0,
        "Word S-curve: first c2 is quarter-height; segs={:?}",
        curve.segments
    );
}

#[test]
fn ellipse_points_lie_on_the_bounding_oval() {
    let pts = ellipse_points(0.0, 0.0, 100.0, 40.0);
    assert_eq!(pts.len(), 24);
    let on_oval = pts.iter().all(|(x, y)| {
        let nx = (*x - 50.0) / 50.0;
        let ny = (*y - 20.0) / 20.0;
        (nx * nx + ny * ny - 1.0).abs() < 0.02
    });
    assert!(on_oval, "ellipse vertices must sit on the oval; {pts:?}");
    assert!(
        !pts.iter().any(|(x, y)| x.abs() < 0.05 && y.abs() < 0.05),
        "must not include the bounding-box corner; {pts:?}"
    );
}

#[test]
fn triangle_points_are_isosceles_apex_top() {
    let pts = triangle_points(10.0, 20.0, 80.0, 40.0);
    assert_eq!(pts.len(), 3);
    assert!((pts[0].0 - 50.0).abs() < 0.01 && (pts[0].1 - 60.0).abs() < 0.01);
    assert!((pts[1].0 - 90.0).abs() < 0.01 && (pts[1].1 - 20.0).abs() < 0.01);
    assert!((pts[2].0 - 10.0).abs() < 0.01 && (pts[2].1 - 20.0).abs() < 0.01);
}

#[test]
fn parallelogram_points_use_ooxml_default_inset() {
    let pts = parallelogram_points(0.0, 0.0, 100.0, 40.0);
    assert_eq!(pts.len(), 4);
    assert!((pts[1].0 - 10.0).abs() < 0.01 && (pts[1].1 - 40.0).abs() < 0.01);
    assert!((pts[3].0 - 90.0).abs() < 0.01 && pts[3].1.abs() < 0.01);
}

#[test]
fn star4_points_have_four_tips() {
    let pts = star4_points(0.0, 0.0, 100.0, 100.0);
    assert_eq!(pts.len(), 8);
    assert!((pts[2].0 - 50.0).abs() < 0.05 && (pts[2].1 - 100.0).abs() < 0.05);
    assert!((pts[6].0 - 50.0).abs() < 0.05 && pts[6].1.abs() < 0.05);
}

#[test]
fn rt_triangle_points_right_angle_at_bottom_left() {
    let pts = rt_triangle_points(0.0, 0.0, 80.0, 40.0);
    assert_eq!(pts.len(), 3);
    assert!((pts[0].0).abs() < 0.01 && pts[0].1.abs() < 0.01);
    assert!((pts[1].0).abs() < 0.01 && (pts[1].1 - 40.0).abs() < 0.01);
    assert!((pts[2].0 - 80.0).abs() < 0.01 && pts[2].1.abs() < 0.01);
}

#[test]
fn star5_points_have_five_tips() {
    let pts = star5_points(0.0, 0.0, 100.0, 100.0);
    assert_eq!(pts.len(), 10);
    assert!((pts[2].0 - 50.0).abs() < 0.05 && (pts[2].1 - 100.0).abs() < 0.05);
    assert!((pts[7].0 - 50.0).abs() < 0.05 && (pts[7].1 - 23.61).abs() < 0.05);
    assert!((pts[6].0 - 80.90).abs() < 0.05 && pts[6].1.abs() < 0.05);
    assert!((pts[8].0 - 19.10).abs() < 0.05 && pts[8].1.abs() < 0.05);
}

#[test]
fn octagon_points_use_ooxml_default_adj() {
    let pts = octagon_points(0.0, 0.0, 100.0, 100.0);
    assert_eq!(pts.len(), 8);
    let x1 = 29.289;
    assert!((pts[1].0 - x1).abs() < 0.02 && (pts[1].1 - 100.0).abs() < 0.02);
    assert!((pts[3].0 - 100.0).abs() < 0.02 && (pts[3].1 - (100.0 - x1)).abs() < 0.02);
}

#[test]
fn heart_points_cleft_and_bottom_tip() {
    let pts = heart_points(0.0, 0.0, 100.0, 100.0);
    assert!(pts.len() >= 16, "{}", pts.len());
    assert!((pts[0].0 - 50.0).abs() < 0.05 && (pts[0].1 - 75.0).abs() < 0.05);
    assert!((pts[8].0 - 50.0).abs() < 0.05 && pts[8].1.abs() < 0.05);
    assert!(
        !pts.iter().any(|(x, y)| (x.abs() < 0.05 && y.abs() < 0.05)
            || ((x - 100.0).abs() < 0.05 && y.abs() < 0.05)),
        "heart must not include bbox corners; {pts:?}"
    );
}

#[test]
fn terminator_points_omit_bbox_corners() {
    let pts = flow_chart_terminator_points(0.0, 0.0, 100.0, 40.0);
    assert!(pts.len() >= 16, "{}", pts.len());
    let rx = 100.0 * 3475.0 / 21_600.0;
    assert!((pts[0].0 - rx).abs() < 0.05 && (pts[0].1 - 40.0).abs() < 0.05);
    assert!(
        !pts.iter().any(|(x, y)| x.abs() < 0.05 && y.abs() < 0.05),
        "stadium must not include the bbox corner; {pts:?}"
    );
    let left = pts
        .iter()
        .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
        .unwrap();
    assert!(
        left.0.abs() < 0.05 && (left.1 - 20.0).abs() < 1.0,
        "{left:?}"
    );
}

#[test]
fn heptagon_points_have_seven_vertices_apex_top() {
    let pts = heptagon_points(0.0, 0.0, 100.0, 100.0);
    assert_eq!(pts.len(), 7);
    assert!((pts[2].0 - 50.0).abs() < 0.05 && (pts[2].1 - 100.0).abs() < 0.05);
    assert!(pts[5].1.abs() < 0.05 && pts[6].1.abs() < 0.05);
}

#[test]
fn star6_points_have_six_tips() {
    let pts = star6_points(0.0, 0.0, 100.0, 100.0);
    assert_eq!(pts.len(), 12);
    assert!((pts[2].0 - 50.0).abs() < 0.05 && (pts[2].1 - 100.0).abs() < 0.05);
    assert!((pts[8].0 - 50.0).abs() < 0.05 && pts[8].1.abs() < 0.05);
    assert!(pts[0].0.abs() < 0.05 && (pts[4].0 - 100.0).abs() < 0.05);
}

#[test]
fn star7_points_have_seven_tips() {
    let pts = star7_points(0.0, 0.0, 100.0, 100.0);
    assert_eq!(pts.len(), 14);
    let top = pts[4];
    assert!(
        (top.0 - 50.0).abs() < 0.05 && (top.1 - 100.0).abs() < 0.05,
        "top tip is (hc,t); {top:?}"
    );
    assert!(
        pts[0].0.abs() < 0.2,
        "leftmost outer vertex on the left edge; {pts:?}"
    );
    assert!(
        (pts[8].0 - 100.0).abs() < 0.2,
        "rightmost outer vertex on the right edge; {pts:?}"
    );
}

#[test]
fn star8_points_have_eight_tips() {
    let pts = star8_points(0.0, 0.0, 100.0, 100.0);
    assert_eq!(pts.len(), 16);
    let start = pts[0];
    let top = pts[4];
    let right = pts[8];
    let bottom = pts[12];
    assert!(
        start.0.abs() < 0.05 && (start.1 - 50.0).abs() < 0.05,
        "start is (l,vc); {start:?}"
    );
    assert!(
        (top.0 - 50.0).abs() < 0.05 && (top.1 - 100.0).abs() < 0.05,
        "top tip is (hc,t); {top:?}"
    );
    assert!(
        (right.0 - 100.0).abs() < 0.05 && (right.1 - 50.0).abs() < 0.05,
        "right tip is (r,vc); {right:?}"
    );
    assert!(
        (bottom.0 - 50.0).abs() < 0.05 && bottom.1.abs() < 0.05,
        "bottom tip is (hc,b); {bottom:?}"
    );
}

#[test]
fn star10_points_have_ten_tips() {
    let pts = star10_points(0.0, 0.0, 100.0, 100.0);
    assert_eq!(pts.len(), 20);
    let top = pts[4];
    let bottom = pts[14];
    assert!(
        (top.0 - 50.0).abs() < 0.05 && (top.1 - 100.0).abs() < 0.05,
        "top tip is (hc,t); {top:?}"
    );
    assert!(
        (bottom.0 - 50.0).abs() < 0.05 && bottom.1.abs() < 0.05,
        "bottom tip is (hc,b); {bottom:?}"
    );
    assert!(
        pts[0].0.abs() < 0.2,
        "leftmost outer vertex on the left edge; {pts:?}"
    );
    assert!(
        (pts[8].0 - 100.0).abs() < 0.2,
        "rightmost outer vertex on the right edge; {pts:?}"
    );
}

#[test]
fn star12_points_have_twelve_tips() {
    let pts = star12_points(0.0, 0.0, 100.0, 100.0);
    assert_eq!(pts.len(), 24);
    let start = pts[0];
    let top = pts[6];
    let right = pts[12];
    let bottom = pts[18];
    assert!(
        start.0.abs() < 0.05 && (start.1 - 50.0).abs() < 0.05,
        "start is (l,vc); {start:?}"
    );
    assert!(
        (top.0 - 50.0).abs() < 0.05 && (top.1 - 100.0).abs() < 0.05,
        "top tip is (hc,t); {top:?}"
    );
    assert!(
        (right.0 - 100.0).abs() < 0.05 && (right.1 - 50.0).abs() < 0.05,
        "right tip is (r,vc); {right:?}"
    );
    assert!(
        (bottom.0 - 50.0).abs() < 0.05 && bottom.1.abs() < 0.05,
        "bottom tip is (hc,b); {bottom:?}"
    );
}

#[test]
fn star16_points_have_sixteen_tips() {
    let pts = star16_points(0.0, 0.0, 100.0, 100.0);
    assert_eq!(pts.len(), 32);
    let start = pts[0];
    let top = pts[8];
    let right = pts[16];
    let bottom = pts[24];
    assert!(
        start.0.abs() < 0.05 && (start.1 - 50.0).abs() < 0.05,
        "start is (l,vc); {start:?}"
    );
    assert!(
        (top.0 - 50.0).abs() < 0.05 && (top.1 - 100.0).abs() < 0.05,
        "top tip is (hc,t); {top:?}"
    );
    assert!(
        (right.0 - 100.0).abs() < 0.05 && (right.1 - 50.0).abs() < 0.05,
        "right tip is (r,vc); {right:?}"
    );
    assert!(
        (bottom.0 - 50.0).abs() < 0.05 && bottom.1.abs() < 0.05,
        "bottom tip is (hc,b); {bottom:?}"
    );
}

#[test]
fn star24_points_have_twenty_four_tips() {
    let pts = star24_points(0.0, 0.0, 100.0, 100.0);
    assert_eq!(pts.len(), 48);
    let start = pts[0];
    let top = pts[12];
    let right = pts[24];
    let bottom = pts[36];
    assert!(
        start.0.abs() < 0.05 && (start.1 - 50.0).abs() < 0.05,
        "start is (l,vc); {start:?}"
    );
    assert!(
        (top.0 - 50.0).abs() < 0.05 && (top.1 - 100.0).abs() < 0.05,
        "top tip is (hc,t); {top:?}"
    );
    assert!(
        (right.0 - 100.0).abs() < 0.05 && (right.1 - 50.0).abs() < 0.05,
        "right tip is (r,vc); {right:?}"
    );
    assert!(
        (bottom.0 - 50.0).abs() < 0.05 && bottom.1.abs() < 0.05,
        "bottom tip is (hc,b); {bottom:?}"
    );
}

#[test]
fn star32_points_have_thirty_two_tips() {
    let pts = star32_points(0.0, 0.0, 100.0, 100.0);
    assert_eq!(pts.len(), 64);
    let start = pts[0];
    let top = pts[16];
    let right = pts[32];
    let bottom = pts[48];
    assert!(
        start.0.abs() < 0.05 && (start.1 - 50.0).abs() < 0.05,
        "start is (l,vc); {start:?}"
    );
    assert!(
        (top.0 - 50.0).abs() < 0.05 && (top.1 - 100.0).abs() < 0.05,
        "top tip is (hc,t); {top:?}"
    );
    assert!(
        (right.0 - 100.0).abs() < 0.05 && (right.1 - 50.0).abs() < 0.05,
        "right tip is (r,vc); {right:?}"
    );
    assert!(
        (bottom.0 - 50.0).abs() < 0.05 && bottom.1.abs() < 0.05,
        "bottom tip is (hc,b); {bottom:?}"
    );
}

#[test]
fn flow_chart_document_has_a_wavy_bottom() {
    let pts = flow_chart_document_points(0.0, 0.0, 100.0, 100.0);
    assert!(pts.len() >= 10, "{}", pts.len());
    let start = pts[0];
    assert!(
        start.0.abs() < 0.05 && (start.1 - 100.0).abs() < 0.05,
        "start is top-left; {start:?}"
    );
    assert!(
        pts.iter()
            .any(|(px, py)| (*px - 100.0).abs() < 0.05 && (*py - 100.0).abs() < 0.05),
        "top-right corner; {pts:?}"
    );
    assert!(
        !pts.iter()
            .any(|(px, py)| (*px - 100.0).abs() < 0.05 && py.abs() < 0.05),
        "right side stops at y1, not the bbox corner; {pts:?}"
    );
    assert!(
        pts.iter().any(|(_, py)| *py < 5.0),
        "bottom cubic hangs below y1; {pts:?}"
    );
}

#[test]
fn flow_chart_offpage_connector_points_have_a_bottom_tip() {
    let pts = flow_chart_offpage_connector_points(0.0, 0.0, 100.0, 100.0);
    assert_eq!(pts.len(), 5);
    let start = pts[0];
    let tip = pts[3];
    assert!(
        start.0.abs() < 0.05 && (start.1 - 100.0).abs() < 0.05,
        "start is top-left; {start:?}"
    );
    assert!(
        (tip.0 - 50.0).abs() < 0.05 && tip.1.abs() < 0.05,
        "bottom tip is (hc,b); {tip:?}"
    );
    assert!(
        !pts.iter()
            .any(|(px, py)| (*px - 100.0).abs() < 0.05 && py.abs() < 0.05),
        "must not include the bbox corner; {pts:?}"
    );
}

#[test]
fn flow_chart_delay_is_a_d_not_a_rect() {
    let pts = flow_chart_delay_points(0.0, 0.0, 100.0, 100.0);
    assert!(pts.len() >= 8, "{}", pts.len());
    let start = pts[0];
    assert!(
        start.0.abs() < 0.05 && (start.1 - 100.0).abs() < 0.05,
        "start is (l,t); {start:?}"
    );
    assert!(
        pts.iter()
            .any(|(px, py)| (*px - 100.0).abs() < 1.0 && (*py - 50.0).abs() < 1.0),
        "right semicircle reaches (r,vc); {pts:?}"
    );
    assert!(
        !pts.iter()
            .any(|(px, py)| (*px - 100.0).abs() < 0.05 && py.abs() < 0.05),
        "must not include the bbox corner; {pts:?}"
    );
}

#[test]
fn flow_chart_manual_input_has_a_slanted_top() {
    let pts = flow_chart_manual_input_points(0.0, 0.0, 100.0, 100.0);
    assert_eq!(pts.len(), 4);
    let start = pts[0];
    assert!(
        start.0.abs() < 0.05 && (start.1 - 80.0).abs() < 0.05,
        "start is (l, hd5); {start:?}"
    );
    assert!(
        (pts[1].0 - 100.0).abs() < 0.05 && (pts[1].1 - 100.0).abs() < 0.05,
        "top-right is (r,t); {:?}",
        pts[1]
    );
    assert!(
        !pts.iter()
            .any(|(px, py)| px.abs() < 0.05 && (*py - 100.0).abs() < 0.05),
        "must not include the bbox top-left; {pts:?}"
    );
}

#[test]
fn flow_chart_punched_card_cuts_the_top_left() {
    let pts = flow_chart_punched_card_points(0.0, 0.0, 100.0, 100.0);
    assert_eq!(pts.len(), 5);
    let start = pts[0];
    assert!(
        start.0.abs() < 0.05 && (start.1 - 80.0).abs() < 0.05,
        "start is (l, hd5); {start:?}"
    );
    assert!(
        (pts[1].0 - 20.0).abs() < 0.05 && (pts[1].1 - 100.0).abs() < 0.05,
        "cut lands at (wd5, t); {:?}",
        pts[1]
    );
    assert!(
        !pts.iter()
            .any(|(px, py)| px.abs() < 0.05 && (*py - 100.0).abs() < 0.05),
        "must not include the bbox top-left; {pts:?}"
    );
}

#[test]
fn flow_chart_preparation_is_a_hexagon_not_a_rect() {
    let pts = flow_chart_preparation_points(0.0, 0.0, 100.0, 100.0);
    assert_eq!(pts.len(), 6);
    let start = pts[0];
    assert!(
        start.0.abs() < 0.05 && (start.1 - 50.0).abs() < 0.05,
        "start is (l,vc); {start:?}"
    );
    assert!(
        (pts[3].0 - 100.0).abs() < 0.05 && (pts[3].1 - 50.0).abs() < 0.05,
        "right tip is (r,vc); {:?}",
        pts[3]
    );
    assert!(
        !pts.iter()
            .any(|(px, py)| px.abs() < 0.05 && (*py - 100.0).abs() < 0.05),
        "must not include the bbox corner; {pts:?}"
    );
}

#[test]
fn flow_chart_extract_is_an_up_triangle() {
    let pts = flow_chart_extract_points(0.0, 0.0, 100.0, 100.0);
    assert_eq!(pts.len(), 3);
    assert!(
        pts[0].0.abs() < 0.05 && pts[0].1.abs() < 0.05,
        "start is bottom-left; {:?}",
        pts[0]
    );
    assert!(
        (pts[1].0 - 50.0).abs() < 0.05 && (pts[1].1 - 100.0).abs() < 0.05,
        "tip is (hc,t); {:?}",
        pts[1]
    );
    assert!(
        (pts[2].0 - 100.0).abs() < 0.05 && pts[2].1.abs() < 0.05,
        "end is bottom-right; {:?}",
        pts[2]
    );
}

#[test]
fn flow_chart_merge_is_a_down_triangle() {
    let pts = flow_chart_merge_points(0.0, 0.0, 100.0, 100.0);
    assert_eq!(pts.len(), 3);
    assert!(
        pts[0].0.abs() < 0.05 && (pts[0].1 - 100.0).abs() < 0.05,
        "start is top-left; {:?}",
        pts[0]
    );
    assert!(
        (pts[2].0 - 50.0).abs() < 0.05 && pts[2].1.abs() < 0.05,
        "tip is (hc,b); {:?}",
        pts[2]
    );
    assert!(
        !pts.iter()
            .any(|(px, py)| px.abs() < 0.05 && py.abs() < 0.05),
        "must not include the bbox corner; {pts:?}"
    );
}

#[test]
fn flow_chart_collate_has_a_waist() {
    let pts = flow_chart_collate_points(0.0, 0.0, 100.0, 100.0);
    assert_eq!(pts.len(), 6);
    let start = pts[0];
    let waist = pts[2];
    assert!(
        start.0.abs() < 0.05 && (start.1 - 100.0).abs() < 0.05,
        "start is top-left; {start:?}"
    );
    assert!(
        (waist.0 - 50.0).abs() < 0.05 && (waist.1 - 50.0).abs() < 0.05,
        "waist is (hc,vc); {waist:?}"
    );
}

#[test]
fn cloud_points_are_lobed_not_a_rect() {
    let pts = cloud_points(0.0, 0.0, 100.0, 100.0);
    assert!(pts.len() >= 40, "{}", pts.len());
    assert!((pts[0].0 - 9.028).abs() < 0.05 && (pts[0].1 - 66.736).abs() < 0.05);
    assert!(
        !pts.iter()
            .any(|(px, py)| px.abs() < 0.05 && py.abs() < 0.05),
        "cloud must not include the bbox corner; {pts:?}"
    );
    let min_x = pts.iter().map(|p| p.0).fold(f32::MAX, f32::min);
    let max_x = pts.iter().map(|p| p.0).fold(f32::MIN, f32::max);
    assert!(min_x < 5.0 && max_x > 90.0, "span {min_x}..{max_x}");
}

#[test]
fn pie_points_are_a_three_quarter_wedge() {
    let pts = pie_points(0.0, 0.0, 100.0, 100.0);
    assert!(pts.len() >= 8, "{}", pts.len());
    assert!((pts[0].0 - 100.0).abs() < 0.05 && (pts[0].1 - 50.0).abs() < 0.05);
    let last = pts[pts.len() - 1];
    assert!((last.0 - 50.0).abs() < 0.05 && (last.1 - 50.0).abs() < 0.05);
    let end = pts[pts.len() - 2];
    assert!(
        (end.0 - 50.0).abs() < 1.0 && (end.1 - 100.0).abs() < 1.0,
        "270° lands at top center; {end:?}"
    );
}

#[test]
fn left_right_arrow_points_have_two_tips() {
    let pts = left_right_arrow_points(0.0, 0.0, 100.0, 40.0);
    assert_eq!(pts.len(), 10);
    assert!(pts[0].0.abs() < 0.05 && (pts[0].1 - 20.0).abs() < 0.05);
    assert!((pts[5].0 - 100.0).abs() < 0.05 && (pts[5].1 - 20.0).abs() < 0.05);
    assert!((pts[1].0 - 20.0).abs() < 0.05 && (pts[1].1 - 40.0).abs() < 0.05);
}

#[test]
fn quad_arrow_points_have_four_tips() {
    let pts = quad_arrow_points(0.0, 0.0, 100.0, 100.0);
    assert_eq!(pts.len(), 24);
    // Left head: base at x1 = 22.5 spreads ±dx2 = 22.5 about vc; the
    // shaft is ±dx3 = 11.25 wide.
    assert!(
        (pts[1].0 - 22.5).abs() < 0.05 && (pts[1].1 - 72.5).abs() < 0.05,
        "{:?}",
        pts[1]
    );
    assert!(
        (pts[2].0 - 22.5).abs() < 0.05 && (pts[2].1 - 61.25).abs() < 0.05,
        "{:?}",
        pts[2]
    );
    assert!(
        (pts[3].0 - 38.75).abs() < 0.05 && (pts[3].1 - 61.25).abs() < 0.05,
        "{:?}",
        pts[3]
    );
    assert!(pts[0].0.abs() < 0.05 && (pts[0].1 - 50.0).abs() < 0.05);
    assert!((pts[6].0 - 50.0).abs() < 0.05 && (pts[6].1 - 100.0).abs() < 0.05);
    assert!((pts[12].0 - 100.0).abs() < 0.05 && (pts[12].1 - 50.0).abs() < 0.05);
    assert!((pts[18].0 - 50.0).abs() < 0.05 && pts[18].1.abs() < 0.05);
}

#[test]
fn lightning_bolt_points_are_a_zigzag() {
    let pts = lightning_bolt_points(0.0, 0.0, 100.0, 100.0);
    assert_eq!(pts.len(), 11);
    assert!((pts[0].0 - 39.222).abs() < 0.05 && (pts[0].1 - 100.0).abs() < 0.05);
    assert!((pts[5].0 - 100.0).abs() < 0.05 && pts[5].1.abs() < 0.05);
    assert!(pts[10].0.abs() < 0.05 && (pts[10].1 - 81.991).abs() < 0.05);
}

#[test]
fn sun_rays_have_eight_tips_and_a_disk() {
    let rays = sun_ray_points(0.0, 0.0, 100.0, 100.0);
    assert_eq!(rays.len(), 8);
    assert!((rays[0][0].0 - 100.0).abs() < 0.05 && (rays[0][0].1 - 50.0).abs() < 0.05);
    assert!(rays[4][0].0.abs() < 0.05 && (rays[4][0].1 - 50.0).abs() < 0.05);
    let disk = sun_disk_points(0.0, 0.0, 100.0, 100.0);
    assert_eq!(disk.len(), 24);
    let on_disk = disk.iter().all(|(px, py)| {
        let nx = (*px - 50.0) / 25.0;
        let ny = (*py - 50.0) / 25.0;
        (nx * nx + ny * ny - 1.0).abs() < 0.05
    });
    assert!(on_disk, "sun disk sits on the inner oval; {disk:?}");
}

#[test]
fn gear6_points_have_six_flat_teeth() {
    let pts = gear6_points(0.0, 0.0, 100.0, 100.0);
    assert_eq!(pts.len(), 24);
    let min_x = pts.iter().map(|p| p.0).fold(f32::MAX, f32::min);
    let max_x = pts.iter().map(|p| p.0).fold(f32::MIN, f32::max);
    assert!(min_x < 5.0 && max_x > 90.0, "span {min_x}..{max_x}");
    let outer = pts
        .iter()
        .filter(|(px, py)| {
            let dx = *px - 50.0;
            let dy = *py - 50.0;
            (dx * dx + dy * dy).sqrt() > 48.0
        })
        .count();
    assert_eq!(outer, 12, "six teeth × two outer vertices; {pts:?}");
    assert!(
        !pts.iter()
            .any(|(px, py)| px.abs() < 0.05 && py.abs() < 0.05),
        "gear6 must not include the bbox corner; {pts:?}"
    );
}

#[test]
fn gear9_points_have_nine_flat_teeth() {
    let pts = gear9_points(0.0, 0.0, 100.0, 100.0);
    assert_eq!(pts.len(), 36);
    let min_x = pts.iter().map(|p| p.0).fold(f32::MAX, f32::min);
    let max_x = pts.iter().map(|p| p.0).fold(f32::MIN, f32::max);
    assert!(min_x < 5.0 && max_x > 90.0, "span {min_x}..{max_x}");
    let outer = pts
        .iter()
        .filter(|(px, py)| {
            let dx = *px - 50.0;
            let dy = *py - 50.0;
            (dx * dx + dy * dy).sqrt() > 48.0
        })
        .count();
    assert_eq!(outer, 18, "nine teeth × two outer vertices; {pts:?}");
    assert!(
        !pts.iter()
            .any(|(px, py)| px.abs() < 0.05 && py.abs() < 0.05),
        "gear9 must not include the bbox corner; {pts:?}"
    );
}

#[test]
fn teardrop_points_tip_at_top_right() {
    let pts = teardrop_points(0.0, 0.0, 100.0, 100.0);
    assert!(pts.len() > 16, "teardrop is arcs+quads, not a box; {pts:?}");
    let tip = pts.iter().any(|(px, py)| *px > 95.0 && *py > 95.0);
    assert!(
        tip,
        "adj=100000 tip is OOXML (r,t) → PDF top-right; {pts:?}"
    );
    assert!(
        !pts.iter()
            .any(|(px, py)| px.abs() < 0.05 && py.abs() < 0.05),
        "teardrop must not include the bbox corner; {pts:?}"
    );
}

#[test]
fn plaque_points_cut_concave_corners() {
    let pts = plaque_points(0.0, 0.0, 100.0, 100.0);
    assert!(pts.len() > 12, "plaque is four inward arcs; {pts:?}");
    assert!(
        !pts.iter()
            .any(|(px, py)| px.abs() < 0.05 && py.abs() < 0.05),
        "plaque must not include the bbox corner; {pts:?}"
    );
    assert!(
        !pts.iter()
            .any(|(px, py)| (*px - 100.0).abs() < 0.05 && (*py - 100.0).abs() < 0.05),
        "plaque must not include the opposite bbox corner; {pts:?}"
    );
    let near_left = pts
        .iter()
        .any(|(px, py)| px.abs() < 0.5 && *py > 10.0 && *py < 90.0);
    assert!(near_left, "left edge after the top-left bite; {pts:?}");
}

#[test]
fn chord_points_are_an_arc_without_a_centre() {
    let pts = chord_points(0.0, 0.0, 100.0, 100.0);
    assert!(pts.len() >= 8, "{}", pts.len());
    let start = pts[0];
    assert!(
        start.0 > 80.0 && start.1 < 20.0,
        "45° start lower-right; {start:?}"
    );
    let end = *pts.last().expect("end");
    assert!(
        (end.0 - 50.0).abs() < 1.0 && (end.1 - 100.0).abs() < 1.0,
        "225° sweep lands at top center; {end:?}"
    );
    assert!(
        !pts.iter()
            .any(|(px, py)| (*px - 50.0).abs() < 0.5 && (*py - 50.0).abs() < 0.5),
        "chord must not include the pie centre; {pts:?}"
    );
}

#[test]
fn arc_points_are_a_quarter_wedge() {
    let pts = arc_points(0.0, 0.0, 100.0, 100.0);
    assert!(pts.len() >= 6, "{}", pts.len());
    let start = pts[0];
    assert!(
        (start.0 - 50.0).abs() < 0.05 && (start.1 - 100.0).abs() < 0.05,
        "270° start is top center; {start:?}"
    );
    let last = *pts.last().expect("centre");
    assert!(
        (last.0 - 50.0).abs() < 0.05 && (last.1 - 50.0).abs() < 0.05,
        "P0 closes through the centre; {last:?}"
    );
    let end = pts[pts.len() - 2];
    assert!(
        (end.0 - 100.0).abs() < 1.0 && (end.1 - 50.0).abs() < 1.0,
        "90° sweep lands at right center; {end:?}"
    );
}

#[test]
fn snip1_rect_points_cut_the_top_right_corner() {
    let pts = snip1_rect_points(0.0, 0.0, 100.0, 100.0);
    assert_eq!(pts.len(), 5, "{pts:?}");
    let start = pts[0];
    assert!(
        start.0.abs() < 0.05 && (start.1 - 100.0).abs() < 0.05,
        "moveTo (l,t) is PDF top-left; {start:?}"
    );
    assert!(
        !pts.iter()
            .any(|(px, py)| (*px - 100.0).abs() < 0.05 && (*py - 100.0).abs() < 0.05),
        "must not include the snipped top-right corner; {pts:?}"
    );
    assert!(
        pts.iter()
            .any(|(px, py)| (*px - 100.0).abs() < 0.05 && (*py - 83.333).abs() < 0.05),
        "snip lands on the right edge at dx1; {pts:?}"
    );
}

#[test]
fn round1_rect_points_round_the_top_right_corner() {
    let pts = round1_rect_points(0.0, 0.0, 100.0, 100.0);
    assert!(pts.len() >= 8, "{}", pts.len());
    let start = pts[0];
    assert!(
        start.0.abs() < 0.05 && (start.1 - 100.0).abs() < 0.05,
        "moveTo (l,t) is PDF top-left; {start:?}"
    );
    assert!(
        !pts.iter()
            .any(|(px, py)| (*px - 100.0).abs() < 0.05 && (*py - 100.0).abs() < 0.05),
        "must not include the sharp top-right corner; {pts:?}"
    );
    let last = *pts.last().expect("end");
    assert!(
        last.0.abs() < 0.05 && last.1.abs() < 0.05,
        "close vertex is PDF bottom-left; {last:?}"
    );
    assert!(
        pts.iter()
            .any(|(px, py)| *px > 90.0 && *py > 90.0 && *px < 100.0 && *py < 100.0),
        "arc samples sit inside the top-right corner; {pts:?}"
    );
}

#[test]
fn snip2_same_rect_points_cut_both_top_corners() {
    let pts = snip2_same_rect_points(0.0, 0.0, 100.0, 100.0);
    assert_eq!(pts.len(), 6, "{pts:?}");
    let start = pts[0];
    assert!(
        (start.0 - 16.667).abs() < 0.05 && (start.1 - 100.0).abs() < 0.05,
        "moveTo (tx1,t) is PDF top edge after left snip; {start:?}"
    );
    assert!(
        !pts.iter()
            .any(|(px, py)| px.abs() < 0.05 && (*py - 100.0).abs() < 0.05),
        "must not include the snipped top-left corner; {pts:?}"
    );
    assert!(
        !pts.iter()
            .any(|(px, py)| (*px - 100.0).abs() < 0.05 && (*py - 100.0).abs() < 0.05),
        "must not include the snipped top-right corner; {pts:?}"
    );
    assert!(
        pts.iter()
            .any(|(px, py)| px.abs() < 0.05 && (*py - 83.333).abs() < 0.05),
        "left snip lands on the left edge at tx1; {pts:?}"
    );
    assert!(
        pts.iter()
            .any(|(px, py)| px.abs() < 0.05 && py.abs() < 0.05),
        "bottom-left stays square (adj2=0); {pts:?}"
    );
}

#[test]
fn round2_same_rect_points_round_both_top_corners() {
    let pts = round2_same_rect_points(0.0, 0.0, 100.0, 100.0);
    assert!(pts.len() >= 10, "{}", pts.len());
    let start = pts[0];
    assert!(
        (start.0 - 16.667).abs() < 0.05 && (start.1 - 100.0).abs() < 0.05,
        "moveTo (tx1,t) is PDF top edge after left radius; {start:?}"
    );
    assert!(
        !pts.iter()
            .any(|(px, py)| px.abs() < 0.05 && (*py - 100.0).abs() < 0.05),
        "must not include the sharp top-left corner; {pts:?}"
    );
    assert!(
        !pts.iter()
            .any(|(px, py)| (*px - 100.0).abs() < 0.05 && (*py - 100.0).abs() < 0.05),
        "must not include the sharp top-right corner; {pts:?}"
    );
    assert!(
        pts.iter()
            .any(|(px, py)| px.abs() < 0.05 && py.abs() < 0.05),
        "bottom-left stays square (adj2=0); {pts:?}"
    );
    assert!(
        pts.iter()
            .any(|(px, py)| *px > 90.0 && *py > 90.0 && *px < 100.0 && *py < 100.0),
        "arc samples sit inside the top-right corner; {pts:?}"
    );
}

#[test]
fn snip2_diag_rect_points_cut_opposite_corners() {
    let pts = snip2_diag_rect_points(0.0, 0.0, 100.0, 100.0);
    assert_eq!(pts.len(), 6, "{pts:?}");
    let start = pts[0];
    assert!(
        start.0.abs() < 0.05 && (start.1 - 100.0).abs() < 0.05,
        "moveTo (l,t) is PDF top-left; {start:?}"
    );
    assert!(
        !pts.iter()
            .any(|(px, py)| (*px - 100.0).abs() < 0.05 && (*py - 100.0).abs() < 0.05),
        "must not include the snipped top-right corner; {pts:?}"
    );
    assert!(
        !pts.iter()
            .any(|(px, py)| px.abs() < 0.05 && py.abs() < 0.05),
        "must not include the snipped bottom-left corner; {pts:?}"
    );
    assert!(
        pts.iter()
            .any(|(px, py)| (*px - 100.0).abs() < 0.05 && py.abs() < 0.05),
        "bottom-right stays square; {pts:?}"
    );
}

#[test]
fn round2_diag_rect_points_round_opposite_corners() {
    let pts = round2_diag_rect_points(0.0, 0.0, 100.0, 100.0);
    assert!(pts.len() >= 10, "{}", pts.len());
    let start = pts[0];
    assert!(
        (start.0 - 16.667).abs() < 0.05 && (start.1 - 100.0).abs() < 0.05,
        "moveTo (x1,t) is PDF top edge after left radius; {start:?}"
    );
    assert!(
        !pts.iter()
            .any(|(px, py)| px.abs() < 0.05 && (*py - 100.0).abs() < 0.05),
        "must not include the sharp top-left corner; {pts:?}"
    );
    assert!(
        pts.iter()
            .any(|(px, py)| (*px - 100.0).abs() < 0.05 && (*py - 100.0).abs() < 0.05),
        "top-right stays square (adj2=0); {pts:?}"
    );
    assert!(
        pts.iter()
            .any(|(px, py)| px.abs() < 0.05 && py.abs() < 0.05),
        "bottom-left stays square (adj2=0); {pts:?}"
    );
    assert!(
        !pts.iter()
            .any(|(px, py)| (*px - 100.0).abs() < 0.05 && py.abs() < 0.05),
        "must not include the sharp bottom-right corner; {pts:?}"
    );
}

#[test]
fn wave_points_undulate_top_and_bottom() {
    let pts = wave_points(0.0, 0.0, 100.0, 100.0);
    assert!(pts.len() >= 16, "{}", pts.len());
    let start = pts[0];
    assert!(
        start.0.abs() < 0.05 && (start.1 - 87.5).abs() < 0.2,
        "{start:?}"
    );
    let crest = pts.iter().any(|(_, py)| *py > 95.0);
    let trough = pts.iter().any(|(_, py)| *py < 80.0 && *py > 60.0);
    assert!(crest, "top cubic crests above y1; {pts:?}");
    assert!(trough, "top cubic troughs below y1; {pts:?}");
}

#[test]
fn double_wave_points_have_two_humps() {
    let pts = double_wave_points(0.0, 0.0, 100.0, 100.0);
    assert!(pts.len() >= 24, "{}", pts.len());
    let start = pts[0];
    assert!(
        start.0.abs() < 0.05 && (start.1 - 93.75).abs() < 0.2,
        "start is (l,y1); {start:?}"
    );
    assert!(
        !pts.iter()
            .any(|(px, py)| px.abs() < 0.05 && (*py - 100.0).abs() < 0.05),
        "must not include the bbox corner; {pts:?}"
    );
}

#[test]
fn flow_chart_display_points_start_at_left_mid() {
    let pts = flow_chart_display_points(0.0, 0.0, 100.0, 100.0);
    assert!(pts.len() >= 8, "{}", pts.len());
    let start = pts[0];
    assert!(
        start.0.abs() < 0.05 && (start.1 - 50.0).abs() < 0.05,
        "start is (l,vc); {start:?}"
    );
    assert!(
        pts.iter()
            .any(|(px, py)| (*px - 100.0).abs() < 1.0 && (*py - 50.0).abs() < 1.0),
        "right semicircle reaches (r,vc); {pts:?}"
    );
}

#[test]
fn flow_chart_input_output_points_are_parallelogram() {
    let pts = flow_chart_input_output_points(0.0, 0.0, 100.0, 100.0);
    assert_eq!(pts.len(), 4, "{pts:?}");
    let start = pts[0];
    assert!(
        start.0.abs() < 0.05 && start.1.abs() < 0.05,
        "start is (l,b) PDF (0,0); {start:?}"
    );
    assert!(
        pts.iter()
            .any(|(px, py)| (*px - 20.0).abs() < 0.05 && (*py - 100.0).abs() < 0.05),
        "top-left after slant (wd5,t); {pts:?}"
    );
    assert!(
        pts.iter()
            .any(|(px, py)| (*px - 100.0).abs() < 0.05 && (*py - 100.0).abs() < 0.05),
        "top-right (r,t); {pts:?}"
    );
    assert!(
        pts.iter()
            .any(|(px, py)| (*px - 80.0).abs() < 0.05 && py.abs() < 0.05),
        "bottom-right after slant (4/5 w,b); {pts:?}"
    );
    assert!(
        !pts.iter()
            .any(|(px, py)| px.abs() < 0.05 && (*py - 100.0).abs() < 0.05),
        "must not include sharp top-left; {pts:?}"
    );
    assert!(
        !pts.iter()
            .any(|(px, py)| (*px - 100.0).abs() < 0.05 && py.abs() < 0.05),
        "must not include sharp bottom-right; {pts:?}"
    );
}

#[test]
fn flow_chart_manual_operation_points_are_bottom_inset_trapezoid() {
    let pts = flow_chart_manual_operation_points(0.0, 0.0, 100.0, 100.0);
    assert_eq!(pts.len(), 4, "{pts:?}");
    let start = pts[0];
    assert!(
        start.0.abs() < 0.05 && (start.1 - 100.0).abs() < 0.05,
        "start is (l,t) PDF (0,100); {start:?}"
    );
    assert!(
        pts.iter()
            .any(|(px, py)| (*px - 100.0).abs() < 0.05 && (*py - 100.0).abs() < 0.05),
        "top-right (r,t); {pts:?}"
    );
    assert!(
        pts.iter()
            .any(|(px, py)| (*px - 80.0).abs() < 0.05 && py.abs() < 0.05),
        "bottom-right inset (4/5 w,b); {pts:?}"
    );
    assert!(
        pts.iter()
            .any(|(px, py)| (*px - 20.0).abs() < 0.05 && py.abs() < 0.05),
        "bottom-left inset (wd5,b); {pts:?}"
    );
    assert!(
        !pts.iter()
            .any(|(px, py)| px.abs() < 0.05 && py.abs() < 0.05),
        "must not include sharp bottom-left; {pts:?}"
    );
    assert!(
        !pts.iter()
            .any(|(px, py)| (*px - 100.0).abs() < 0.05 && py.abs() < 0.05),
        "must not include sharp bottom-right; {pts:?}"
    );
}

#[test]
fn flow_chart_online_storage_points_start_inset_top() {
    let pts = flow_chart_online_storage_points(0.0, 0.0, 100.0, 100.0);
    assert!(pts.len() >= 8, "{}", pts.len());
    let start = pts[0];
    assert!(
        (start.0 - 100.0 / 6.0).abs() < 0.05 && (start.1 - 100.0).abs() < 0.05,
        "start is (wd6,t) PDF; {start:?}"
    );
    assert!(
        pts.iter().any(|(px, _)| px.abs() < 1.0),
        "left cap reaches l; {pts:?}"
    );
    assert!(
        pts.iter().any(|(_, py)| py.abs() < 1.0),
        "bottom edge present; {pts:?}"
    );
}

#[test]
fn pentagon_points_have_apex_at_top_center() {
    let pts = pentagon_points(0.0, 0.0, 100.0, 100.0);
    assert_eq!(pts.len(), 5);
    assert!((pts[1].0 - 50.0).abs() < 0.05 && (pts[1].1 - 100.0).abs() < 0.05);
    assert!(
        pts.iter()
            .all(|(px, py)| *px >= -1.0 && *px <= 101.0 && *py >= -1.0 && *py <= 101.0)
    );
}

#[test]
fn plus_points_use_ooxml_default_arm() {
    let pts = plus_points(0.0, 0.0, 100.0, 100.0);
    assert_eq!(pts.len(), 12);
    assert!((pts[2].0 - 25.0).abs() < 0.01 && (pts[2].1 - 100.0).abs() < 0.01);
    assert!((pts[5].0 - 100.0).abs() < 0.01 && (pts[5].1 - 75.0).abs() < 0.01);
}

#[test]
fn chevron_points_use_ooxml_default_adj() {
    let pts = chevron_points(0.0, 0.0, 100.0, 40.0);
    assert_eq!(pts.len(), 6);
    assert!((pts[2].0 - 100.0).abs() < 0.01 && (pts[2].1 - 20.0).abs() < 0.01);
    assert!((pts[5].0 - 20.0).abs() < 0.01 && (pts[5].1 - 20.0).abs() < 0.01);
}

#[test]
fn hexagon_points_use_ooxml_default_inset() {
    let pts = hexagon_points(0.0, 0.0, 100.0, 40.0);
    assert_eq!(pts.len(), 6);
    assert!((pts[0].0 - 25.0).abs() < 0.01 && (pts[0].1 - 40.0).abs() < 0.01);
    assert!((pts[2].0 - 100.0).abs() < 0.01 && (pts[2].1 - 20.0).abs() < 0.01);
}

#[test]
fn round_rect_points_use_ooxml_default_radius() {
    let pts = round_rect_points(0.0, 0.0, 120.0, 60.0);
    let r = 60.0 * 16_667.0 / 100_000.0;
    assert!(pts.len() > 8, "arcs must add vertices; n={}", pts.len());
    assert!(
        pts.iter().any(|(x, y)| (*x - r).abs() < 0.05 && *y < 0.05),
        "bottom edge starts after radius; {pts:?}"
    );
    let sharp_corner = pts.iter().any(|(x, y)| x.abs() < 0.05 && y.abs() < 0.05);
    assert!(
        !sharp_corner,
        "must not include the sharp (0,0) corner; {pts:?}"
    );
    let inset = pts
        .iter()
        .any(|(x, y)| *x > 0.5 && *x < r && *y > 0.5 && *y < r);
    assert!(inset, "quarter-arc vertices sit inside the corner; {pts:?}");
}

#[test]
fn wrap_runs_keeps_a_url_whole_across_its_slashes() {
    // fixtures_500 001c1554: Word never breaks a URL after `/` (0 of
    // 34 mid-URL breaks in fixtures_500; 24 after `-`, 10 at the
    // edge). "…2024:  https://drive.google.com/" | "drive/…" was ours;
    // Word moves the whole URL down.
    let fonts = Fonts::new();
    let text = "Adviento 2024: https://drive.google.com/drive/folders/1_jNG4oVykh98IEmHS";
    let run = TextRun::new(text.to_string(), default_run_style());
    let lines = wrap_runs(&fonts, std::slice::from_ref(&run), 300.0, 300.0, false);
    let joined: Vec<String> = lines
        .iter()
        .map(|l| l.iter().map(|r| r.text.as_str()).collect())
        .collect();
    assert!(
        !joined[0].contains("https"),
        "the URL moves down whole; {joined:?}"
    );
}

#[test]
fn wrap_runs_breaks_https_url_at_hyphen() {
    // comments-lots appendix: Word wraps
    // `https://learn.microsoft.com/en-` then `us/purview/…`.
    // Whole-token overflow (Test 7 lock) parks the Copilot URL as
    // one 536pt line on a 486pt measure. Not generic character-break
    // (mini 57 / table-gated −24 ITT).
    let fonts = Fonts::new();
    let url = "https://learn.microsoft.com/en-us/microsoft-365/copilot/microsoft-365-copilot-architecture-data-protection-auditing";
    let run = TextRun::new(url.to_string(), default_run_style());
    let lines = wrap_runs(&fonts, std::slice::from_ref(&run), 400.0, 400.0, false);
    assert!(
        lines.len() >= 2,
        "Word wraps this URL at / and -; n={} {:?}",
        lines.len(),
        lines
            .iter()
            .map(|l| l.iter().map(|r| r.text.as_str()).collect::<String>())
            .collect::<Vec<_>>()
    );
    let joined: Vec<String> = lines
        .iter()
        .map(|l| l.iter().map(|r| r.text.as_str()).collect())
        .collect();
    assert!(
        joined[0].contains("https://"),
        "first line keeps the scheme; {joined:?}"
    );
    assert!(
        joined.iter().skip(1).any(|s| s.contains("copilot")
            || s.contains("auditing")
            || s.contains("microsoft-365")),
        "later line is a URL tail, not a second copy of https://; {joined:?}"
    );
}

#[test]
fn iso_strict_tab_val_end_is_right_and_start_is_left() {
    // Strict01 / file_100 TOC: ISO Strict writes w:tab val=end
    // (ECMA ST_TabJc; LTR end = right). Mapping it through the
    // left default parked PAGEREF as a left stop at pos.
    let xml = r#"<?xml version="1.0"?>
<w:p xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:pPr>
    <w:tabs>
      <w:tab w:val="end" w:leader="dot" w:pos="9350"/>
      <w:tab w:val="start" w:pos="1440"/>
      <w:tab w:val="right" w:pos="8640"/>
    </w:tabs>
  </w:pPr>
</w:p>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let ppr = first_named(&dom, root, "pPr").expect("pPr");
    let stops = parse_tab_stops(&dom, ppr);
    assert_eq!(stops.len(), 3);
    assert!(stops[0].align == TabAlign::Left, "start is LTR left");
    assert!((stops[0].pos - twip(1440.0)).abs() < 0.05);
    assert!(stops[1].align == TabAlign::Right, "right stays right");
    assert!(stops[2].align == TabAlign::Right, "end is LTR right");
    assert!((stops[2].pos - twip(9350.0)).abs() < 0.05);
}

#[test]
fn right_arrow_points_match_ooxml_default() {
    // Word Strict01 rightArrow: adj1=adj2=50000, extent ~91.3×25.25.
    let x = 227.8001;
    let y = 664.6999;
    let dw = 91.3;
    let dh = 25.25;
    let pts = right_arrow_points(x, y, dw, dh);
    assert_eq!(pts.len(), 7, "{pts:?}");
    let (px, py) = pts[3];
    assert!(
        (px - (x + dw)).abs() < 0.02 && (py - (y + dh * 0.5)).abs() < 0.02,
        "tip must be (right, mid); {pts:?}"
    );
    let head = pts[3].0 - pts[2].0;
    assert!(
        (head - dh * 0.5).abs() < 0.02,
        "head width is min(w,h)/2; head={head} pts={pts:?}"
    );
    assert!(
        (pts[0].1 - (y + dh * 0.75)).abs() < 0.02 && (pts[6].1 - (y + dh * 0.25)).abs() < 0.02,
        "shaft is the middle 50%; {pts:?}"
    );
}

#[test]
fn parse_radar_chart_kind_is_radar() {
    let xml = r#"<?xml version="1.0"?>
<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart">
<c:chart><c:plotArea><c:radarChart>
  <c:ser>
    <c:cat><c:strLit>
      <c:pt idx="0"><c:v>N</c:v></c:pt>
      <c:pt idx="1"><c:v>E</c:v></c:pt>
      <c:pt idx="2"><c:v>S</c:v></c:pt>
      <c:pt idx="3"><c:v>W</c:v></c:pt>
    </c:strLit></c:cat>
    <c:val><c:numLit>
      <c:pt idx="0"><c:v>1</c:v></c:pt>
      <c:pt idx="1"><c:v>1</c:v></c:pt>
      <c:pt idx="2"><c:v>1</c:v></c:pt>
      <c:pt idx="3"><c:v>1</c:v></c:pt>
    </c:numLit></c:val>
  </c:ser>
</c:radarChart></c:plotArea></c:chart></c:chartSpace>"#;
    let data = parse_chart(xml).expect("radar");
    assert_eq!(
        data.kind,
        ChartKind::Radar,
        "c:radarChart must not parse as Bar"
    );
    assert_eq!(data.series.len(), 1);
    assert_eq!(data.cats, ["N", "E", "S", "W"]);
    assert_eq!(data.series[0].len(), 4);
}

#[test]
fn radar_chart_polygon_points_start_at_twelve_oclock() {
    let pts = radar_chart_polygon_points(0.0, 0.0, 100.0, 100.0, &[1.0, 1.0, 1.0, 1.0], 1.0);
    assert_eq!(pts.len(), 4, "{pts:?}");
    assert!(
        (pts[0].0 - 50.0).abs() < 0.05 && (pts[0].1 - 100.0).abs() < 0.05,
        "first vertex is 12 o'clock; {pts:?}"
    );
    assert!(
        (pts[1].0 - 100.0).abs() < 0.05 && (pts[1].1 - 50.0).abs() < 0.05,
        "second vertex is 3 o'clock clockwise; {pts:?}"
    );
    assert!(
        (pts[2].0 - 50.0).abs() < 0.05 && pts[2].1.abs() < 0.05,
        "third vertex is 6 o'clock; {pts:?}"
    );
    assert!(
        pts[3].0.abs() < 0.05 && (pts[3].1 - 50.0).abs() < 0.05,
        "fourth vertex is 9 o'clock; {pts:?}"
    );
}

#[test]
fn parse_scatter_chart_kind_is_scatter() {
    let xml = r#"<?xml version="1.0"?>
<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart">
<c:chart><c:plotArea><c:scatterChart>
  <c:ser>
    <c:xVal><c:numLit>
      <c:pt idx="0"><c:v>1</c:v></c:pt>
      <c:pt idx="1"><c:v>3</c:v></c:pt>
    </c:numLit></c:xVal>
    <c:yVal><c:numLit>
      <c:pt idx="0"><c:v>2</c:v></c:pt>
      <c:pt idx="1"><c:v>4</c:v></c:pt>
    </c:numLit></c:yVal>
  </c:ser>
</c:scatterChart></c:plotArea></c:chart></c:chartSpace>"#;
    let data = parse_chart(xml).expect("c:scatterChart must parse");
    assert_eq!(
        data.kind,
        ChartKind::Scatter,
        "c:scatterChart must not parse as Bar"
    );
    assert_eq!(data.cats, ["1", "3"]);
    assert_eq!(data.series.len(), 1);
    assert!((data.series[0][0] - 2.0).abs() < 0.01);
    assert!((data.series[0][1] - 4.0).abs() < 0.01);
}

#[test]
fn scatter_chart_marker_points_map_xy() {
    let pts = scatter_chart_marker_points(
        0.0,
        0.0,
        100.0,
        40.0,
        &[(0.0, 0.0), (4.0, 4.0)],
        (0.0, 4.0),
        (0.0, 4.0),
    );
    assert_eq!(pts.len(), 2, "{pts:?}");
    assert!(pts[0].0.abs() < 0.05 && pts[0].1.abs() < 0.05, "{pts:?}");
    assert!(
        (pts[1].0 - 100.0).abs() < 0.05 && (pts[1].1 - 40.0).abs() < 0.05,
        "{pts:?}"
    );
}

#[test]
fn line_series_share_category_slots_and_keep_point_idx() {
    // #105: series of 3 and 2 values share 3 slots, and a sparse point
    // (idx 2 of 3) keeps slot 2 with a gap before it.
    let xml = r#"<?xml version="1.0"?>
<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart">
<c:chart><c:plotArea><c:lineChart>
  <c:ser><c:val><c:numLit><c:ptCount val="3"/>
    <c:pt idx="0"><c:v>1</c:v></c:pt><c:pt idx="1"><c:v>2</c:v></c:pt><c:pt idx="2"><c:v>3</c:v></c:pt>
  </c:numLit></c:val></c:ser>
  <c:ser><c:val><c:numLit><c:ptCount val="3"/>
    <c:pt idx="0"><c:v>4</c:v></c:pt><c:pt idx="2"><c:v>5</c:v></c:pt>
  </c:numLit></c:val></c:ser>
</c:lineChart></c:plotArea></c:chart></c:chartSpace>"#;
    let data = parse_chart(xml).expect("line");
    assert_eq!(data.n_cats, 3);
    assert_eq!(data.indexed[1], vec![(0, 4.0), (2, 5.0)]);
    let runs = line_chart_runs(&data.indexed[1], data.n_cats);
    assert_eq!(
        runs,
        vec![vec![(0, 4.0)], vec![(2, 5.0)]],
        "a missing idx breaks the line"
    );
}

#[test]
fn scatter_pairs_every_series_by_idx_and_scales_like_word() {
    // #107: both series paint, each with its own xVal; negatives stay
    // negative; c:scaling wins; otherwise min(0, lo) .. max(1, hi).
    let xml = r#"<?xml version="1.0"?>
<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart">
<c:chart><c:plotArea><c:scatterChart>
  <c:ser>
    <c:xVal><c:numLit><c:pt idx="0"><c:v>-2</c:v></c:pt><c:pt idx="1"><c:v>4</c:v></c:pt></c:numLit></c:xVal>
    <c:yVal><c:numLit><c:pt idx="0"><c:v>1</c:v></c:pt><c:pt idx="1"><c:v>8</c:v></c:pt></c:numLit></c:yVal>
  </c:ser>
  <c:ser>
    <c:xVal><c:numLit><c:pt idx="1"><c:v>10</c:v></c:pt></c:numLit></c:xVal>
    <c:yVal><c:numLit><c:pt idx="0"><c:v>3</c:v></c:pt><c:pt idx="1"><c:v>6</c:v></c:pt></c:numLit></c:yVal>
  </c:ser>
</c:scatterChart>
<c:valAx><c:scaling><c:max val="20"/></c:scaling><c:axPos val="b"/></c:valAx>
<c:valAx><c:scaling/><c:axPos val="l"/></c:valAx>
</c:plotArea></c:chart></c:chartSpace>"#;
    let data = parse_chart(xml).expect("scatter");
    let pairs = scatter_series_pairs(&data);
    assert_eq!(pairs[0], vec![(-2.0, 1.0), (4.0, 8.0)]);
    assert_eq!(
        pairs[1],
        vec![(10.0, 6.0)],
        "y idx 0 has no x: not fabricated"
    );
    assert_eq!(data.x_scaling, (None, Some(20.0)));
    let bx = scatter_axis_bounds(pairs.iter().flatten().map(|p| p.0), data.x_scaling);
    assert_eq!(bx, (-2.0, 20.0));
    let by = scatter_axis_bounds(pairs.iter().flatten().map(|p| p.1), data.y_scaling);
    assert_eq!(by, (0.0, 8.0), "positive data keeps a zero minimum");
    assert_eq!(
        scatter_axis_bounds([5.0_f32].into_iter(), (Some(5.0), Some(5.0))),
        (5.0, 6.0)
    );
}

#[test]
fn parse_area_chart_kind_is_area() {
    let xml = r#"<?xml version="1.0"?>
<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart">
<c:chart><c:plotArea><c:areaChart>
  <c:ser>
    <c:cat><c:strLit>
      <c:pt idx="0"><c:v>A</c:v></c:pt>
      <c:pt idx="1"><c:v>B</c:v></c:pt>
    </c:strLit></c:cat>
    <c:val><c:numLit>
      <c:pt idx="0"><c:v>1</c:v></c:pt>
      <c:pt idx="1"><c:v>3</c:v></c:pt>
    </c:numLit></c:val>
  </c:ser>
</c:areaChart></c:plotArea></c:chart></c:chartSpace>"#;
    let data = parse_chart(xml).expect("area");
    assert_eq!(
        data.kind,
        ChartKind::Area,
        "c:areaChart must not parse as Bar"
    );
    assert_eq!(data.series.len(), 1);
    assert!((data.series[0][0] - 1.0).abs() < 0.01);
    assert!((data.series[0][1] - 3.0).abs() < 0.01);
    assert_eq!(data.cats, ["A", "B"]);
}

#[test]
fn area_chart_fill_points_close_to_baseline() {
    let pts = area_chart_fill_points(0.0, 0.0, 100.0, 40.0, &[1.0, 3.0], 4.0);
    assert_eq!(pts.len(), 4, "{pts:?}");
    assert!(
        (pts[0].0 - 25.0).abs() < 0.05 && (pts[0].1 - 10.0).abs() < 0.05,
        "first plot point; {pts:?}"
    );
    assert!(
        (pts[1].0 - 75.0).abs() < 0.05 && (pts[1].1 - 30.0).abs() < 0.05,
        "second plot point; {pts:?}"
    );
    assert!(
        (pts[2].0 - 75.0).abs() < 0.05 && pts[2].1.abs() < 0.05,
        "drops to baseline at last x; {pts:?}"
    );
    assert!(
        (pts[3].0 - 25.0).abs() < 0.05 && pts[3].1.abs() < 0.05,
        "baseline back to first x; {pts:?}"
    );
}

#[test]
fn parse_line_chart_kind_is_line() {
    let xml = r#"<?xml version="1.0"?>
<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart">
<c:chart><c:plotArea><c:lineChart>
  <c:ser>
    <c:cat><c:strLit>
      <c:pt idx="0"><c:v>A</c:v></c:pt>
      <c:pt idx="1"><c:v>B</c:v></c:pt>
    </c:strLit></c:cat>
    <c:val><c:numLit>
      <c:pt idx="0"><c:v>1</c:v></c:pt>
      <c:pt idx="1"><c:v>3</c:v></c:pt>
    </c:numLit></c:val>
  </c:ser>
</c:lineChart></c:plotArea></c:chart></c:chartSpace>"#;
    let data = parse_chart(xml).expect("line");
    assert_eq!(
        data.kind,
        ChartKind::Line,
        "c:lineChart must not parse as Bar"
    );
    assert_eq!(data.series.len(), 1);
    assert!((data.series[0][0] - 1.0).abs() < 0.01);
    assert!((data.series[0][1] - 3.0).abs() < 0.01);
}

#[test]
fn line_chart_polyline_points_follow_values() {
    let pts = line_chart_polyline_points(0.0, 0.0, 100.0, 40.0, &[0.0, 4.0], 4.0);
    assert_eq!(pts.len(), 2, "{pts:?}");
    assert!(
        (pts[0].0 - 25.0).abs() < 0.05 && pts[0].1.abs() < 0.05,
        "{pts:?}"
    );
    assert!(
        (pts[1].0 - 75.0).abs() < 0.05 && (pts[1].1 - 40.0).abs() < 0.05,
        "{pts:?}"
    );
}

#[test]
fn parse_pie_chart_kind_is_pie() {
    let xml = r#"<?xml version="1.0"?>
<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart">
<c:chart><c:plotArea><c:pieChart>
  <c:ser>
    <c:cat><c:strLit>
      <c:pt idx="0"><c:v>A</c:v></c:pt>
      <c:pt idx="1"><c:v>B</c:v></c:pt>
    </c:strLit></c:cat>
    <c:val><c:numLit>
      <c:pt idx="0"><c:v>1</c:v></c:pt>
      <c:pt idx="1"><c:v>3</c:v></c:pt>
    </c:numLit></c:val>
  </c:ser>
</c:pieChart></c:plotArea></c:chart></c:chartSpace>"#;
    let data = parse_chart(xml).expect("pie");
    assert_eq!(
        data.kind,
        ChartKind::Pie,
        "c:pieChart must not parse as Bar"
    );
    assert_eq!(data.series.len(), 1);
    assert!((data.series[0][0] - 1.0).abs() < 0.01);
    assert!((data.series[0][1] - 3.0).abs() < 0.01);
    assert_eq!(data.cats, ["A", "B"]);
}

#[test]
fn pie_chart_wedge_points_first_slice_is_a_quarter() {
    // 1 of 4 → 90° clockwise from 12 o'clock (stAng=3cd4, swAng=cd4).
    let pts = pie_chart_wedge_points(0.0, 0.0, 100.0, 100.0, 16_200_000.0, 5_400_000.0);
    assert!(pts.len() >= 6, "{}", pts.len());
    let start = pts[0];
    assert!(
        (start.0 - 50.0).abs() < 0.5 && (start.1 - 100.0).abs() < 0.5,
        "start is 12 o'clock; {start:?}"
    );
    let last = *pts.last().expect("center");
    assert!(
        (last.0 - 50.0).abs() < 0.05 && (last.1 - 50.0).abs() < 0.05,
        "closes through centre; {last:?}"
    );
    assert!(
        pts.iter()
            .any(|(px, py)| (*px - 100.0).abs() < 1.0 && (*py - 50.0).abs() < 1.0),
        "90° clockwise lands at 3 o'clock; {pts:?}"
    );
}

#[test]
fn parse_bar_chart_reads_series_values() {
    let xml = r#"<?xml version="1.0"?>
<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart">
<c:chart><c:plotArea><c:barChart>
  <c:ser>
    <c:cat><c:strLit>
      <c:pt idx="0"><c:v>A</c:v></c:pt>
      <c:pt idx="1"><c:v>B</c:v></c:pt>
    </c:strLit></c:cat>
    <c:val><c:numLit>
      <c:pt idx="0"><c:v>4.3</c:v></c:pt>
      <c:pt idx="1"><c:v>2.5</c:v></c:pt>
    </c:numLit></c:val>
  </c:ser>
  <c:ser>
    <c:val><c:numLit>
      <c:pt idx="0"><c:v>2.4</c:v></c:pt>
      <c:pt idx="1"><c:v>4.4</c:v></c:pt>
    </c:numLit></c:val>
  </c:ser>
</c:barChart></c:plotArea></c:chart></c:chartSpace>"#;
    let data = parse_chart(xml).expect("chart");
    // No c:title element: Word paints no title (docxide case29).
    assert_eq!(data.title, "");
    assert_eq!(data.cats, ["A", "B"]);
    assert_eq!(data.series.len(), 2);
    assert!((data.series[0][0] - 4.3).abs() < 0.01);
    assert!((data.series[1][1] - 4.4).abs() < 0.01);
    assert_eq!(data.names, ["Series 1", "Series 2"]);
    assert!(!data.legend);
    let a1 = theme_slot_color("accent1").expect("accent1");
    let a2 = theme_slot_color("accent2").expect("accent2");
    assert_eq!(data.colors.len(), 2);
    assert!(
        (data.colors[0][0] - a1[0]).abs() < 0.02,
        "missing schemeClr falls back to accent1; {:?}",
        data.colors[0]
    );
    assert!(
        (data.colors[1][0] - a2[0]).abs() < 0.02,
        "series 2 fallback accent2; {:?}",
        data.colors[1]
    );
}

#[test]
fn parse_bar_chart_reads_series_scheme_colors() {
    let xml = r#"<?xml version="1.0"?>
<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
<c:chart><c:plotArea><c:barChart>
  <c:ser>
    <c:spPr><a:solidFill><a:schemeClr val="accent1"/></a:solidFill></c:spPr>
    <c:val><c:numLit><c:pt idx="0"><c:v>1</c:v></c:pt></c:numLit></c:val>
  </c:ser>
  <c:ser>
    <c:spPr><a:solidFill><a:schemeClr val="accent2"/></a:solidFill></c:spPr>
    <c:val><c:numLit><c:pt idx="0"><c:v>1</c:v></c:pt></c:numLit></c:val>
  </c:ser>
  <c:ser>
    <c:spPr><a:solidFill><a:schemeClr val="accent3"/></a:solidFill></c:spPr>
    <c:val><c:numLit><c:pt idx="0"><c:v>1</c:v></c:pt></c:numLit></c:val>
  </c:ser>
</c:barChart></c:plotArea></c:chart></c:chartSpace>"#;
    let data = parse_chart(xml).expect("chart");
    let a3 = theme_slot_color("accent3").expect("accent3");
    assert_eq!(data.colors.len(), 3);
    assert!(
        (data.colors[2][0] - a3[0]).abs() < 0.02
            && (data.colors[2][1] - a3[1]).abs() < 0.02
            && (data.colors[2][2] - a3[2]).abs() < 0.02,
        "series 3 must be accent3, got {:?}",
        data.colors[2]
    );
}

#[test]
fn parse_chart_ignores_major_gridline_lum_after_mini_385() {
    // ChartData no longer stores grid color; emit stays hardcoded 0.88
    // (mini 385–388 ITT-neg). This XML must still parse series.
    let xml = r#"<?xml version="1.0"?>
<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
<c:chart><c:plotArea>
  <c:barChart>
    <c:ser>
      <c:val><c:numLit><c:pt idx="0"><c:v>1</c:v></c:pt></c:numLit></c:val>
    </c:ser>
  </c:barChart>
  <c:valAx>
    <c:majorGridlines>
      <c:spPr>
        <a:ln w="9525">
          <a:solidFill>
            <a:schemeClr val="tx1"><a:lumMod val="15%"/><a:lumOff val="85%"/></a:schemeClr>
          </a:solidFill>
        </a:ln>
      </c:spPr>
    </c:majorGridlines>
  </c:valAx>
</c:plotArea></c:chart></c:chartSpace>"#;
    let data = parse_chart(xml).expect("chart");
    assert_eq!(data.series.len(), 1);
}

#[test]
fn parse_chart_reads_series_names_and_bottom_legend() {
    let xml = r#"<?xml version="1.0"?>
<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart">
<c:chart>
  <c:plotArea><c:barChart>
    <c:ser>
      <c:tx><c:strRef><c:strCache><c:pt idx="0"><c:v>Series 1</c:v></c:pt></c:strCache></c:strRef></c:tx>
      <c:val><c:numLit><c:pt idx="0"><c:v>4.3</c:v></c:pt></c:numLit></c:val>
    </c:ser>
    <c:ser>
      <c:tx><c:strRef><c:strCache><c:pt idx="0"><c:v>Series 2</c:v></c:pt></c:strCache></c:strRef></c:tx>
      <c:val><c:numLit><c:pt idx="0"><c:v>2.4</c:v></c:pt></c:numLit></c:val>
    </c:ser>
  </c:barChart></c:plotArea>
  <c:legend><c:legendPos val="b"/></c:legend>
</c:chart></c:chartSpace>"#;
    let data = parse_chart(xml).expect("chart");
    assert_eq!(data.names, ["Series 1", "Series 2"]);
    assert!(data.legend);
}

#[test]
fn parse_chart_reads_explicit_title() {
    let xml = r#"<?xml version="1.0"?>
<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart">
<c:chart><c:title><c:tx><c:rich>
  <a:p xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
    <a:r><a:t>Sales</a:t></a:r>
  </a:p></c:rich></c:tx></c:title>
<c:plotArea><c:barChart>
  <c:ser><c:val><c:numLit>
    <c:pt idx="0"><c:v>1</c:v></c:pt>
  </c:numLit></c:val></c:ser>
</c:barChart></c:plotArea></c:chart></c:chartSpace>"#;
    let data = parse_chart(xml).expect("chart");
    assert_eq!(data.title, "Sales");
}

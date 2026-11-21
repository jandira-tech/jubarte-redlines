// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;

fn fixture(shape: &str) -> (Dom, NodeId, NodeId) {
    let mut dom = Dom::new();
    let doc=dom.parse_xdocument(&format!(r#"<w:document xmlns:w="{}" xmlns:v="urn:schemas-microsoft-com:vml"><w:body><w:p><w:r><w:pict>{shape}</w:pict></w:r></w:p></w:body></w:document>"#,W::URI));
    let root = dom.root(doc).unwrap();
    let image = descendants_local(&dom, root, "imagedata")[0];
    (dom, root, image)
}

#[test]
fn inline_owner_ancestry_walk_uses_own_extent_and_never_a_sibling_float_slot() {
    for (style, extent) in [
        ("", (60.0, 70.0)),
        ("width:30pt", (60.0, 70.0)),
        ("height:40pt", (60.0, 70.0)),
        ("width:30pt;height:40pt", (30.0, 40.0)),
    ] {
        let (dom, root, image) = fixture(&format!(
            "<v:shape style='{style}'><v:imagedata/></v:shape><v:shape style='position:absolute;margin-left:90pt;margin-top:80pt;width:60pt;height:70pt'/>"
        ));
        let source = dom.serialize_element(root);
        assert!(vml_owner_slot(&dom, image, root).is_none());
        assert_eq!(vml_owner_extent_pt(&dom, image, root), extent);
        assert_eq!(dom.serialize_element(root), source);
    }
}

#[test]
fn group_owner_coordinates_add_child_origin_to_the_correct_page_or_paragraph_frame() {
    for (relative, page_x, page_y, col_x, para_y) in [
        ("page", Some(20.0), Some(30.0), None, None),
        ("text", None, None, Some(20.0), Some(30.0)),
    ] {
        let (dom, root, image) = fixture(&format!(
            "<v:group style='position:absolute;margin-left:10pt;margin-top:20pt;width:100pt;height:50pt;mso-position-horizontal-relative:{relative};mso-position-vertical-relative:{relative}' coordorigin='100,50' coordsize='1000,500'><v:shape style='left:200;top:150;width:300;height:100'><v:imagedata/></v:shape></v:group>"
        ));
        let source = dom.serialize_element(root);
        assert_eq!(vml_owner_extent_pt(&dom, image, root), (30.0, 10.0));
        let Some(ImageSlot::Float {
            page_x: actual_x,
            page_y: actual_y,
            col_x: actual_col,
            para_y: actual_para,
            align,
            v_align,
            pct_x,
            pct_y,
            pct_w,
            pct_h,
            wrap_square,
            wrap_top_bottom,
            wrap_polygon,
            dist_l,
            dist_r,
            dist_t,
            dist_b,
            effect_b,
            col_in_column,
            v_off,
            ..
        }) = vml_owner_slot(&dom, image, root)
        else {
            panic!("authored group float")
        };
        assert_eq!(
            (actual_x, actual_y, actual_col, actual_para),
            (page_x, page_y, col_x, para_y)
        );
        assert!(matches!(align, Align::Left) && matches!(v_align, Align::Left));
        assert!(
            pct_x.is_none()
                && pct_y.is_none()
                && pct_w.is_none()
                && pct_h.is_none()
                && v_off.is_none()
        );
        assert!(!wrap_square && !wrap_top_bottom && !wrap_polygon);
        assert_eq!(
            (dist_l, dist_r, dist_t, dist_b, effect_b),
            (9.0, 9.0, 0.0, 0.0, 0.0)
        );
        assert_eq!(col_in_column, relative == "text");
        assert_eq!(dom.serialize_element(root), source);
    }
}

#[test]
fn a_flow_group_keeps_scaled_child_extent_and_declines_floating_owner_metadata() {
    let (dom, root, image) = fixture(
        "<v:group style='width:100pt;height:50pt' coordsize='1000,500'><v:shape style='left:200;top:100;width:300;height:100'><v:imagedata/></v:shape></v:group>",
    );
    let source = dom.serialize_element(root);
    assert_eq!(vml_owner_extent_pt(&dom, image, root), (30.0, 10.0));
    assert!(vml_owner_slot(&dom, image, root).is_none());
    assert_eq!(dom.serialize_element(root), source);
}

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;

fn close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 0.00002,
        "{actual} != {expected}"
    );
}

fn xml(content: &str) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&format!(
        r#"<w:root xmlns:w="{}" xmlns:v="urn:schemas-microsoft-com:vml">{content}</w:root>"#,
        W::URI
    ));
    let root = dom.root(doc).unwrap();
    (dom, root)
}

#[test]
fn fixed_table_zero_weight_columns_share_the_remaining_measure() {
    // A fixed grid may omit dxa widths entirely; its explicit percentage
    // still holds, while unspecified columns share the remaining room.
    assert_eq!(
        fixed_pct_widths(&[25.0, 0.0, 0.0], &[Some(0.25), None, None], 100.0, 1.0),
        [25.0, 37.5, 37.5]
    );
    assert_eq!(
        fixed_pct_widths(&[25.0, 20.0, 60.0], &[Some(0.25), None, None], 100.0, 1.0),
        [25.0, 18.75, 56.25]
    );
    // Percentages exceeding the measure must not starve unspecified
    // columns below their padding and minimum ink width.
    assert_eq!(
        fixed_pct_widths(
            &[100.0, 100.0, 0.0],
            &[Some(1.0), Some(1.0), None],
            100.0,
            10.0
        ),
        [80.0, 10.0, 10.0]
    );
}

#[test]
fn hsl_round_trip_preserves_primaries_secondary_colors_and_achromatic_samples() {
    // The named colors exercise every hue sector; red-purple crosses
    // the hue origin, and grey has no hue or saturation.
    for rgb in [
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 1.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [0.8, 0.1, 0.4],
        [0.0; 3],
        [0.5; 3],
        [1.0; 3],
    ] {
        let (h, s, l) = rgb_to_hsl(rgb[0], rgb[1], rgb[2]);
        assert!((0.0..=1.0).contains(&h));
        assert!((0.0..=1.0).contains(&s));
        assert!((0.0..=1.0).contains(&l));
        for (actual, expected) in hsl_to_rgb(h, s, l).into_iter().zip(rgb) {
            close(actual, expected);
        }
    }
    assert_eq!(rgb_to_hsl(0.5, 0.5, 0.5), (0.0, 0.0, 0.5));
    assert_eq!(hsl_to_rgb(-1.0, 1.0, 0.5), [1.0, 0.0, 0.0]);
    assert_eq!(hsl_to_rgb(2.0, 1.0, 0.5), [1.0, 0.0, 0.0]);
}

#[test]
fn color_luminance_and_shade_have_distinct_color_space_contracts() {
    let mut grey = [0.5; 3];
    apply_hsl_lum_mod(&mut grey, 0.5);
    assert_eq!(grey, [64.0 / 255.0; 3]);
    // Linear-light shading must not simply halve the sRGB sample.
    assert_eq!(srgb_shade(1.0, 0.5), 188.0 / 255.0);
    assert_eq!(srgb_shade(0.0, 0.5), 0.0);
    for value in [0.0, 0.02, 0.04045, 0.1, 0.5, 1.0] {
        close(linear_to_srgb(srgb_to_linear(value)), value);
    }
    for factor in [-1.0, 0.0, 3.0] {
        let mut color = [0.5; 3];
        apply_hsl_lum_mod(&mut color, factor);
        assert_eq!(color, [if factor > 1.0 { 1.0 } else { 0.0 }; 3]);
    }
}

#[test]
fn table_style_override_requires_word_uri_and_an_enabled_value() {
    assert!(!settings_override_table_style_size_xml(""));
    for name in [
        "overrideTableStyleFontSizeAndJustification",
        "anotherSetting",
    ] {
        for uri in [WORD_COMPAT_SETTING_URI, "urn:another-vendor"] {
            for value in ["1", "true", "on", "0", "false", "off", "", "TRUE"] {
                let source = format!(
                    r#"<w:settings xmlns:w="{}"><w:compat><w:compatSetting w:name="{name}" w:uri="{uri}" w:val="{value}"/></w:compat></w:settings>"#,
                    W::URI
                );
                let expected = name == "overrideTableStyleFontSizeAndJustification"
                    && uri == WORD_COMPAT_SETTING_URI
                    && matches!(value, "1" | "true" | "on");
                assert_eq!(settings_override_table_style_size_xml(&source), expected);
            }
        }
    }
}

#[test]
fn attached_template_replaces_only_default_paragraph_style_and_inserts_when_absent() {
    let nondefault =
        r#"<w:style w:type="paragraph" w:styleId="Heading"><w:rPr><w:b/></w:rPr></w:style>"#;
    let character = r#"<w:style w:type="character" w:default="1" w:styleId="DefaultChar"/>"#;
    for normal in [
        r#"<w:style w:type="paragraph" w:default="1" w:styleId="Normal"/>"#,
        r#"<w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:rPr><w:sz w:val="99"/></w:rPr></w:style>"#,
        "",
    ] {
        let source = format!(
            r#"<w:styles xmlns:w="{}">{nondefault}{character}{normal}</w:styles>"#,
            W::URI
        );
        let result = link_template_styles(&source);
        assert!(result.contains(nondefault) && result.contains(character));
        assert!(!result.contains("99"));
        assert_eq!(result.matches(TEMPLATE_NORMAL).count(), 1);
        let (dom, root) = xml(&result);
        let defaults: Vec<_> = dom
            .descendants(root, Some(&W::name("style")))
            .into_iter()
            .filter(|&n| {
                attr_any(&dom, n, "type") == Some("paragraph")
                    && attr_any(&dom, n, "default") == Some("1")
            })
            .collect();
        assert_eq!(defaults.len(), 1);
        assert_eq!(attr_any(&dom, defaults[0], "styleId"), Some("Normal"));
        assert_eq!(
            link_template_styles(&result),
            result,
            "linking twice is idempotent"
        );
    }
    for truncated in [
        "<w:styles><w:style ",
        "<w:styles><w:style w:type=\"paragraph\">",
        "<w:styles>",
    ] {
        assert_eq!(link_template_styles(truncated), truncated);
    }
}

#[test]
fn vml_images_take_their_own_geometry_without_borrowing_sibling_anchors() {
    for owner in [
        "<v:shape><v:imagedata/></v:shape>",
        "<v:shape style=\"width:30pt;height:40pt\"><v:imagedata/></v:shape>",
        "<v:imagedata/>",
    ] {
        let (dom, root) = xml(&format!(
            r#"<w:pict>{owner}<v:shape style="position:absolute;margin-left:90pt;margin-top:80pt;width:60pt;height:70pt"/></w:pict>"#
        ));
        let pict = first_named(&dom, root, "pict").unwrap();
        let image = descendants_local(&dom, pict, "imagedata")
            .into_iter()
            .next()
            .expect("VML fixture node");
        assert!(vml_owner_slot(&dom, image, pict).is_none());
        let expected = if owner.contains("30pt") {
            (30.0, 40.0)
        } else {
            (60.0, 70.0)
        };
        assert_eq!(vml_owner_extent_pt(&dom, image, pict), expected);
    }
}

#[test]
fn vml_cropping_rejects_nonfinite_samples_and_preserves_negative_blank_strips() {
    let (dom, root) =
        xml(r#"<v:imagedata cropleft="32768f" croptop="NaN" cropright="2" cropbottom="-16384f"/>"#);
    let image = descendants_local(&dom, root, "imagedata")
        .into_iter()
        .next()
        .expect("VML fixture node");
    assert_eq!(vml_crop(&dom, image), Some([0.5, 0.0, 1.0, -0.25]));
    for attrs in [
        "",
        "cropleft=\"bad\"",
        "croptop=\"inf\"",
        "cropright=\"0.0001\"",
    ] {
        let (dom, root) = xml(&format!("<v:imagedata {attrs}/>"));
        let image = descendants_local(&dom, root, "imagedata")
            .into_iter()
            .next()
            .expect("VML fixture node");
        assert_eq!(vml_crop(&dom, image), None);
    }
    // These are the three measured Word sizes from the production contract.
    for (input, output) in [(30.0, 30.2), (36.0, 36.25), (40.0, 39.8)] {
        close(vml_pixel_snap(input), output);
    }
    for size in [0.0, -1.0, f32::INFINITY] {
        assert_eq!(vml_pixel_snap(size), size);
    }
    assert!(vml_pixel_snap(f32::NAN).is_nan());
}

#[test]
fn preset_polygons_use_short_dimension_adjustments_and_pdf_upward_coordinates() {
    assert_eq!(
        diamond_points(10.0, 20.0, 80.0, 40.0),
        [(50.0, 60.0), (90.0, 40.0), (50.0, 20.0), (10.0, 40.0)]
    );
    assert_eq!(
        trapezoid_points(10.0, 20.0, 80.0, 40.0),
        [(10.0, 20.0), (20.0, 60.0), (80.0, 60.0), (90.0, 20.0)]
    );
    assert_eq!(
        home_plate_points(10.0, 20.0, 80.0, 40.0),
        [
            (10.0, 60.0),
            (70.0, 60.0),
            (90.0, 40.0),
            (70.0, 20.0),
            (10.0, 20.0)
        ]
    );
    // Tall shapes must use width as the short dimension, not height.
    assert_eq!(trapezoid_points(10.0, 20.0, 40.0, 80.0)[1], (20.0, 100.0));
    assert_eq!(home_plate_points(10.0, 20.0, 40.0, 80.0)[1], (30.0, 100.0));
}
#[test]
fn vml_lines_keep_anchor_offsets_and_collapse_only_negative_extents() {
    for (from, to, dimensions, endpoints) in [
        (
            "10pt,20pt",
            "40pt,50pt",
            (30.0, 30.0),
            ([0.0, 0.0], [1.0, 1.0]),
        ),
        (
            "10pt,20pt",
            "5pt,50pt",
            (0.0, 30.0),
            ([0.0, 0.0], [0.0, 1.0]),
        ),
        (
            "10pt,20pt",
            "40pt,5pt",
            (30.0, 0.0),
            ([0.0, 0.0], [1.0, 0.0]),
        ),
        ("12,24", "52,64", (30.0, 30.0), ([0.0, 0.0], [1.0, 1.0])),
    ] {
        let (dom, root) = xml(&format!(
            r##"<v:line from="{from}" to="{to}" style="position:absolute;margin-left:2pt;margin-top:3pt" strokecolor="#FF0000" strokeweight="2pt"/>"##
        ));
        let line = descendants_local(&dom, root, "line")
            .into_iter()
            .next()
            .expect("VML fixture node");
        let image = vml_line_image(&dom, line, root).unwrap();
        assert_eq!((image.w, image.h), dimensions);
        let ImageKind::VmlLine {
            from: a,
            to: b,
            color,
            width,
        } = image.kind
        else {
            panic!("line payload")
        };
        assert_eq!((a, b), endpoints);
        assert_eq!(color, [1.0, 0.0, 0.0]);
        assert_eq!(width, 2.0);
        let ImageSlot::Float {
            page_x,
            page_y,
            col_x,
            para_y,
            ..
        } = image.slot
        else {
            panic!("line anchor")
        };
        assert!(page_x.is_none() && page_y.is_none());
        let (x, y) = if from.contains("pt") {
            (12.0, 23.0)
        } else {
            (11.0, 21.0)
        };
        assert_eq!((col_x, para_y), (Some(x), Some(y)));
    }
    for attrs in [
        "",
        r#"from="0,0" stroked="false""#,
        r#"from="20,20" to="10,10" style="position:absolute""#,
        r#"from="bad,10" to="20,20" style="position:absolute""#,
        r#"from="0,0" to="10,10""#,
    ] {
        let (dom, root) = xml(&format!("<v:line {attrs}/>"));
        let line = descendants_local(&dom, root, "line")
            .into_iter()
            .next()
            .expect("VML fixture node");
        assert!(vml_line_image(&dom, line, root).is_none());
    }
}

#[test]
fn grouped_vml_lines_scale_from_group_origin_and_require_a_positioned_group() {
    let (dom, root) = xml(
        r#"<v:group coordsize="200,100" coordorigin="20,10" style="position:absolute;width:80pt;height:40pt;margin-left:5pt;margin-top:7pt"><v:line from="20,10" to="220,110"/></v:group>"#,
    );
    let line = descendants_local(&dom, root, "line")
        .into_iter()
        .next()
        .expect("VML fixture node");
    let image = vml_line_image(&dom, line, root).unwrap();
    assert_eq!((image.w, image.h), (80.0, 40.0));
    let ImageKind::VmlLine {
        from,
        to,
        color,
        width,
    } = image.kind
    else {
        panic!("line payload")
    };
    assert_eq!((from, to), ([0.0, 0.0], [1.0, 1.0]));
    assert_eq!((color, width), ([0.0; 3], 0.75));
    for style in [
        "",
        "position:absolute;height:40pt",
        "width:80pt;height:40pt",
    ] {
        let (dom, root) = xml(&format!(
            r#"<v:group style="{style}"><v:line from="0,0" to="1000,1000"/></v:group>"#
        ));
        let line = descendants_local(&dom, root, "line")
            .into_iter()
            .next()
            .expect("VML fixture node");
        assert!(vml_line_image(&dom, line, root).is_none());
    }
}

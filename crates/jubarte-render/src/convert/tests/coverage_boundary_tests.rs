// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;

// These probes use only the existing in-memory DOM and production value types.
fn xml_root(tag: &str, contents: &str) -> (Dom, NodeId) {
    let xml = format!(
        r#"<w:{tag} xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml">{contents}</w:{tag}>"#
    );
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).expect("probe root");
    (dom, root)
}

fn close(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 0.0001, "{actual} != {expected}");
}

fn run_props(contents: &str) -> RunStyle {
    let (dom, root) = xml_root("rPr", contents);
    let mut style = Defaults::word().run;
    apply_rpr(&dom, root, &mut style, &ThemeFonts::default());
    style
}

#[test]
fn length_units_and_invalid_operands_have_explicit_results() {
    for (input, expected) in [
        ("1440", 72.0),
        (" -120 ", -6.0),
        (" 9.5 pt ", 9.5),
        ("1in", 72.0),
        ("2.54cm", 72.0),
        ("25.4mm", 72.0),
        ("0pt", 0.0),
        ("-1in", -72.0),
        // A finite exponent is still a number. Only a non-finite result is absent.
        ("1e2", 5.0),
    ] {
        close(parse_len(input).expect(input), expected);
    }
    for input in [
        "",
        "pt",
        "in",
        "cm",
        "mm",
        "xpt",
        "xin",
        "xcm",
        "xmm",
        "12px",
        "2PT",
        "NaN",
        "nan",
        "inf",
        "+inf",
        "-inf",
        "infinity",
        "-infinity",
        "1e40",
        "1e40pt",
        "1e38in",
        "NaNin",
    ] {
        assert_eq!(parse_len(input), None, "{input}");
    }
}

#[test]
fn malformed_run_numeric_properties_preserve_inherited_values() {
    for property in ["sz", "szCs", "spacing", "w", "position", "kern"] {
        for attrs in ["", r#"w:val="bad""#] {
            let (dom, root) = xml_root("rPr", &format!("<w:{property} {attrs}/>"));
            let mut style = Defaults::word().run;
            style.size_cs = Some(13.0);
            style.track = 2.0;
            style.scale = 1.25;
            style.offset = 3.0;
            style.kern_half = 24;
            apply_rpr(&dom, root, &mut style, &ThemeFonts::default());
            assert_eq!(
                (style.size, style.size_cs),
                (11.0, Some(13.0)),
                "{property} {attrs}"
            );
            assert_eq!(
                (style.track, style.scale, style.offset, style.kern_half),
                (2.0, 1.25, 3.0, 24)
            );
        }
    }
    for input in ["0", "-5", "0%", "-20%"] {
        assert_eq!(run_props(&format!(r#"<w:w w:val="{input}"/>"#)).scale, 1.0);
    }
}

#[test]
fn authored_run_metrics_apply_units_and_percent_scale() {
    let style = run_props(
        r#"<w:sz w:val="30"/><w:szCs w:val="28"/><w:spacing w:val="40"/><w:w w:val=" 150% "/><w:position w:val="-6"/><w:kern w:val="24"/>"#,
    );
    assert_eq!((style.size, style.size_cs), (15.0, Some(14.0)));
    assert_eq!(
        (style.track, style.scale, style.offset, style.kern_half),
        (2.0, 1.5, -3.0, 24)
    );
    assert!(!style.kerns_at(11.99));
    assert!(style.kerns_at(12.0));
    assert!(style.kerns_at(13.0));
    assert!(!Defaults::word().run.kerns_at(100.0));
}

#[test]
fn run_boolean_properties_distinguish_false_from_implicit_true() {
    for (attrs, on) in [
        ("", true),
        (r#"w:val="1""#, true),
        (r#"w:val="true""#, true),
        (r#"w:val="0""#, false),
        (r#"w:val="false""#, false),
        (r#"w:val="off""#, false),
    ] {
        let contents = [
            "b",
            "i",
            "rtl",
            "vanish",
            "caps",
            "smallCaps",
            "strike",
            "dstrike",
        ]
        .map(|tag| format!("<w:{tag} {attrs}/>"))
        .join("");
        let style = run_props(&contents);
        assert_eq!(
            (style.bold, style.italic, style.rtl, style.hidden),
            (on, on, on, on)
        );
        assert_eq!(
            (
                style.caps,
                style.small_caps,
                style.strike,
                style.strike_double
            ),
            (on, on, on, on)
        );
        assert!(style.bold_set && style.italic_set);
    }
}

#[test]
fn underline_kinds_have_independent_decoration_flags() {
    for (val, underline, double, wave) in [
        (None, false, false, false),
        (Some("none"), false, false, false),
        (Some("single"), true, false, false),
        (Some("double"), true, true, false),
        (Some("thick"), true, true, false),
        (Some("wave"), true, false, true),
        (Some("wavy"), true, false, true),
    ] {
        let attrs = val.map_or(String::new(), |v| format!(r#"w:val="{v}""#));
        let style = run_props(&format!("<w:u {attrs}/>"));
        assert_eq!(
            (
                style.underline,
                style.underline_double,
                style.underline_wave
            ),
            (underline, double, wave),
            "{val:?}"
        );
    }
}

#[test]
fn text_effect_suppression_requires_the_documented_combinations() {
    for (contents, skipped) in [
        ("<w14:reflection/>", true),
        ("<w14:shadow/>", false),
        ("<w14:textOutline/>", false),
        ("<w14:shadow/><w14:textOutline/>", true),
        (
            "<w14:shadow/><w14:textOutline><w14:noFill/></w14:textOutline>",
            false,
        ),
        (r#"<w:color w:val="FF0000"/><w14:textOutline/>"#, true),
        (
            r#"<w:sz w:val="24"/><w:color w:val="FF0000"/><w14:textOutline/>"#,
            false,
        ),
    ] {
        assert_eq!(run_props(contents).effect_skip, skipped, "{contents}");
    }
}

#[test]
fn theme_run_color_obeys_explicit_color_shade_tint_and_auto_precedence() {
    let theme = ThemeFonts {
        colors: HashMap::from([("accent1".into(), [1.0, 0.0, 0.0])]),
        ..ThemeFonts::default()
    };
    for (attrs, expected, auto) in [
        (
            r#"w:val="00FF00" w:themeColor="accent1""#,
            [0.0, 1.0, 0.0],
            false,
        ),
        (
            r#"w:val="auto" w:themeColor="accent1" w:themeShade="00""#,
            [0.0; 3],
            false,
        ),
        (
            r#"w:themeColor="accent1" w:themeTint="00""#,
            [1.0; 3],
            false,
        ),
        (
            r#"w:themeColor="accent1" w:themeShade="FF" w:themeTint="FF""#,
            [1.0, 0.0, 0.0],
            false,
        ),
        (
            r#"w:themeColor="accent1" w:themeShade="bad" w:themeTint="bad""#,
            [1.0, 0.0, 0.0],
            false,
        ),
        (r#"w:val="auto" w:themeColor="unknown""#, [0.0; 3], true),
    ] {
        let (dom, root) = xml_root("rPr", &format!("<w:color {attrs}/>"));
        let mut style = Defaults::word().run;
        style.color = [0.2, 0.3, 0.4];
        style.color_auto = false;
        apply_rpr(&dom, root, &mut style, &theme);
        assert_eq!((style.color, style.color_auto), (expected, auto), "{attrs}");
    }
    for attrs in ["", r#"w:val="invalid""#, r#"w:themeColor="unknown""#] {
        let (dom, root) = xml_root("rPr", &format!("<w:color {attrs}/>"));
        let mut style = Defaults::word().run;
        style.color = [0.2, 0.3, 0.4];
        apply_rpr(&dom, root, &mut style, &theme);
        assert_eq!(style.color, [0.2, 0.3, 0.4]);
    }
}

#[test]
fn w14_fill_ignores_unpaintable_slots_and_explicit_run_color_wins() {
    for (contents, expected_auto) in [
        ("<w14:textFill/>", true),
        (r#"<w14:textFill><a:schemeClr/></w14:textFill>"#, true),
        (
            r#"<w14:textFill><a:schemeClr val="unknown"/></w14:textFill>"#,
            true,
        ),
        (
            r#"<w14:textFill><a:noFill/><a:schemeClr val="accent1"/></w14:textFill>"#,
            true,
        ),
        (
            r#"<w14:textFill><a:schemeClr val="accent1"/></w14:textFill>"#,
            false,
        ),
    ] {
        let style = run_props(contents);
        assert_eq!(style.color_auto, expected_auto);
        assert_eq!(
            style.color,
            if expected_auto {
                [0.0; 3]
            } else {
                [79.0 / 255.0, 129.0 / 255.0, 189.0 / 255.0]
            }
        );
    }
    let style = run_props(
        r#"<w:color w:val="FF0000"/><w14:textFill><a:schemeClr val="accent1"/></w14:textFill>"#,
    );
    assert_eq!(style.color, [1.0, 0.0, 0.0]);
}

#[test]
fn every_highlight_color_and_unknown_color_has_an_expected_paint() {
    for (name, rgb) in [
        ("yellow", [1.0, 1.0, 0.0]),
        ("green", [0.0, 1.0, 0.0]),
        ("cyan", [0.0, 1.0, 1.0]),
        ("magenta", [1.0, 0.0, 1.0]),
        ("blue", [0.0, 0.0, 1.0]),
        ("red", [1.0, 0.0, 0.0]),
        ("darkBlue", [0.0, 0.0, 0.5]),
        ("darkCyan", [0.0, 0.5, 0.5]),
        ("darkGreen", [0.0, 0.5, 0.0]),
        ("darkMagenta", [0.5, 0.0, 0.5]),
        ("darkRed", [0.5, 0.0, 0.0]),
        ("darkYellow", [0.5, 0.5, 0.0]),
        ("darkGray", [0.5, 0.5, 0.5]),
        ("lightGray", [0.75; 3]),
        ("black", [0.0; 3]),
    ] {
        let style = run_props(&format!(
            r#"<w:highlight w:val="{name}"/><w:shd w:fill="112233"/>"#
        ));
        assert_eq!(style.highlight, Some(rgb), "{name}");
        assert!(style.highlight_marker);
    }
    for name in ["none", "unknown"] {
        let style = run_props(&format!(
            r#"<w:highlight w:val="{name}"/><w:shd w:fill="FF0000"/>"#
        ));
        assert_eq!(style.highlight, Some([1.0, 0.0, 0.0]));
        assert!(!style.highlight_marker);
    }
}

#[test]
fn paragraph_auto_spacing_merges_fallback_values_and_can_be_disabled() {
    for sum in [false, true] {
        let mut style = Defaults::word().para;
        style.sum_spacing = sum;
        let (dom, root) = xml_root(
            "pPr",
            r#"<w:spacing w:before="120" w:after="160" w:beforeAutospacing="1" w:afterAutospacing="true"/>"#,
        );
        apply_ppr(&dom, root, &mut style);
        assert_eq!((style.before_spec, style.after_spec), (6.0, 8.0));
        assert_eq!((style.before_auto, style.after_auto), (!sum, !sum));
        assert_eq!(
            (style.before, style.after),
            if sum { (6.0, 8.0) } else { (14.0, 14.0) }
        );
        let (dom, root) = xml_root("pPr", r#"<w:spacing w:before="200" w:after="220"/>"#);
        apply_ppr(&dom, root, &mut style);
        assert_eq!((style.before_spec, style.after_spec), (10.0, 11.0));
        assert_eq!(
            (style.before, style.after),
            if sum { (10.0, 11.0) } else { (14.0, 14.0) }
        );
        let (dom, root) = xml_root(
            "pPr",
            r#"<w:spacing w:beforeAutospacing="0" w:afterAutospacing="off"/>"#,
        );
        apply_ppr(&dom, root, &mut style);
        assert_eq!((style.before, style.after), (10.0, 11.0));
        assert!(!style.before_auto && !style.after_auto);
    }
}

#[test]
fn malformed_paragraph_attributes_leave_inherited_metrics_intact() {
    for tag in ["jc", "spacing", "ind", "outlineLvl"] {
        for attrs in [
            "",
            r#"w:val="bad" w:before="bad" w:after="bad" w:line="bad" w:left="bad" w:right="bad" w:firstLine="bad" w:hanging="bad""#,
        ] {
            let (dom, root) = xml_root("pPr", &format!("<w:{tag} {attrs}/>"));
            let mut style = Defaults::word().para;
            style.line_exact = Some(17.0);
            style.indent_left = 12.0;
            style.indent_right = 8.0;
            style.indent_first = -3.0;
            style.outline_lvl = Some(2);
            apply_ppr(&dom, root, &mut style);
            assert_eq!(
                (style.before, style.after, style.line_exact),
                (0.0, 10.0, Some(17.0))
            );
            assert_eq!(
                (style.indent_left, style.indent_right, style.indent_first),
                (12.0, 8.0, -3.0)
            );
            assert_eq!(style.outline_lvl, Some(2));
        }
    }
}

#[test]
fn paragraph_boolean_switches_and_alignment_aliases_are_resolved() {
    for (value, on) in [
        ("0", false),
        ("false", false),
        ("off", false),
        ("1", true),
        ("on", true),
    ] {
        let content = [
            "bidi",
            "widowControl",
            "contextualSpacing",
            "autoSpaceDE",
            "autoSpaceDN",
            "pageBreakBefore",
            "snapToGrid",
            "keepNext",
            "keepLines",
        ]
        .map(|tag| format!(r#"<w:{tag} w:val="{value}"/>"#))
        .join("");
        let (dom, root) = xml_root("pPr", &content);
        let mut style = Defaults::word().para;
        apply_ppr(&dom, root, &mut style);
        assert_eq!(
            (style.bidi, style.widow_control, style.contextual),
            (on, on, on)
        );
        assert_eq!(
            (
                style.page_break_before,
                style.snap_to_grid,
                style.keep_next,
                style.keep_lines
            ),
            (on, on, on, on)
        );
        assert_eq!(
            (style.auto_space_de_off, style.auto_space_dn_off),
            (!on, !on)
        );
    }
    for (value, expected) in [
        ("center", Align::Center),
        ("right", Align::Right),
        ("end", Align::Right),
        ("both", Align::Justify),
        ("distribute", Align::Justify),
        ("start", Align::Left),
        ("unknown", Align::Left),
    ] {
        let (dom, root) = xml_root("pPr", &format!(r#"<w:jc w:val="{value}"/>"#));
        let mut style = Defaults::word().para;
        apply_ppr(&dom, root, &mut style);
        assert!(style.align == expected, "{value}");
    }
}

#[test]
fn paragraph_line_rules_replace_each_other_and_unit_auto_has_a_floor() {
    for (attrs, exact, minimum, multiplier) in [
        (r#"w:line="400" w:lineRule="exact""#, Some(20.0), None, 1.15),
        (
            r#"w:line="400" w:lineRule="atLeast""#,
            None,
            Some(20.0),
            1.0,
        ),
        (r#"w:line="480" w:lineRule="auto""#, None, None, 2.0),
        (r#"w:line="22pt""#, None, None, 2.0),
        (r#"w:line="1pt" w:lineRule="unknown""#, None, None, 0.8),
    ] {
        let (dom, root) = xml_root("pPr", &format!("<w:spacing {attrs}/>"));
        let mut style = Defaults::word().para;
        style.line_exact = Some(9.0);
        style.line_at_least = Some(10.0);
        apply_ppr(&dom, root, &mut style);
        assert_eq!((style.line_exact, style.line_at_least), (exact, minimum));
        close(style.line_mult, multiplier);
    }
}

#[test]
fn strict_indent_aliases_and_hanging_override_first_line() {
    for (attrs, expected) in [
        (
            r#"w:start="240" w:end="160" w:firstLine="60""#,
            (12.0, 8.0, 3.0),
        ),
        (
            r#"w:left="200" w:start="240" w:right="100" w:end="160" w:firstLine="60" w:hanging="80""#,
            (10.0, 5.0, -4.0),
        ),
    ] {
        let (dom, root) = xml_root("pPr", &format!("<w:ind {attrs}/>"));
        let mut style = Defaults::word().para;
        apply_ppr(&dom, root, &mut style);
        assert_eq!(
            (style.indent_left, style.indent_right, style.indent_first),
            expected
        );
    }
}

#[test]
fn old_tracked_properties_never_supply_live_formatting() {
    for change in [
        "pPrChange",
        "rPrChange",
        "tblPrChange",
        "trPrChange",
        "tcPrChange",
        "sectPrChange",
        "numPrChange",
    ] {
        let (dom, root) = xml_root(
            "pPr",
            &format!(
                r#"<w:{change}><w:pPr><w:ind w:left="1440"/><w:sectPr/></w:pPr></w:{change}><w:spacing w:after="120"/>"#
            ),
        );
        assert!(first_named(&dom, root, "ind").is_none(), "{change}");
        assert!(live_sect_prs(&dom, root).is_empty(), "{change}");
        let mut style = Defaults::word().para;
        apply_ppr(&dom, root, &mut style);
        assert_eq!((style.indent_left, style.after), (0.0, 6.0));
    }
    let (dom, root) = xml_root(
        "body",
        "<w:p><w:pPr><w:sectPr><w:sectPrChange><w:sectPr/></w:sectPrChange></w:sectPr></w:pPr></w:p><w:sectPr/>",
    );
    assert_eq!(live_sect_prs(&dom, root).len(), 2);
}

#[test]
fn tab_xml_preserves_kind_leader_and_numbering_semantics() {
    for (val, align) in [
        ("left", TabAlign::Left),
        ("start", TabAlign::Left),
        ("num", TabAlign::Left),
        ("right", TabAlign::Right),
        ("end", TabAlign::Right),
        ("center", TabAlign::Center),
        ("decimal", TabAlign::Decimal),
        ("bar", TabAlign::Bar),
        ("unknown", TabAlign::Left),
    ] {
        for (leader, expected) in [
            ("dot", TabLeader::Dot),
            ("middleDot", TabLeader::Dot),
            ("hyphen", TabLeader::Hyphen),
            ("underscore", TabLeader::Underscore),
            ("heavy", TabLeader::Heavy),
            ("unknown", TabLeader::None),
        ] {
            let (dom, root) = xml_root(
                "pPr",
                &format!(
                    r#"<w:tabs><w:tab w:val="{val}" w:pos="720" w:leader="{leader}"/><w:tab w:val="clear" w:pos="1440"/><w:tab/><w:tab w:pos="bad"/></w:tabs>"#
                ),
            );
            let stops = parse_tab_stops(&dom, root);
            assert_eq!(stops.len(), 1);
            assert_eq!(stops[0].pos, 36.0);
            assert!(stops[0].align == align && stops[0].leader == expected);
            assert_eq!(stops[0].numbering, val == "num");
        }
    }
}

#[test]
fn tab_clear_replace_merge_and_destination_boundaries() {
    let stop = |pos, align| TabStop {
        pos,
        align,
        leader: TabLeader::None,
        numbering: false,
    };
    let mut stops = vec![stop(36.0, TabAlign::Left), stop(72.0, TabAlign::Right)];
    let (dom, root) = xml_root(
        "pPr",
        r#"<w:tabs><w:tab w:val="clear" w:pos="720"/><w:tab w:val="clear" w:pos="bad"/><w:tab w:val="center" w:pos="1440"/></w:tabs>"#,
    );
    apply_tab_stops(&mut stops, &dom, root);
    assert_eq!(stops.len(), 1);
    assert_eq!(stops[0].pos, 72.0);
    assert!(stops[0].align == TabAlign::Center);
    merge_tab_stops(
        &mut stops,
        &[stop(72.4, TabAlign::Left), stop(108.0, TabAlign::Right)],
    );
    assert_eq!(
        stops.iter().map(|s| s.pos).collect::<Vec<_>>(),
        vec![72.0, 108.0]
    );
    let (dom, root) = xml_root("pPr", "");
    apply_tab_stops(&mut stops, &dom, root);
    assert_eq!(stops.len(), 2);
    assert!(parse_tab_stops(&dom, root).is_empty());
    for (align, expected) in [
        (TabAlign::Left, 100.0),
        (TabAlign::Right, 70.0),
        (TabAlign::Center, 85.0),
        (TabAlign::Decimal, 80.0),
    ] {
        let tabs = [stop(50.0, TabAlign::Bar), stop(100.0, align)];
        assert_eq!(tab_dest(10.0, 0.0, &tabs, 36.0, 30.0, 20.0), expected);
        if align != TabAlign::Left {
            assert_eq!(tab_dest(90.0, 0.0, &tabs, 36.0, 100.0, 100.0), 90.0);
        }
    }
    assert_eq!(next_tab_x(36.0, 0.0, &[], 36.0), 72.0);
    assert_eq!(next_tab_x(-10.0, 5.0, &[], 0.5), 41.0);
    assert_eq!(next_tab_x(35.99, 0.0, &[], 36.0), 72.0);
}

#[test]
fn section_gutter_margins_and_negative_exact_margins_are_resolved() {
    for (rtl, top, expected) in [
        ("", false, (18.0, 10.0, 20.0)),
        ("<w:rtlGutter/>", false, (10.0, 18.0, 20.0)),
        (r#"<w:rtlGutter w:val="0"/>"#, false, (18.0, 10.0, 20.0)),
        (r#"<w:rtlGutter w:val="false"/>"#, false, (18.0, 10.0, 20.0)),
        ("<w:rtlGutter/>", true, (10.0, 10.0, 28.0)),
    ] {
        let (dom, root) = xml_root(
            "sectPr",
            &format!(
                r#"<w:pgSz w:w="8000" w:h="12000" w:orient="landscape"/><w:pgMar w:left="200" w:right="200" w:top="-400" w:bottom="-600" w:header="100" w:footer="120" w:gutter="160"/>{rtl}"#
            ),
        );
        let mut fallback = Defaults::word().page;
        fallback.gutter_at_top = top;
        let page = apply_sect_pr(&dom, root, &fallback);
        assert_eq!((page.width, page.height), (400.0, 600.0));
        assert_eq!((page.margin_l, page.margin_r, page.margin_t), expected);
        assert_eq!((page.margin_b, page.header, page.footer), (30.0, 5.0, 6.0));
        assert!(page.top_exact && page.bottom_exact);
    }
    for gutter in ["0", "-100", "bad"] {
        let (dom, root) = xml_root("sectPr", &format!(r#"<w:pgMar w:gutter="{gutter}"/>"#));
        let page = apply_sect_pr(&dom, root, &Defaults::word().page);
        assert_eq!(
            (page.margin_l, page.margin_r, page.margin_t),
            (72.0, 72.0, 72.0)
        );
    }
}

#[test]
fn vertical_sections_restore_physical_geometry_before_inheriting() {
    let mut fallback = Defaults::word().page;
    (
        fallback.margin_l,
        fallback.margin_r,
        fallback.margin_t,
        fallback.margin_b,
    ) = (10.0, 20.0, 30.0, 40.0);
    turn_page(&mut fallback, true);
    assert_eq!((fallback.width, fallback.height), (792.0, 612.0));
    assert_eq!(
        (
            fallback.margin_l,
            fallback.margin_r,
            fallback.margin_t,
            fallback.margin_b
        ),
        (30.0, 40.0, 20.0, 10.0)
    );
    for direction in ["tbRl", "tbRlV", "lrTb", "unknown"] {
        let (dom, root) = xml_root(
            "sectPr",
            &format!(r#"<w:textDirection w:val="{direction}"/>"#),
        );
        let page = apply_sect_pr(&dom, root, &fallback);
        let vertical = matches!(direction, "tbRl" | "tbRlV");
        assert_eq!(page.vertical, vertical);
        assert_eq!(
            (page.width, page.height),
            if vertical {
                (792.0, 612.0)
            } else {
                (612.0, 792.0)
            }
        );
        assert_eq!(
            (page.margin_l, page.margin_r, page.margin_t, page.margin_b),
            if vertical {
                (30.0, 40.0, 20.0, 10.0)
            } else {
                (10.0, 20.0, 30.0, 40.0)
            }
        );
    }
}

#[test]
fn custom_columns_are_capped_and_invalid_lengths_use_fallbacks() {
    for off in ["0", "false", "off"] {
        let (dom, root) = xml_root(
            "sectPr",
            &format!(
                r#"<w:cols w:num="0" w:space="200" w:equalWidth="{off}"><w:col w:w="2000" w:space="100"/><w:col w:w="bad"/><w:col w:w="3000" w:space="bad"/><w:col/><w:col w:w="9000"/></w:cols>"#
            ),
        );
        let page = apply_sect_pr(&dom, root, &Defaults::word().page);
        assert_eq!(
            (page.col_count, page.col_custom, page.col_space),
            (4, 4, 10.0)
        );
        assert_eq!(page.col_w, [100.0, 0.0, 150.0, 0.0]);
        assert_eq!(page.col_gap, [5.0, 10.0, 10.0, 10.0]);
    }
    for attrs in [
        r#"w:num="bad" w:space="bad""#,
        r#"w:num="0" w:equalWidth="off""#,
        r#"w:equalWidth="false""#,
    ] {
        let (dom, root) = xml_root("sectPr", &format!("<w:cols {attrs}/>"));
        let page = apply_sect_pr(&dom, root, &Defaults::word().page);
        assert_eq!(
            (page.col_count, page.col_custom, page.col_space),
            (1, 0, 36.0)
        );
    }
}

#[test]
fn section_line_numbers_grid_and_alignment_have_boundary_results() {
    for (restart, code) in [
        ("newPage", 0),
        ("newSection", 1),
        ("continuous", 2),
        ("unknown", 0),
    ] {
        let (dom, root) = xml_root(
            "sectPr",
            &format!(
                r#"<w:lnNumType w:countBy="0" w:start="0" w:distance="40" w:restart="{restart}"/>"#
            ),
        );
        let page = apply_sect_pr(&dom, root, &Defaults::word().page);
        assert_eq!(
            (
                page.ln_count_by,
                page.ln_start,
                page.ln_distance,
                page.ln_restart
            ),
            (1, 0, 2.0, code)
        );
    }
    for (ty, pitch, spacing) in [
        ("default", 0.0, 0.0),
        ("lines", 18.0, 0.0),
        ("snapToChars", 0.0, -1.0),
        ("linesAndChars", 18.0, -1.0),
        ("unknown", 0.0, 0.0),
    ] {
        let (dom, root) = xml_root(
            "sectPr",
            &format!(r#"<w:docGrid w:type="{ty}" w:linePitch="360" w:charSpace="-4096"/>"#),
        );
        let page = apply_sect_pr(&dom, root, &Defaults::word().page);
        assert_eq!((page.grid_pitch, page.grid_char), (pitch, spacing));
    }
    for (value, on) in [
        ("center", true),
        ("both", true),
        ("top", false),
        ("bottom", false),
    ] {
        let (dom, root) = xml_root("sectPr", &format!(r#"<w:vAlign w:val="{value}"/>"#));
        assert_eq!(
            apply_sect_pr(&dom, root, &Defaults::word().page).valign_center,
            on
        );
    }
}

#[test]
fn section_bad_numeric_values_preserve_sizes_but_reset_section_local_features() {
    let (dom, root) = xml_root(
        "sectPr",
        r#"<w:pgSz w:w="bad" w:h="bad"/><w:pgMar w:left="bad" w:right="bad" w:top="bad" w:bottom="bad" w:header="bad" w:footer="bad"/><w:pgNumType w:start="bad" w:chapStyle="0"/><w:lnNumType w:countBy="bad" w:start="bad" w:distance="bad"/><w:docGrid w:type="linesAndChars" w:linePitch="bad" w:charSpace="bad"/>"#,
    );
    let mut fallback = Defaults::word().page;
    fallback.grid_pitch = 18.0;
    fallback.grid_char = 2.0;
    fallback.page_num_start = Some(3);
    fallback.chap_style = Some(2);
    let page = apply_sect_pr(&dom, root, &fallback);
    assert_eq!(
        (
            page.width,
            page.height,
            page.margin_l,
            page.margin_r,
            page.margin_t,
            page.margin_b,
            page.header,
            page.footer
        ),
        (612.0, 792.0, 72.0, 72.0, 72.0, 72.0, 36.0, 36.0)
    );
    assert_eq!((page.page_num_start, page.chap_style), (Some(3), Some(2)));
    assert_eq!(
        (page.ln_count_by, page.ln_start, page.ln_distance),
        (1, 1, 0.0)
    );
    assert_eq!((page.grid_pitch, page.grid_char), (0.0, 0.0));
}

#[test]
fn section_chapter_separator_and_zero_page_start_are_preserved() {
    for (name, separator) in [
        ("hyphen", "-"),
        ("period", "."),
        ("colon", ":"),
        ("emDash", "—"),
        ("enDash", "–"),
        ("unknown", "-"),
    ] {
        let (dom, root) = xml_root(
            "sectPr",
            &format!(r#"<w:pgNumType w:start="0" w:chapStyle="1" w:chapSep="{name}"/>"#),
        );
        let page = apply_sect_pr(&dom, root, &Defaults::word().page);
        assert_eq!(
            (page.page_num_start, page.chap_style, page.chap_sep),
            (Some(0), Some(1), separator)
        );
    }
}

#[test]
fn border_edges_distinguish_absence_none_and_clamped_stroke_widths() {
    for attrs in [
        r#"w:val="none""#,
        r#"w:val="nil""#,
        r#"w:sz="0""#,
        r#"w:sz="-1""#,
    ] {
        let (dom, parent) = xml_root("tblBorders", &format!("<w:top {attrs}/>"));
        assert!(parse_border_edge(&dom, first_named(&dom, parent, "top").unwrap()).is_none());
    }
    for (attrs, width, color) in [
        ("", 1.0, [0.0; 3]),
        (
            r#"w:val="double" w:sz="8" w:color="FF0000""#,
            3.0,
            [1.0, 0.0, 0.0],
        ),
        (r#"w:sz="1" w:color="bad""#, 0.24, [0.0; 3]),
        (r#"w:sz="bad""#, 1.0, [0.0; 3]),
    ] {
        let (dom, parent) = xml_root("tblBorders", &format!("<w:top {attrs}/>"));
        assert_eq!(
            parse_border_edge(&dom, first_named(&dom, parent, "top").unwrap()),
            Some((color, width))
        );
    }
    let (dom, root) = xml_root(
        "tblPr",
        r#"<w:tblBorders><w:top w:val="none"/><w:bottom w:val="nil"/></w:tblBorders>"#,
    );
    let borders = parse_tbl_borders(&dom, root).expect("explicit empty override");
    assert!(
        !borders.top
            && !borders.bottom
            && !borders.left
            && !borders.right
            && !borders.inside_h
            && !borders.inside_v
    );
    assert_eq!((borders.width, borders.outer_width), (0.5, 0.5));
}

#[test]
fn row_exception_edges_use_outer_width_only_at_the_table_rim() {
    let borders = TblBorders {
        top: true,
        bottom: true,
        left: true,
        right: true,
        inside_h: true,
        inside_v: true,
        color: [1.0, 0.0, 0.0],
        width: 1.0,
        outer_width: 3.0,
    };
    for first_row in [false, true] {
        for last_row in [false, true] {
            for first_col in [false, true] {
                for last_col in [false, true] {
                    let cell = row_exception_cell_borders(
                        borders, first_row, last_row, first_col, last_col,
                    );
                    for (edge, rim) in [
                        (cell.top, first_row),
                        (cell.bottom, last_row),
                        (cell.left, first_col),
                        (cell.right, last_col),
                    ] {
                        assert_eq!(edge, Some(([1.0, 0.0, 0.0], if rim { 3.0 } else { 1.0 })));
                    }
                }
            }
        }
    }
    let absent = TblBorders {
        top: false,
        bottom: false,
        left: false,
        right: false,
        inside_h: false,
        inside_v: false,
        ..borders
    };
    for rim in [false, true] {
        let cell = row_exception_cell_borders(absent, rim, rim, rim, rim);
        assert_eq!(
            (cell.top, cell.bottom, cell.left, cell.right),
            (None, None, None, None)
        );
    }
}

#[test]
fn comment_range_end_requires_live_preceding_content_or_its_own_start() {
    for (prefix, live) in [
        ("", false),
        ("<w:r><w:t/></w:r>", false),
        ("<w:r><w:t>x</w:t></w:r>", true),
        ("<w:r><w:delText>x</w:delText></w:r>", true),
        (r#"<w:commentRangeStart w:id="2"/>"#, true),
        (r#"<w:commentRangeStart w:id="3"/>"#, false),
    ] {
        let (dom, root) = xml_root("p", &format!(r#"{prefix}<w:commentRangeEnd w:id="2"/>"#));
        assert_eq!(
            comment_range_end_is_live(&dom, first_named(&dom, root, "commentRangeEnd").unwrap()),
            live
        );
    }
    for tag in [
        "drawing",
        "pict",
        "object",
        "sym",
        "tab",
        "commentReference",
    ] {
        let (dom, root) = xml_root(
            "p",
            &format!(r#"<w:r><w:{tag}/></w:r><w:commentRangeEnd w:id="2"/>"#),
        );
        assert!(
            comment_range_end_is_live(&dom, first_named(&dom, root, "commentRangeEnd").unwrap()),
            "{tag}"
        );
    }
    for tag in ["body", "tc", "document"] {
        let (dom, root) = xml_root(tag, r#"<w:commentRangeEnd w:id="2"/>"#);
        assert!(!comment_range_end_is_live(
            &dom,
            first_named(&dom, root, "commentRangeEnd").unwrap()
        ));
    }
}

#[test]
fn page_selection_sorts_deduplicates_and_reports_the_largest_invalid_page() {
    // Document pages numbered as given (a gap is a page changed_only
    // dropped).
    let kept = |numbers: &[usize]| -> Vec<pdf::Page> {
        numbers
            .iter()
            .map(|&number| {
                let mut page = pdf::Page::new(612.0, 792.0);
                page.number = number;
                page
            })
            .collect()
    };
    assert_eq!(
        selected_pages(None, &kept(&[1, 2, 3]), 3).unwrap(),
        vec![0, 1, 2]
    );
    assert!(selected_pages(None, &[], 0).unwrap().is_empty());
    assert!(selected_pages(Some(vec![]), &[], 0).unwrap().is_empty());
    assert_eq!(
        selected_pages(Some(vec![2, 0, 2, 1]), &kept(&[1, 2, 3]), 3).unwrap(),
        vec![0, 1, 2]
    );
    for count in [0, 1, 3] {
        let numbers: Vec<usize> = (1..=count).collect();
        let result = selected_pages(Some(vec![0, count, count + 1]), &kept(&numbers), count);
        assert!(
            matches!(result,Err(ConvertError::PageOutOfRange{requested,page_count}) if requested==count+1 && page_count==count)
        );
    }
    // Pages 2 and 4 of 4 kept: page 4 is the second kept; page 1 was
    // dropped; the listing (5) lies past the layout's 4.
    let gaps = kept(&[2, 4, 5]);
    assert_eq!(
        selected_pages(Some(vec![3, 4]), &gaps, 4).unwrap(),
        vec![1, 2]
    );
    assert!(matches!(
        selected_pages(Some(vec![0, 1]), &gaps, 4),
        Err(ConvertError::PageNotKept { requested: 0 })
    ));
    assert!(matches!(
        selected_pages(Some(vec![5]), &gaps, 4),
        Err(ConvertError::PageOutOfRange {
            requested: 5,
            page_count: 5
        })
    ));
    assert_eq!(
        ConvertError::PageOutOfRange {
            requested: 1,
            page_count: 1
        }
        .to_string(),
        "page 2 is out of range: the document has 1 page"
    );
    assert_eq!(
        ConvertError::PageOutOfRange {
            requested: 3,
            page_count: 2
        }
        .to_string(),
        "page 4 is out of range: the document has 2 pages"
    );
    assert_eq!(
        ConvertError::PageNotKept { requested: 2 }.to_string(),
        "page 3 is not among the kept pages (changed pages only)"
    );
    assert_eq!(
        ConvertError::Emit("probe".into()).to_string(),
        "emitting PDF: probe"
    );
    assert_eq!(
        ConvertError::Raster("probe".into()).to_string(),
        "rasterizing PNG: probe"
    );
}

#[test]
fn line_box_grid_and_page_fit_distinguish_exact_minimum_and_multiplier() {
    let mut style = Defaults::word().para;
    style.line_mult = 2.0;
    assert_eq!(line_box_from_natural(12.0, &style), 24.0);
    assert_eq!(grid_line_box(12.0, &style, 18.0), 36.0);
    assert_eq!(line_fit_need(12.0, 8.0, &style, 24.0), 12.0);
    // A face over one grid line: the lines it needs or the multiple's,
    // the more (Word probe 2026-10-15), not its multiplied face snapped.
    assert_eq!(grid_line_box(20.0, &style, 18.0), 36.0);
    style.line_mult = 3.0;
    assert_eq!(grid_line_box(20.0, &style, 18.0), 54.0);
    style.line_mult = 0.0;
    assert_eq!(line_box_from_natural(12.0, &style), 12.0);
    assert_eq!(grid_line_box(12.0, &style, 18.0), 18.0);
    // No multiple set: a face over one grid line takes the lines it needs.
    assert_eq!(grid_line_box(20.0, &style, 18.0), 36.0);
    style.line_exact = Some(4.0);
    assert_eq!(line_fit_need(12.0, 8.0, &style, 4.0), 4.0);
    assert_eq!(para_grid_pitch(&style, 18.0), 0.0);
    style.line_exact = None;
    style.line_at_least = Some(20.0);
    assert_eq!(line_box_from_natural(12.0, &style), 20.0);
    assert_eq!(line_box_from_natural(24.0, &style), 24.0);
    assert_eq!(line_fit_need(12.0, 8.0, &style, 20.0), 20.0);
    assert_eq!(grid_line_box(12.0, &style, 18.0), 36.0);
    style.snap_to_grid = false;
    assert_eq!(para_grid_pitch(&style, 18.0), 0.0);
    style.snap_to_grid = true;
    assert_eq!(para_grid_pitch(&style, 18.0), 18.0);
    assert_eq!(snap_doc_grid(0.0, 18.0), 18.0);
    assert_eq!(snap_doc_grid(12.0, 0.5), 12.0);
}

#[test]
fn missing_glyph_detection_exempts_spaces_and_handles_ligature_counts() {
    for (chars, glyphs, missing) in [
        (vec![' '], vec![(0, 1.0)], false),
        (vec!['x'], vec![(0, 1.0)], true),
        (vec!['x'], vec![(7, 1.0)], false),
        (vec!['f', 'i'], vec![(0, 1.0)], true),
        (vec!['f', 'i'], vec![(7, 1.0)], false),
        (vec![' ', '\t'], vec![(0, 1.0)], false),
        (vec![], vec![], false),
    ] {
        assert_eq!(shaped_lacks_ink(&chars, &glyphs), missing);
    }
}

#[test]
fn contextual_spacing_is_directional_and_sum_spacing_adds_both_values() {
    for prev_context in [false, true] {
        for next_context in [false, true] {
            for same in [false, true] {
                for sum in [false, true] {
                    let mut prev = Defaults::word().para;
                    prev.style_id = "A".into();
                    prev.contextual = prev_context;
                    let mut next = Defaults::word().para;
                    next.style_id = if same { "A" } else { "B" }.into();
                    next.contextual = next_context;
                    next.sum_spacing = sum;
                    let after = if prev_context && same { 0.0 } else { 6.0 };
                    let before = if next_context && same {
                        0.0
                    } else if sum {
                        10.0
                    } else {
                        4.0
                    };
                    assert_eq!(contextual_gap(&prev, &next, 6.0, 10.0), after + before);
                }
            }
        }
    }
    assert_eq!(before_past(3.0, 6.0, false), 0.0);
}

#[test]
fn jpeg_header_scan_handles_restart_markers_noise_and_truncation() {
    for marker in [0xC0, 0xC1, 0xC2] {
        let bytes = vec![
            0xFF, 0xD8, 0, 0xFF, 0xD0, 0xFF, 0xD9, 0xFF, 0xD8, 0xFF, 0xE0, 0, 2, 0xFF, marker, 0,
            8, 8, 0, 3, 0, 5, 3,
        ];
        assert_eq!(jpeg_info(&bytes), Some((5, 3, 3)));
    }
    for bytes in [
        vec![],
        vec![0xFF, 0xD8],
        vec![0, 0, 0, 0],
        vec![0xFF, 0xD8, 0xFF, 0xC0, 0, 8, 8, 0, 3, 0, 5],
        vec![0xFF, 0xD8, 0xFF, 0xE0, 0, 255, 0, 0, 0, 0, 0, 0],
    ] {
        assert_eq!(jpeg_info(&bytes), None);
    }
}

#[test]
fn theme_font_slots_and_language_override_resolve_without_system_fonts() {
    let theme = ThemeFonts {
        major: Some("Major Latin".into()),
        minor: Some("Minor Latin".into()),
        major_ea: Some("Major CJK".into()),
        minor_ea: Some("Minor CJK".into()),
        major_cs: Some("Major Script".into()),
        minor_cs: None,
        script_fonts: HashMap::from([
            (("minor".into(), "Arab".into()), "Arabic fallback".into()),
            (("major".into(), "Jpan".into()), "Japanese major".into()),
            (("minor".into(), "Hans".into()), "Chinese minor".into()),
        ]),
        ..ThemeFonts::default()
    };
    for (slot, face) in [
        ("majorEastAsia", "Major CJK"),
        ("minorEastAsia", "Minor CJK"),
        ("majorCs", "Major Script"),
        ("majorBidi", "Major Script"),
        ("minorCs", "Arabic fallback"),
        ("minorBidi", "Arabic fallback"),
    ] {
        assert_eq!(theme_script_face(&theme, slot).as_deref(), Some(face));
    }
    assert_eq!(theme_script_face(&theme, "unknown"), None);
    for (lang, script) in [
        ("JA-jp", Some("Jpan")),
        ("ko-KR", Some("Hang")),
        ("zh-Hant", Some("Hant")),
        ("zh-TW", Some("Hant")),
        ("zh-HK", Some("Hant")),
        ("zh-MO", Some("Hant")),
        ("zh-CN", Some("Hans")),
        ("en-US", None),
    ] {
        assert_eq!(lang_to_ooxml_script(lang), script);
    }
    for (slot, lang, expected) in [
        (Some("majorEastAsia"), Some("ja-JP"), "Japanese major"),
        (Some("minorEastAsia"), Some("zh-CN"), "Chinese minor"),
        (Some("majorEastAsia"), Some("ko-KR"), "Inherited"),
        (Some("majorEastAsia"), Some("en-US"), "Inherited"),
        (None, Some("ja-JP"), "Inherited"),
        (Some("majorEastAsia"), None, "Inherited"),
    ] {
        let mut style = Defaults::word().run;
        style.family_ea = Some("Inherited".into());
        style.ea_theme_slot = slot.map(str::to_string);
        style.lang_ea = lang.map(str::to_string);
        apply_theme_script_fonts(&mut style, &theme);
        assert_eq!(style.family_ea.as_deref(), Some(expected));
    }
}

#[test]
fn font_name_and_theme_precedence_keeps_ascii_and_hansi_separate() {
    let theme = ThemeFonts {
        major: Some("Major".into()),
        minor: Some("Minor".into()),
        major_ea: Some("CJK".into()),
        major_cs: Some("Script".into()),
        ..ThemeFonts::default()
    };
    for (attrs, family, hansi) in [
        (r#"w:hAnsi="Other""#, "Inherited", Some("Other")),
        (r#"w:hAnsi="inherited""#, "Inherited", None),
        (r#"w:ascii="Explicit" w:hAnsi="explicit""#, "Explicit", None),
        (
            r#"w:ascii="Explicit" w:hAnsi="Other""#,
            "Explicit",
            Some("Other"),
        ),
        (
            r#"w:asciiTheme="minorHAnsi" w:hAnsi="Other""#,
            "Minor",
            Some("Other"),
        ),
        (r#"w:asciiTheme="majorHAnsi""#, "Major", None),
        (
            r#"w:ascii="Aptos Display" w:asciiTheme="majorHAnsi""#,
            "Major",
            None,
        ),
        (
            r#"w:ascii="Explicit" w:asciiTheme="majorHAnsi""#,
            "Explicit",
            None,
        ),
        (r#"w:asciiTheme="majorBidi""#, "Script", None),
        (r#"w:asciiTheme="unknown""#, "Inherited", None),
    ] {
        let (dom, root) = xml_root("rPr", &format!("<w:rFonts {attrs}/>"));
        let mut style = Defaults::word().run;
        style.family = "Inherited".into();
        apply_rpr(&dom, root, &mut style, &theme);
        assert_eq!(style.family, family, "{attrs}");
        assert_eq!(style.family_hansi.as_deref(), hansi, "{attrs}");
    }
    for (attrs, ea, cs, hint) in [
        (
            r#"w:eastAsia="Named CJK" w:eastAsiaTheme="majorEastAsia" w:cs="Named CS" w:cstheme="majorBidi" w:hint="eastAsia""#,
            "Named CJK",
            "Named CS",
            FontHint::EastAsia,
        ),
        (
            r#"w:eastAsiaTheme="majorEastAsia" w:csTheme="majorBidi" w:hint="cs""#,
            "CJK",
            "Script",
            FontHint::Complex,
        ),
        (
            r#"w:eastAsiaTheme="majorEastAsia" w:cstheme="majorBidi" w:hint="other""#,
            "CJK",
            "Script",
            FontHint::Default,
        ),
    ] {
        let (dom, root) = xml_root("rPr", &format!("<w:rFonts {attrs}/>"));
        let mut style = Defaults::word().run;
        apply_rpr(&dom, root, &mut style, &theme);
        assert_eq!(
            (style.family_ea.as_deref(), style.family_cs.as_deref()),
            (Some(ea), Some(cs))
        );
        assert_eq!(style.hint, hint);
    }
}

#[test]
fn font_hints_claim_symbols_while_script_letters_keep_their_own_face() {
    let mut style = Defaults::word().run;
    style.family = "Latin".into();
    style.family_ea = Some("CJK".into());
    style.family_cs = Some("Complex".into());
    for (hint, text, expected) in [
        (FontHint::Default, "漢字", "CJK"),
        (FontHint::Default, "العربية", "Complex"),
        (FontHint::EastAsia, "—“”", "CJK"),
        (FontHint::EastAsia, "abc123", "Latin"),
        (FontHint::EastAsia, "гарантиране", "Latin"),
        (FontHint::Default, "—", "Latin"),
        (FontHint::Complex, "-", "Complex"),
        (FontHint::Complex, "12", "Latin"),
        (FontHint::Complex, "A", "Latin"),
    ] {
        style.hint = hint;
        assert_eq!(paint_family(&style, text), expected, "{hint:?} {text}");
    }
}

#[test]
fn theme_parser_handles_empty_slots_and_limits_format_style_lists() {
    let theme = parse_theme_xml(
        r#"<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:themeElements>
          <a:fontScheme><a:majorFont><a:latin typeface="Major"/><a:ea typeface=" "/><a:cs/><a:font script="Jpan" typeface="Japanese"/><a:font script="Hang"/><a:font typeface="unused"/></a:majorFont><a:minorFont><a:latin typeface="Minor"/><a:cs typeface="Arabic"/></a:minorFont></a:fontScheme>
          <a:clrScheme><a:dk1><a:sysClr lastClr="112233"/></a:dk1><a:accent1><a:srgbClr val="FF0000"/></a:accent1><a:accent2><a:srgbClr val="bad"/></a:accent2></a:clrScheme>
          <a:fmtScheme><a:fillStyleLst>
            <a:solidFill><a:srgbClr val="00FF00"/></a:solidFill><a:gradFill/><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:srgbClr val="0000FF"/></a:solidFill>
          </a:fillStyleLst><a:lnStyleLst><a:unknown/><a:ln w="1"><a:solidFill><a:srgbClr val="FF0000"/></a:solidFill></a:ln><a:ln w="127000"/><a:ln w="bad"/><a:ln w="12700"/></a:lnStyleLst></a:fmtScheme>
        </a:themeElements></a:theme>"#,
    );
    assert_eq!(
        (
            theme.major.as_deref(),
            theme.minor.as_deref(),
            theme.major_ea.as_deref(),
            theme.major_cs.as_deref(),
            theme.minor_cs.as_deref()
        ),
        (Some("Major"), Some("Minor"), None, None, Some("Arabic"))
    );
    assert_eq!(theme.script_fonts.len(), 1);
    assert_eq!(
        theme
            .script_fonts
            .get(&("major".into(), "Jpan".into()))
            .map(String::as_str),
        Some("Japanese")
    );
    assert_eq!(theme.fill_styles, [Some([0.0, 1.0, 0.0]), None, None]);
    assert_eq!(theme.ln_widths, [Some(0.4), Some(4.0), None]);
    assert_eq!(theme.ln_colors, [Some([1.0, 0.0, 0.0]), None, None]);
    assert_eq!(
        theme.slot_color("text1"),
        Some([17.0 / 255.0, 34.0 / 255.0, 51.0 / 255.0])
    );
    for idx in [i32::MIN, -1, 0, 4, i32::MAX] {
        assert_eq!(theme.fill_style(idx), None);
        assert_eq!(theme.ln_width_pt(idx), None);
        assert_eq!(theme.ln_color(idx), None);
    }
    assert_eq!(theme.fill_style(1), Some([0.0, 1.0, 0.0]));
    assert_eq!(theme.ln_width_pt(2), Some(4.0));
    let empty = parse_theme_xml("");
    assert!(empty.major.is_none() && empty.colors.is_empty());
}

#[test]
fn numeric_character_references_and_font_scans_skip_invalid_entries() {
    assert_eq!(
        char_refs("&#x3001;&#X41;&#65;&#bogus;&#xD800;&#1114112;&#xZZ;&#12").collect::<Vec<_>>(),
        vec!['、', 'A', 'A']
    );
    assert_eq!(
        font_attr_names(
            r#"w:ascii="Arial" w:ascii="" w:ascii="Arial" w:ascii="Calibri" w:ascii="unterminated"#,
            &["w:ascii=\""]
        ),
        vec!["Arial", "Calibri"]
    );
    assert!(!number_formats_write("<w:numFmt w:val='decimal'/>", is_cjk));
    assert!(number_formats_write(
        "<w:pgNumType w:fmt='ideographDigital'/>",
        is_cjk
    ));
    assert!(number_formats_write(
        "<w:numFmt w:val='thaiNumbers'/>",
        is_thai
    ));
}

#[test]
fn localized_numbering_zero_gaps_and_fallback_limits_are_explicit() {
    for (n, japanese, legal, chinese, taiwanese) in [
        (0, "零", "零", "〇", "零"),
        (10, "十", "拾", "十", "十"),
        (11, "十一", "拾壹", "十一", "十一"),
        (20, "二十", "貳拾", "二十", "二十"),
        (101, "百一", "壹佰零壹", "一百〇一", "一百零一"),
        (110, "百十", "壹佰壹拾", "一百一十", "一百一十"),
        (1001, "千一", "壹仟零壹", "一千〇一", "一千零一"),
        (10000, "一万", "壹萬", "一万", "一萬"),
        (10001, "一万一", "壹萬零壹", "一万〇一", "一萬零一"),
        (11000, "一万千", "壹萬壹仟", "一万一千", "一萬一千"),
    ] {
        assert_eq!(format_num(parse_num_fmt("japaneseCounting"), n), japanese);
        assert_eq!(
            format_num(parse_num_fmt("ideographLegalTraditional"), n),
            legal
        );
        assert_eq!(
            format_num(parse_num_fmt("chineseCountingThousand"), n),
            chinese
        );
        assert_eq!(
            format_num(parse_num_fmt("taiwaneseCountingThousand"), n),
            taiwanese
        );
    }
    for (n, expected) in [
        (0, "○"),
        (9, "九"),
        (10, "十"),
        (19, "十九"),
        (20, "二十"),
        (99, "九十九"),
        (100, "一○○"),
        (101, "一○一"),
    ] {
        assert_eq!(chinese_counting_label(n), expected);
    }
    for (n, expected) in [
        (0, "0"),
        (1, "א"),
        (10, "י"),
        (15, "טו"),
        (16, "טז"),
        (115, "קטו"),
        (116, "קטז"),
        (999, "תתקצט"),
        (1000, "1000"),
    ] {
        assert_eq!(hebrew1_label(n), expected);
    }
}

#[test]
fn korean_and_thai_number_words_cover_units_and_decimal_fallbacks() {
    for (n, expected) in [
        (0, "영"),
        (1, "일"),
        (10, "십"),
        (11, "십일"),
        (20, "이십"),
        (101, "백일"),
        (2100, "이천백"),
        (9999, "구천구백구십구"),
        (10000, "10000"),
    ] {
        assert_eq!(korean_counting_label(n), expected);
    }
    for (n, expected) in [
        (0, "영"),
        (1, "하나"),
        (10, "열"),
        (20, "스물"),
        (21, "스물하나"),
        (99, "아흔아홉"),
        (100, "100"),
    ] {
        assert_eq!(korean_legal_label(n), expected);
    }
    for (n, expected) in [
        (0, "ศูนย์"),
        (1, "หนึ่ง"),
        (10, "สิบ"),
        (11, "สิบเอ็ด"),
        (12, "สิบสอง"),
        (19, "สิบเก้า"),
        (20, "ยี่สิบ"),
        (21, "ยี่สิบเอ็ด"),
        (22, "ยี่สิบสอง"),
        (30, "สามสิบ"),
        (31, "สามสิบเอ็ด"),
        (99, "เก้าสิบเก้า"),
        (100, "100"),
    ] {
        assert_eq!(thai_counting_label(n), expected);
    }
    assert_eq!(hindi_counting_label(0), "शून्य");
    assert_eq!(hindi_counting_label(20), "बीस");
    assert_eq!(hindi_counting_label(21), "21");
}

#[test]
fn ordinal_text_words_and_suffixes_cover_irregular_ones_and_tens() {
    for (n, expected) in [
        (0, "Zeroth"),
        (1, "First"),
        (2, "Second"),
        (3, "Third"),
        (4, "Fourth"),
        (5, "Fifth"),
        (6, "Sixth"),
        (7, "Seventh"),
        (8, "Eighth"),
        (9, "Ninth"),
        (10, "Tenth"),
        (11, "Eleventh"),
        (12, "Twelfth"),
        (13, "Thirteenth"),
        (14, "Fourteenth"),
        (15, "Fifteenth"),
        (16, "Sixteenth"),
        (17, "Seventeenth"),
        (18, "Eighteenth"),
        (19, "Nineteenth"),
        (20, "Twentieth"),
        (21, "Twenty-First"),
        (22, "Twenty-Second"),
        (23, "Twenty-Third"),
        (24, "Twenty-Fourth"),
        (25, "Twenty-Fifth"),
        (28, "Twenty-Eighth"),
        (29, "Twenty-Ninth"),
        (30, "Thirtieth"),
        (40, "Fortieth"),
        (50, "Fiftieth"),
        (60, "Sixtieth"),
        (70, "Seventieth"),
        (80, "Eightieth"),
        (90, "Ninetieth"),
        (100, "100th"),
    ] {
        assert_eq!(ordinal_text_label(n), expected, "{n}");
    }
    for (n, expected) in [
        (1, "1st"),
        (2, "2nd"),
        (3, "3rd"),
        (4, "4th"),
        (11, "11th"),
        (12, "12th"),
        (13, "13th"),
        (21, "21st"),
        (111, "111th"),
        (112, "112th"),
        (113, "113th"),
    ] {
        assert_eq!(ordinal_label(n), expected);
    }
    assert_eq!(cardinal_label(100), "100");
    assert_eq!(cardinal_label(90), "Ninety");
    assert_eq!(cardinal_label(99), "Ninety-Nine");
}

#[test]
fn numbering_marker_font_takes_hansi_only_for_non_ascii_output() {
    for (fmt, text, expected) in [
        ("decimal", "%1.", ""),
        ("upperRoman", "(%1)", ""),
        ("bullet", "o", ""),
        ("bullet", "•", "Other"),
        ("decimal", "§ %1", "Other"),
        ("ideographDigital", "%1.", "Other"),
        ("decimal", "%1%2.", ""),
    ] {
        let (dom, root) = xml_root("lvl", r#"<w:rPr><w:rFonts w:hAnsi="Other"/></w:rPr>"#);
        assert_eq!(lvl_marker_family(&dom, root, fmt, text), expected);
    }
    let (dom, root) = xml_root(
        "lvl",
        r#"<w:rPr><w:rFonts w:ascii="Explicit" w:hAnsi="Other"/></w:rPr>"#,
    );
    assert_eq!(lvl_marker_family(&dom, root, "bullet", "•"), "Explicit");
    for content in ["", "<w:rPr/>"] {
        let (dom, root) = xml_root("lvl", content);
        assert_eq!(lvl_marker_family(&dom, root, "bullet", "•"), "");
    }
}

#[test]
fn numbering_level_defaults_and_explicit_marker_properties_survive_parsing() {
    let (dom, root) = xml_root(
        "lvl",
        r#"<w:start w:val="bad"/><w:rPr><w:sz w:val="bad"/><w:b w:val="false"/><w:i w:val="off"/><w:u w:val="none"/></w:rPr><w:pPr><w:ind w:start="bad" w:hanging="bad"/></w:pPr>"#,
    );
    let level = parse_num_level(&dom, root);
    assert_eq!(
        (level.text.as_str(), level.start, level.left, level.hanging),
        ("%1.", 1, 0.0, 0.0)
    );
    assert_eq!(
        (level.size, level.underline, level.bold, level.italic),
        (None, false, false, false)
    );
    for underline in ["", r#"w:val="0""#, r#"w:val="NONE""#, r#"w:val="single""#] {
        let (dom, root) = xml_root(
            "lvl",
            &format!(
                r#"<w:start w:val="0"/><w:lvlText w:val="(%1)"/><w:suff w:val="NOTHING"/><w:lvlJc w:val="END"/><w:rPr><w:sz w:val="30"/><w:b/><w:i/><w:u {underline}/></w:rPr><w:pPr><w:ind w:start="720" w:hanging="360"/><w:jc w:val="center"/></w:pPr>"#
            ),
        );
        let level = parse_num_level(&dom, root);
        assert_eq!(
            (level.text.as_str(), level.start, level.left, level.hanging),
            ("(%1)", 0, 36.0, 18.0)
        );
        assert_eq!(
            (level.size, level.bold, level.italic),
            (Some(15.0), true, true)
        );
        assert_eq!(
            level.underline,
            underline.is_empty() || underline.contains("single")
        );
        assert!(level.suff_nothing && level.jc_right);
        assert_eq!(level.jc, Some(Align::Center));
    }
}

#[test]
fn fixed_advance_splitting_preserves_unicode_text_and_revision_metadata() {
    let mut run = TextRun::new("a漢字b", Defaults::word().run);
    run.rev = true;
    run.style.bold = true;
    let mut runs = vec![run];
    split_fixed_advances(&mut runs, &[false, true, true, false], |_| 6.0);
    assert_eq!(
        runs.iter().map(|r| r.text.as_str()).collect::<Vec<_>>(),
        vec!["a", "漢字", "b"]
    );
    assert_eq!(
        runs.iter()
            .map(|r| r.style.fixed_advance)
            .collect::<Vec<_>>(),
        vec![0.0, 6.0, 0.0]
    );
    assert!(runs.iter().all(|r| r.rev && r.style.bold));
    split_fixed_advances(&mut runs, &[false, false, false, false], |_| 99.0);
    assert_eq!(
        runs.iter().map(|r| r.text.as_str()).collect::<String>(),
        "a漢字b"
    );
    assert_eq!(runs.len(), 3);
    assert_eq!(runs[1].style.fixed_advance, 6.0);
}

#[test]
fn fixed_advance_splitting_keeps_fields_notes_and_layout_sentinels_atomic() {
    for kind in 0..14 {
        let mut run = TextRun::new("漢字", Defaults::word().run);
        match kind {
            0 => run.field = FieldKind::Page,
            1 => run.comments.push(CommentNote {
                id: "1".into(),
                author: "Author".into(),
                initials: "A".into(),
                label: "1".into(),
                color: [0.0; 3],
                after: false,
                resolved: false,
                text: "note".into(),
            }),
            2 => run.pageref = Some("bookmark".into()),
            3 => run.ref_name = Some("bookmark".into()),
            4 => run.rule = Some(([0.0; 3], 1.0)),
            5 => run.footnote_id = Some("1".into()),
            6 => run.endnote_id = Some("2".into()),
            7 => run.note_ref = true,
            8 => run.note_rule = true,
            9 => run.ends_line = true,
            10 => run.list_marker = true,
            11 => run.checkbox = Some(true),
            12 => run.strut = true,
            _ => run.hf_para = Some(std::rc::Rc::new(Defaults::word().para)),
        }
        let original = run.clone();
        let mut runs = vec![run];
        let calls = std::cell::Cell::new(0);
        split_fixed_advances(&mut runs, &[true, false], |_| {
            calls.set(calls.get() + 1);
            6.0
        });
        assert_eq!(calls.get(), 0, "protected kind {kind}");
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].text, "漢字");
        assert_eq!(runs[0].style.fixed_advance, 0.0);
        assert!(runs[0].field == original.field);
        assert_eq!(runs[0].comments.len(), original.comments.len());
        assert_eq!(
            (
                &runs[0].pageref,
                &runs[0].ref_name,
                &runs[0].footnote_id,
                &runs[0].endnote_id
            ),
            (
                &original.pageref,
                &original.ref_name,
                &original.footnote_id,
                &original.endnote_id
            )
        );
        assert_eq!(
            (
                runs[0].rule,
                runs[0].note_ref,
                runs[0].note_rule,
                runs[0].ends_line,
                runs[0].list_marker,
                runs[0].checkbox,
                runs[0].strut
            ),
            (
                original.rule,
                original.note_ref,
                original.note_rule,
                original.ends_line,
                original.list_marker,
                original.checkbox,
                original.strut
            )
        );
        assert_eq!(runs[0].hf_para.is_some(), original.hf_para.is_some());
    }
    let mut runs = vec![
        TextRun::new("plain", Defaults::word().run),
        TextRun::new("字", Defaults::word().run),
    ];
    split_fixed_advances(
        &mut runs,
        &[false, false, false, false, false, true],
        |_| 6.0,
    );
    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0].text, "plain");
    assert_eq!(runs[0].style.fixed_advance, 0.0);
    assert_eq!(runs[1].style.fixed_advance, 6.0);
}

#[test]
fn section_page_number_formats_keep_the_authored_scheme() {
    for (name, expected) in [
        ("lowerRoman", PageNumFmt::LowerRoman),
        ("upperRoman", PageNumFmt::UpperRoman),
        ("lowerLetter", PageNumFmt::LowerLetter),
        ("upperLetter", PageNumFmt::UpperLetter),
        ("decimalZero", PageNumFmt::DecimalZero),
        ("cardinalText", PageNumFmt::CardinalText),
        ("ordinal", PageNumFmt::Ordinal),
        ("ordinalText", PageNumFmt::OrdinalText),
        ("hex", PageNumFmt::Hex),
        ("chicago", PageNumFmt::Chicago),
        ("ideographDigital", PageNumFmt::IdeographDigital),
        ("ideographTraditional", PageNumFmt::IdeographTraditional),
        ("ideographZodiac", PageNumFmt::IdeographZodiac),
        (
            "ideographZodiacTraditional",
            PageNumFmt::IdeographZodiacTraditional,
        ),
        (
            "ideographLegalTraditional",
            PageNumFmt::IdeographLegalTraditional,
        ),
        (
            "ideographEnclosedCircle",
            PageNumFmt::IdeographEnclosedCircle,
        ),
        ("japaneseCounting", PageNumFmt::JapaneseCounting),
        ("aiueo", PageNumFmt::Aiueo),
        ("iroha", PageNumFmt::Iroha),
        ("decimalFullWidth", PageNumFmt::DecimalFullWidth),
        ("ganada", PageNumFmt::Ganada),
        ("chosung", PageNumFmt::Chosung),
        ("decimalEnclosedCircle", PageNumFmt::DecimalEnclosedCircle),
        ("decimalEnclosedParen", PageNumFmt::DecimalEnclosedParen),
        (
            "decimalEnclosedFullstop",
            PageNumFmt::DecimalEnclosedFullstop,
        ),
        ("hebrew1", PageNumFmt::Hebrew1),
        ("hebrew2", PageNumFmt::Hebrew2),
        ("arabicAlpha", PageNumFmt::ArabicAlpha),
        ("arabicAbjad", PageNumFmt::ArabicAbjad),
        ("russianLower", PageNumFmt::RussianLower),
        ("russianUpper", PageNumFmt::RussianUpper),
        ("thaiNumbers", PageNumFmt::ThaiNumbers),
        ("thaiLetters", PageNumFmt::ThaiLetters),
        ("thaiCounting", PageNumFmt::ThaiCounting),
        ("hindiNumbers", PageNumFmt::HindiNumbers),
        ("hindiVowels", PageNumFmt::HindiVowels),
        ("hindiConsonants", PageNumFmt::HindiConsonants),
        ("hindiCounting", PageNumFmt::HindiCounting),
        ("koreanCounting", PageNumFmt::KoreanCounting),
        ("koreanDigital", PageNumFmt::KoreanDigital),
        ("koreanDigital2", PageNumFmt::KoreanDigital2),
        ("koreanLegal", PageNumFmt::KoreanLegal),
        ("unknown", PageNumFmt::Decimal),
    ] {
        let (dom, root) = xml_root("sectPr", &format!(r#"<w:pgNumType w:fmt="{name}"/>"#));
        assert!(
            apply_sect_pr(&dom, root, &Defaults::word().page).page_num_fmt == expected,
            "{name}"
        );
    }
}

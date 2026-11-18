// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;

fn dom(fragment: &str) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&format!(
        r#"<a:root xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart">{fragment}</a:root>"#
    ));
    let root = dom.root(doc).unwrap();
    (dom, root)
}

#[test]
fn malformed_chart_points_do_not_erase_valid_sparse_indexed_peers() {
    // idx and count errors deliberately probe tolerant chart-cache input.
    // The cache also contains an absent v and an invalid numeric v; neither
    // is an indexed numeric point, while string category data stays intact.
    for (count, expected_count) in [
        ("", 0),
        ("<c:ptCount/>", 0),
        ("<c:ptCount val='invalid'/>", 0),
        ("<c:ptCount val='9'/>", 9),
    ] {
        let (dom, root) = dom(&format!(
            "<c:numLit>{count}<c:pt idx='0'/><c:pt idx='1'><c:v>invalid</c:v></c:pt><c:pt idx='6'><c:v> 2.5 </c:v></c:pt><c:pt idx='invalid'><c:v>-3</c:v></c:pt><c:pt><c:v>4</c:v></c:pt></c:numLit>"
        ));
        let before = dom.serialize_element(root);
        let cache = descendants_local(&dom, root, "numLit")[0];
        assert_eq!(
            chart_pts_indexed(&dom, cache),
            (vec![(6, 2.5), (3, -3.0), (4, 4.0)], expected_count)
        );
        assert_eq!(chart_pts(&dom, cache), ["invalid", " 2.5 ", "-3", "4"]);
        assert_eq!(dom.serialize_element(root), before);
    }
}

#[test]
fn automatic_chart_title_and_series_name_skip_blank_cached_nodes() {
    for (names, expected_name, expected_title) in [
        (
            "<c:tx><c:v> </c:v><c:v>\t</c:v><c:v> Owned </c:v></c:tx>",
            " Owned ",
            " Owned ",
        ),
        (
            "<c:tx><c:v> </c:v><c:v>\t</c:v></c:tx>",
            "Series 1",
            "Series 1",
        ),
        ("", "Series 1", "Series 1"),
    ] {
        let (dom, root) = dom(&format!(
            "<c:chart><c:title><c:tx><a:t> </a:t><c:v>\t</c:v></c:tx></c:title><c:plotArea><c:barChart><c:ser>{names}<c:val><c:numLit><c:pt idx='0'><c:v>2</c:v></c:pt></c:numLit></c:val></c:ser></c:barChart></c:plotArea></c:chart>"
        ));
        let before = dom.serialize_element(root);
        let ser = descendants_local(&dom, root, "ser")[0];
        assert_eq!(chart_ser_name(&dom, ser, 0), expected_name);
        assert_eq!(chart_title(&dom, root), expected_title);
        assert_eq!(dom.serialize_element(root), before);
    }
    let (dom, root) = dom(
        "<c:chart><c:title><c:tx><a:t> </a:t><c:v> Exact title </c:v></c:tx></c:title></c:chart>",
    );
    assert_eq!(chart_title(&dom, root), " Exact title ");
}

fn package(drawing: &str) -> PartFs {
    use std::io::{Cursor, Write};
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, payload) in [
        (
            "[Content_Types].xml",
            "<Types xmlns='http://schemas.openxmlformats.org/package/2006/content-types'><Default Extension='xml' ContentType='application/xml'/></Types>",
        ),
        (
            "word/_rels/document.xml.rels",
            "<Relationships xmlns='http://schemas.openxmlformats.org/package/2006/relationships'><Relationship Id='drawing' Type='http://schemas.openxmlformats.org/officeDocument/2006/relationships/diagramDrawing' Target='drawing.xml'/></Relationships>",
        ),
        ("word/drawing.xml", drawing),
    ] {
        zip.start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(payload.as_bytes()).unwrap();
    }
    PartFs::open(&zip.finish().unwrap().into_inner()).unwrap()
}

fn shape(w: i32, h: i32, fill: &str, line: &str) -> String {
    format!(
        "<a:sp><a:spPr><a:xfrm><a:off x='127000' y='254000'/><a:ext cx='{w}' cy='{h}'/></a:xfrm>{fill}{line}<a:prstGeom prst='roundRect'/></a:spPr><a:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:t> </a:t></a:r><a:r><a:t>Owned</a:t></a:r><a:r><a:t>label</a:t></a:r></a:p></a:txBody></a:sp>"
    )
}

#[test]
fn smartart_width_and_height_thresholds_decline_only_the_sub_eight_point_dimension() {
    let red = "<a:solidFill><a:srgbClr val='FF0000'/></a:solidFill>";
    for (w, h, expected_extent) in [
        (100330, 101600, None),
        (101600, 100330, None),
        (100330, 100330, None),
        (101600, 101600, Some((8.0, 8.0))),
        (127000, 101600, Some((10.0, 8.0))),
        (101600, 127000, Some((8.0, 10.0))),
    ] {
        let drawing = format!(
            r#"<a:root xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">{}</a:root>"#,
            shape(w, h, red, "")
        );
        let pkg = package(&drawing);
        let (host, root) = dom("");
        let host_before = host.serialize_element(root);
        let out = load_diag_shapes(
            &pkg,
            "word/document.xml",
            &host,
            root,
            &ThemeFonts::default(),
        );
        if let Some((expected_w, expected_h)) = expected_extent {
            let out = out.unwrap();
            assert_eq!(out.len(), 1);
            let s = &out[0];
            assert_eq!((s.x, s.y, s.w, s.h), (10.0, 20.0, expected_w, expected_h));
            assert_eq!(
                (s.fill, s.stroke, s.label.as_str(), s.round),
                (Some([1.0, 0.0, 0.0]), None, "Owned label", true)
            );
        } else {
            assert!(out.is_none());
        }
        assert_eq!(
            pkg.part_string("word/drawing.xml").as_deref(),
            Some(drawing.as_str())
        );
        assert_eq!(host.serialize_element(root), host_before);
    }
}

#[test]
fn smartart_near_white_fill_and_stroke_boundaries_preserve_complete_shape_records() {
    for (fill_color, stroke_color, width, expected_fill, expected_stroke) in [
        ("F2F2F2", None, "", Some([242.0 / 255.0; 3]), None),
        ("F3F3F3", None, "", None, None),
        ("FFF2FF", None, "", Some([1.0, 242.0 / 255.0, 1.0]), None),
        ("FFFFF2", None, "", Some([1.0, 1.0, 242.0 / 255.0]), None),
        (
            "FFFFFF",
            Some("0000FF"),
            "",
            Some([1.0; 3]),
            Some(([0.0, 0.0, 1.0], 1.0)),
        ),
        (
            "FFFFFF",
            Some("0000FF"),
            " w='invalid'",
            Some([1.0; 3]),
            Some(([0.0, 0.0, 1.0], 1.0)),
        ),
        (
            "FFFFFF",
            Some("F2F2F2"),
            " w='25400'",
            Some([1.0; 3]),
            Some(([242.0 / 255.0; 3], 2.0)),
        ),
        ("FFFFFF", Some("F3F3F3"), " w='25400'", None, None),
    ] {
        let fill = format!("<a:solidFill><a:srgbClr val='{fill_color}'/></a:solidFill>");
        let line = stroke_color
            .map(|c| {
                format!("<a:ln{width}><a:solidFill><a:srgbClr val='{c}'/></a:solidFill></a:ln>")
            })
            .unwrap_or_default();
        let drawing = format!(
            r#"<a:root xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">{}</a:root>"#,
            shape(127000, 254000, &fill, &line)
        );
        let pkg = package(&drawing);
        let (host, root) = dom("");
        let out = load_diag_shapes(
            &pkg,
            "word/document.xml",
            &host,
            root,
            &ThemeFonts::default(),
        );
        if expected_fill.is_some() || expected_stroke.is_some() {
            let out = out.unwrap();
            assert_eq!(out.len(), 1);
            let s = &out[0];
            assert_eq!((s.x, s.y, s.w, s.h), (10.0, 20.0, 10.0, 20.0));
            assert_eq!(
                (s.fill, s.stroke, s.label.as_str(), s.round),
                (expected_fill, expected_stroke, "Owned label", true)
            );
        } else {
            assert!(out.is_none());
        }
        assert_eq!(
            pkg.part_string("word/drawing.xml").as_deref(),
            Some(drawing.as_str())
        );
    }
}

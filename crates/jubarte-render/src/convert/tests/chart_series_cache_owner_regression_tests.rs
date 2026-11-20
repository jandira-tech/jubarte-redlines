// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[test]
fn skipped_empty_series_cache_does_not_assign_its_name_to_later_values() {
    for empty in [
        "<c:numLit><c:ptCount val='0'/></c:numLit>",
        "<c:numLit><c:ptCount val='1'/><c:pt idx='0'><c:v>invalid</c:v></c:pt></c:numLit>",
    ] {
        let xml = format!(
            r#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><c:chart><c:title><c:tx><c:v>Ownership</c:v></c:tx></c:title><c:plotArea><c:barChart><c:barDir val="col"/><c:grouping val="clustered"/><c:ser><c:idx val="0"/><c:order val="0"/><c:tx><c:v>Empty original</c:v></c:tx><c:val>{empty}</c:val></c:ser><c:ser><c:idx val="1"/><c:order val="1"/><c:tx><c:v>Owned values</c:v></c:tx><c:spPr><a:solidFill><a:srgbClr val="0000FF"/></a:solidFill></c:spPr><c:cat><c:strLit><c:ptCount val="1"/><c:pt idx="0"><c:v>Category</c:v></c:pt></c:strLit></c:cat><c:val><c:numLit><c:ptCount val="1"/><c:pt idx="0"><c:v>7</c:v></c:pt></c:numLit></c:val></c:ser></c:barChart></c:plotArea><c:legend><c:legendPos val="b"/></c:legend></c:chart></c:chartSpace>"#
        );
        // The zero-length cache is legal. The second case deliberately
        // exercises tolerant rejection of a malformed numeric cache.
        let chart = parse_chart_with(&xml, &ThemeFonts::default()).unwrap();
        assert_eq!(chart.title, "Ownership");
        assert_eq!(chart.names, ["Owned values"]);
        assert_eq!(chart.series, [vec![7.0]]);
        assert_eq!(chart.indexed, [vec![(0, 7.0)]]);
        assert_eq!(chart.x_indexed, [Vec::<(usize, f32)>::new()]);
        assert_eq!(chart.cats, ["Category"]);
        assert_eq!(chart.n_cats, 1);
        assert_eq!(chart.colors, [[0.0, 0.0, 1.0]]);
        assert!(chart.legend && matches!(chart.kind, ChartKind::Bar));
        assert_eq!(chart.x_scaling, (None, None));
        assert_eq!(chart.y_scaling, (None, None));
    }
}

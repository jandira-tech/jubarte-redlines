// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[test]
fn skipping_empty_series_keeps_source_color_indices_and_owned_names_across_chart_kinds() {
    let theme = ThemeFonts {
        colors: HashMap::from([
            ("accent1".into(), [1.0, 0.0, 0.0]),
            ("accent2".into(), [0.0, 1.0, 0.0]),
            ("accent3".into(), [0.0, 0.0, 1.0]),
            ("accent4".into(), [1.0, 1.0, 0.0]),
        ]),
        ..ThemeFonts::default()
    };
    for kind in [
        "barChart",
        "lineChart",
        "areaChart",
        "scatterChart",
        "radarChart",
    ] {
        for named in [false, true] {
            for empty_at in [0, 1, 2] {
                let mut series = String::new();
                let mut names = Vec::new();
                let mut colors = Vec::new();
                let mut values = Vec::new();
                for index in 0..3 {
                    let title = if named {
                        format!("<c:tx><c:v>Owner {index}</c:v></c:tx>")
                    } else {
                        String::new()
                    };
                    let cache = if index == empty_at {
                        "<c:numLit><c:ptCount val='0'/></c:numLit>".to_string()
                    } else {
                        format!(
                            "<c:numLit><c:ptCount val='1'/><c:pt idx='0'><c:v>{}</c:v></c:pt></c:numLit>",
                            index + 1
                        )
                    };
                    let value_tag = if kind == "scatterChart" {
                        "yVal"
                    } else {
                        "val"
                    };
                    series.push_str(&format!("<c:ser><c:idx val='{index}'/><c:order val='{index}'/>{title}<c:cat><c:strLit><c:ptCount val='1'/><c:pt idx='0'><c:v>Category</c:v></c:pt></c:strLit></c:cat><c:{value_tag}>{cache}</c:{value_tag}></c:ser>"));
                    if index != empty_at {
                        names.push(if named {
                            format!("Owner {index}")
                        } else {
                            format!("Series {}", index + 1)
                        });
                        colors.push(match index {
                            0 => [1.0, 0.0, 0.0],
                            1 => [0.0, 1.0, 0.0],
                            _ => [0.0, 0.0, 1.0],
                        });
                        values.push(vec![(index + 1) as f32]);
                    }
                }
                let source = format!(
                    r#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:chart><c:plotArea><c:{kind}>{series}</c:{kind}><c:valAx><c:axPos val="b"/><c:scaling><c:min val="0"/><c:max val="4"/></c:scaling></c:valAx><c:valAx><c:axPos val="l"/><c:scaling><c:min val="1"/><c:max val="5"/></c:scaling></c:valAx></c:plotArea></c:chart></c:chartSpace>"#
                );
                let chart = parse_chart_with(&source, &theme).unwrap();
                assert_eq!(chart.names, names, "{kind}/{named}/{empty_at}");
                assert_eq!(
                    chart.colors, colors,
                    "source indices own default palette slots"
                );
                assert_eq!(chart.series, values);
                assert_eq!(
                    chart.indexed,
                    values.iter().map(|v| vec![(0, v[0])]).collect::<Vec<_>>()
                );
                assert_eq!(chart.x_indexed, vec![vec![], vec![]]);
                assert_eq!(chart.cats, ["Category"]);
                assert_eq!(chart.n_cats, 1);
                assert_eq!(chart.title, "");
                assert!(!chart.legend);
                assert_eq!(chart.x_scaling, (Some(0.0), Some(4.0)));
                assert_eq!(chart.y_scaling, (Some(1.0), Some(5.0)));
                let right_kind = matches!(
                    (kind, &chart.kind),
                    ("barChart", ChartKind::Bar)
                        | ("lineChart", ChartKind::Line)
                        | ("areaChart", ChartKind::Area)
                        | ("scatterChart", ChartKind::Scatter)
                        | ("radarChart", ChartKind::Radar)
                );
                assert!(right_kind);
                assert_eq!(parse_chart_with(&source, &theme).unwrap().names, names);
            }
        }
    }
}

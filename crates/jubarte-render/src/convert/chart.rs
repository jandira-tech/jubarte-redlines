// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Charts: the chart part's series, axes and titles.

use super::*;

/// `c:pt` values with their `@idx`, and the cache's `c:ptCount`.
pub(super) fn chart_pts_indexed(dom: &Dom, node: NodeId) -> (Vec<(usize, f32)>, usize) {
    let mut pts = Vec::new();
    for (pos, pt) in descendants_local(dom, node, "pt").into_iter().enumerate() {
        let idx = attr_any(dom, pt, "idx")
            .and_then(|s| s.parse().ok())
            .unwrap_or(pos);
        if let Some(v) = descendants_local(dom, pt, "v").into_iter().next()
            && let Some(n) = parse_finite_f32(element_text(dom, v).trim())
        {
            pts.push((idx, n));
        }
    }
    let count = descendants_local(dom, node, "ptCount")
        .into_iter()
        .next()
        .and_then(|n| attr_any(dom, n, "val"))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    (pts, count)
}

/// `c:valAx` `c:scaling` bounds split by axis position: `(horizontal,
/// vertical)`, each `(min, max)`.
type AxisScaling = ((Option<f32>, Option<f32>), (Option<f32>, Option<f32>));

fn chart_val_ax_scaling(dom: &Dom, root: NodeId) -> AxisScaling {
    let mut horiz = (None, None);
    let mut vert = (None, None);
    for ax in descendants_local(dom, root, "valAx") {
        let pos = descendants_local(dom, ax, "axPos")
            .into_iter()
            .next()
            .and_then(|n| attr_any(dom, n, "val"))
            .unwrap_or("l");
        let bound = |local: &str| {
            descendants_local(dom, ax, "scaling")
                .into_iter()
                .next()
                .and_then(|sc| descendants_local(dom, sc, local).into_iter().next())
                .and_then(|n| attr_any(dom, n, "val"))
                .and_then(parse_finite_f32)
        };
        let b = (bound("min"), bound("max"));
        if matches!(pos, "b" | "t") {
            horiz = b;
        } else {
            vert = b;
        }
    }
    (horiz, vert)
}

pub(super) fn chart_pts(dom: &Dom, node: NodeId) -> Vec<String> {
    let mut pts = Vec::new();
    for pt in descendants_local(dom, node, "pt") {
        if let Some(v) = descendants_local(dom, pt, "v").into_iter().next() {
            pts.push(element_text(dom, v));
        }
    }
    pts
}

pub(super) fn chart_ser_name(dom: &Dom, ser: NodeId, idx: usize) -> String {
    if let Some(tx) = descendants_local(dom, ser, "tx").into_iter().next() {
        for v in descendants_local(dom, tx, "v") {
            let s = element_text(dom, v);
            if !s.trim().is_empty() {
                return s;
            }
        }
    }
    format!("Series {}", idx + 1)
}

pub(super) fn chart_title(dom: &Dom, root: NodeId) -> String {
    let deleted = descendants_local(dom, root, "autoTitleDeleted")
        .into_iter()
        .next()
        .and_then(|n| attr_any(dom, n, "val"))
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));
    if deleted {
        return String::new();
    }
    // The chart's own c:title (not an axis title): absent, the chart has
    // none (docxide case29: Word paints no title); present without text,
    // it is Word's automatic one, the lone series' name or "Chart Title".
    let chart = descendants_local(dom, root, "chart")
        .into_iter()
        .next()
        .unwrap_or(root);
    let Some(title) = (0..dom.child_count(chart))
        .map(|i| dom.child_at(chart, i))
        .find(|c| local_name_is(dom, *c, "title"))
    else {
        return String::new();
    };
    for local in ["t", "v"] {
        for n in descendants_local(dom, title, local) {
            let s = element_text(dom, n);
            if !s.trim().is_empty() {
                return s;
            }
        }
    }
    let series = descendants_local(dom, root, "ser");
    if series.len() == 1 {
        return chart_ser_name(dom, series[0], 0);
    }
    "Chart Title".into()
}

pub(super) fn chart_ser_color(dom: &Dom, ser: NodeId, idx: usize, theme: &ThemeFonts) -> [f32; 3] {
    for fill in descendants_local(dom, ser, "solidFill") {
        if let Some(parent) = dom.parent(fill)
            && local_name_is(dom, parent, "ln")
        {
            continue;
        }
        if let Some(rgb) = descendants_local(dom, fill, "schemeClr")
            .into_iter()
            .next()
            .and_then(|n| attr_any(dom, n, "val"))
            .and_then(|slot| theme.slot_color(slot))
        {
            return rgb;
        }
        if let Some(rgb) = descendants_local(dom, fill, "srgbClr")
            .into_iter()
            .next()
            .and_then(|n| attr_any(dom, n, "val"))
            .and_then(parse_hex_color)
        {
            return rgb;
        }
    }
    let slots = [
        "accent1", "accent2", "accent3", "accent4", "accent5", "accent6",
    ];
    theme
        .slot_color(slots[idx % slots.len()])
        .unwrap_or([0.5, 0.5, 0.5])
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
pub(super) fn parse_chart(xml: &str) -> Option<ChartData> {
    parse_chart_with(xml, &ThemeFonts::default())
}

pub(super) fn parse_chart_with(xml: &str, theme: &ThemeFonts) -> Option<ChartData> {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc)?;
    let is_pie = !descendants_local(&dom, root, "pieChart").is_empty();
    let is_line = !descendants_local(&dom, root, "lineChart").is_empty();
    let is_area = !descendants_local(&dom, root, "areaChart").is_empty();
    let is_scatter = !descendants_local(&dom, root, "scatterChart").is_empty();
    let is_radar = !descendants_local(&dom, root, "radarChart").is_empty();
    let host = descendants_local(&dom, root, "pieChart")
        .into_iter()
        .next()
        .or_else(|| {
            descendants_local(&dom, root, "lineChart")
                .into_iter()
                .next()
        })
        .or_else(|| {
            descendants_local(&dom, root, "areaChart")
                .into_iter()
                .next()
        })
        .or_else(|| {
            descendants_local(&dom, root, "scatterChart")
                .into_iter()
                .next()
        })
        .or_else(|| {
            descendants_local(&dom, root, "radarChart")
                .into_iter()
                .next()
        })
        .or_else(|| descendants_local(&dom, root, "barChart").into_iter().next())
        .unwrap_or(root);
    let mut cats = Vec::new();
    let mut series = Vec::new();
    let mut names = Vec::new();
    let mut colors = Vec::new();
    let mut indexed = Vec::new();
    let mut x_indexed = Vec::new();
    let mut n_cats = 0usize;
    for (idx, ser) in descendants_local(&dom, host, "ser").into_iter().enumerate() {
        if cats.is_empty() {
            if let Some(cat) = descendants_local(&dom, ser, "cat").into_iter().next() {
                cats = chart_pts(&dom, cat);
            } else if let Some(xval) = descendants_local(&dom, ser, "xVal").into_iter().next() {
                cats = chart_pts(&dom, xval);
            }
        }
        let val = descendants_local(&dom, ser, "yVal")
            .into_iter()
            .next()
            .or_else(|| descendants_local(&dom, ser, "val").into_iter().next());
        if let Some(val) = val {
            let nums: Vec<f32> = chart_pts(&dom, val)
                .iter()
                .filter_map(|s| parse_finite_f32(s))
                .collect();
            if !nums.is_empty() {
                names.push(chart_ser_name(&dom, ser, idx));
                colors.push(chart_ser_color(&dom, ser, idx, theme));
                series.push(nums);
                let (pts, count) = chart_pts_indexed(&dom, val);
                n_cats = n_cats
                    .max(count)
                    .max(pts.iter().map(|p| p.0 + 1).max().unwrap_or(0));
                indexed.push(pts);
                x_indexed.push(
                    descendants_local(&dom, ser, "xVal")
                        .into_iter()
                        .next()
                        .map(|x| chart_pts_indexed(&dom, x).0)
                        .unwrap_or_default(),
                );
            }
        }
    }
    if series.is_empty() {
        return None;
    }
    names.truncate(series.len());
    colors.truncate(series.len());
    if is_pie && let Some(vals) = series.first() {
        let slots = [
            "accent1", "accent2", "accent3", "accent4", "accent5", "accent6",
        ];
        colors = (0..vals.len())
            .map(|i| {
                theme
                    .slot_color(slots[i % slots.len()])
                    .unwrap_or([0.5, 0.5, 0.5])
            })
            .collect();
    }
    let legend = !descendants_local(&dom, root, "legend").is_empty();
    let n_cats = n_cats.max(cats.len());
    let (x_scaling, y_scaling) = chart_val_ax_scaling(&dom, root);
    Some(ChartData {
        indexed,
        x_indexed,
        n_cats,
        x_scaling,
        y_scaling,
        title: chart_title(&dom, root),
        cats,
        series,
        names,
        colors,
        legend,
        kind: if is_pie {
            ChartKind::Pie
        } else if is_line {
            ChartKind::Line
        } else if is_area {
            ChartKind::Area
        } else if is_scatter {
            ChartKind::Scatter
        } else if is_radar {
            ChartKind::Radar
        } else {
            ChartKind::Bar
        },
    })
}

pub(super) fn load_chart(
    pkg: &PartFs,
    main: &str,
    dom: &Dom,
    shape: NodeId,
    theme: &ThemeFonts,
) -> Option<ChartData> {
    let mut rid = None;
    for el in descendants_local(dom, shape, "chart") {
        if let Some(id) = attr_any(dom, el, "id") {
            rid = Some(id.to_string());
            break;
        }
    }
    let bytes = resolve_media(pkg, main, rid.as_deref()?)?;
    parse_chart_with(&String::from_utf8_lossy(&bytes), theme)
}

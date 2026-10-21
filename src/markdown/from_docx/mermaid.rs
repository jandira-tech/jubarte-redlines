// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Mermaid is a chart's readable layer: a `c:chartSpace` part holding a
//! pie, bar or line chart serializes to `pie`/`xychart-beta`, and the
//! fence in the view edits back into that part. Geometry, colours and
//! styling are the payload's, not the Mermaid's; the drawing envelope
//! carries them, so an unchanged fence restores the chart byte for byte
//! and an edited one only rewrites the part's title, categories and
//! values in place.

use std::fmt::Write as _;

use super::ooxml::{Element, Node, parse_xml};

/// The chart's series: name, categories and values in document order.
fn series(chart: &Element, kind: &str) -> Vec<(String, Vec<String>, Vec<String>)> {
    let mut charts = Vec::new();
    chart.find_all(kind, &mut charts);
    let mut out = Vec::new();
    for chart in charts {
        let mut sers = Vec::new();
        chart.find_all("ser", &mut sers);
        for ser in sers {
            let name = ser
                .find("tx")
                .and_then(|tx| tx.find("v"))
                .map(|v| v.text().trim().to_string())
                .unwrap_or_default();
            let mut cats: Vec<(u32, String)> = Vec::new();
            let mut vals: Vec<(u32, String)> = Vec::new();
            if let Some(cat) = ser.find("cat") {
                let mut refs = Vec::new();
                cat.find_all("pt", &mut refs);
                for pt in refs {
                    if let (Some(idx), Some(v)) = (pt.attr("idx"), pt.find("v"))
                        && let Ok(idx) = idx.parse()
                    {
                        cats.push((idx, v.text().trim().to_string()));
                    }
                }
            }
            if let Some(val) = ser.find("val") {
                let mut refs = Vec::new();
                val.find_all("pt", &mut refs);
                for pt in refs {
                    if let (Some(idx), Some(v)) = (pt.attr("idx"), pt.find("v"))
                        && let Ok(idx) = idx.parse()
                    {
                        vals.push((idx, v.text().trim().to_string()));
                    }
                }
            }
            cats.sort_by_key(|(i, _)| *i);
            vals.sort_by_key(|(i, _)| *i);
            out.push((
                name,
                cats.into_iter().map(|(_, v)| v).collect(),
                vals.into_iter().map(|(_, v)| v).collect(),
            ));
        }
    }
    out
}

/// `chart`'s title, the run text under `c:title`.
fn title(chart: &Element) -> String {
    let mut runs = Vec::new();
    if let Some(title) = chart.find("title") {
        title.find_all("t", &mut runs);
    }
    runs.iter()
        .map(|t| t.text())
        .collect::<Vec<_>>()
        .join("")
        .trim()
        .to_string()
}

/// A category or series name Mermaid quotes: `'` for `"`, one line.
fn quoted(text: &str) -> String {
    text.replace('"', "'").replace('\n', " ")
}

/// `chartSpace`'s XML as Mermaid — `pie` for a pie chart, `xychart-beta`
/// for bar and line — `None` when the part is no chart these say. Series
/// that share no categories, or hold none, cannot be an `xychart-beta`.
pub(crate) fn chart(xml: &str) -> Option<String> {
    let root = parse_xml(xml.as_bytes()).ok()?;
    if !root.is("chartSpace") {
        return None;
    }
    let chart = root.find("chart")?;
    let title = title(chart);
    let pies = series(chart, "pieChart");
    if !pies.is_empty() {
        let mut out = String::from("pie");
        if !title.is_empty() {
            let _ = write!(out, " title {}", quoted(&title));
        }
        for (_, cats, vals) in &pies {
            if cats.is_empty() || cats.len() != vals.len() {
                return None;
            }
            for (cat, val) in cats.iter().zip(vals) {
                let _ = write!(out, "\n    \"{}\" : {}", quoted(cat), val);
            }
        }
        return Some(out);
    }
    let bars = series(chart, "barChart");
    let lines = series(chart, "lineChart");
    if bars.is_empty() && lines.is_empty() {
        return None;
    }
    let all = bars.iter().chain(&lines).collect::<Vec<_>>();
    let cats = &all.first()?.1;
    if cats.is_empty()
        || all
            .iter()
            .any(|(_, c, v)| *c != *cats || v.len() != cats.len())
    {
        return None;
    }
    let mut out = String::from("xychart-beta");
    if !title.is_empty() {
        let _ = write!(out, "\n    title \"{}\"", quoted(&title));
    }
    let _ = write!(
        out,
        "\n    x-axis [{}]",
        cats.iter()
            .map(|c| format!("\"{}\"", quoted(c)))
            .collect::<Vec<_>>()
            .join(", ")
    );
    for (_, _, vals) in &bars {
        let _ = write!(out, "\n    bar [{}]", vals.join(", "));
    }
    for (_, _, vals) in &lines {
        let _ = write!(out, "\n    line [{}]", vals.join(", "));
    }
    Some(out)
}

/// A Mermaid fence's text, parsed: its chart's series in order — `pie`
/// slices as one series, `xychart-beta` `bar`/`line` lines as theirs —
/// with the categories and title it names.
struct Mermaid {
    title: Option<String>,
    cats: Vec<String>,
    /// `(chart kind, values)`: `pieChart`, `barChart` or `lineChart`.
    sers: Vec<(String, Vec<String>)>,
}

/// `text` as a [`Mermaid`]: `pie` or `xychart-beta`, `None` for anything
/// else or a line the two cannot say.
fn parse(text: &str) -> Option<Mermaid> {
    let mut lines = text.trim().lines().map(str::trim);
    match lines.next()? {
        head if head == "pie" || head.starts_with("pie ") => {
            let title = head
                .trim_start_matches("pie")
                .trim()
                .strip_prefix("title")
                .map(str::trim)
                .filter(|t| !t.is_empty())
                .map(str::to_string);
            let mut sers = Vec::new();
            for line in lines {
                if line.is_empty() {
                    continue;
                }
                let (name, value) = line.rsplit_once(':')?;
                sers.push((
                    name.trim().trim_matches('"').replace('\'', "\""),
                    value.trim().to_string(),
                ));
            }
            if sers.is_empty() {
                return None;
            }
            Some(Mermaid {
                title,
                cats: sers.iter().map(|(n, _)| n.clone()).collect(),
                sers: vec![(
                    "pieChart".to_string(),
                    sers.into_iter().map(|(_, v)| v).collect(),
                )],
            })
        }
        "xychart-beta" => {
            let mut mermaid = Mermaid {
                title: None,
                cats: Vec::new(),
                sers: Vec::new(),
            };
            for line in lines {
                if line.is_empty() {
                    continue;
                }
                if let Some(rest) = line.strip_prefix("title ") {
                    mermaid.title = Some(rest.trim().trim_matches('"').replace('\'', "\""));
                } else if let Some(rest) = line.strip_prefix("x-axis ") {
                    mermaid.cats = list(rest)?;
                } else if let Some(rest) = line.strip_prefix("bar ") {
                    mermaid.sers.push(("barChart".to_string(), list(rest)?));
                } else if let Some(rest) = line.strip_prefix("line ") {
                    mermaid.sers.push(("lineChart".to_string(), list(rest)?));
                } else if line.starts_with("y-axis") {
                    // The axis labels stay the part's own.
                } else {
                    return None;
                }
            }
            (!mermaid.cats.is_empty()
                && !mermaid.sers.is_empty()
                && mermaid
                    .sers
                    .iter()
                    .all(|(_, v)| v.len() == mermaid.cats.len()))
            .then_some(mermaid)
        }
        _ => None,
    }
}

/// A Mermaid `[a, "b", …]` list's items; a comma inside quotes is text.
fn list(text: &str) -> Option<Vec<String>> {
    let inner = text.trim().strip_prefix('[')?.strip_suffix(']')?;
    let mut items = Vec::new();
    let mut item = String::new();
    let mut quoted = false;
    for c in inner.chars() {
        match c {
            '"' => quoted = !quoted,
            ',' if !quoted => {
                items.push(item.trim().trim_matches('"').to_string());
                item.clear();
            }
            _ => item.push(c),
        }
    }
    items.push(item.trim().trim_matches('"').to_string());
    (!items.is_empty() && !items.iter().all(String::is_empty)).then_some(items)
}

/// Every descendant of `root` with this local name, mutably.
fn find_all_mut<'a>(element: &'a mut Element, name: &str, out: &mut Vec<&'a mut Element>) {
    for child in &mut element.children {
        if let Node::Element(inner) = child {
            if inner.is(name) {
                out.push(inner);
            } else {
                find_all_mut(inner, name, out);
            }
        }
    }
}

/// First descendant of `element` with this local name, mutably.
fn find_mut<'a>(element: &'a mut Element, name: &str) -> Option<&'a mut Element> {
    for child in &mut element.children {
        if let Node::Element(inner) = child {
            if inner.is(name) {
                return Some(inner);
            }
            if let Some(found) = find_mut(inner, name) {
                return Some(found);
            }
        }
    }
    None
}

/// A `c:pt` element holding `v` at `idx`, with `prefix` the chart's own.
fn pt(prefix: &str, idx: usize, v: &str) -> Element {
    Element {
        name: format!("{prefix}pt"),
        attrs: vec![("idx".to_string(), idx.to_string())],
        children: vec![Node::Element(Element {
            name: format!("{prefix}v"),
            attrs: Vec::new(),
            children: vec![Node::Text(v.to_string())],
        })],
    }
}

/// `cache` (`strCache`, `numCache`, `strLit`, `numLit`) with `values` as
/// its points: the `pt`s replaced, `ptCount` renumbered, anything else —
/// a `formatCode` — kept.
fn set_cache(cache: &mut Element, values: &[String]) {
    let prefix = cache
        .name
        .split_once(':')
        .map(|(p, _)| format!("{p}:"))
        .unwrap_or_default();
    let mut pts = Vec::new();
    for child in cache.children.drain(..) {
        match child {
            Node::Element(e) if e.is("pt") => {}
            Node::Element(mut e) => {
                if e.is("ptCount") {
                    for (name, value) in &mut e.attrs {
                        if local_name(name) == "val" {
                            *value = values.len().to_string();
                        }
                    }
                }
                pts.push(Node::Element(e));
            }
            text => pts.push(text),
        }
    }
    pts.extend(
        values
            .iter()
            .enumerate()
            .map(|(i, v)| Node::Element(pt(&prefix, i, v))),
    );
    cache.children = pts;
}

fn local_name(name: &str) -> &str {
    name.rsplit(':').next().unwrap_or(name)
}

/// `ser`'s `c:cat` and `c:val` points replaced by `cats`/`vals`; `false`
/// when the series has no cache to write.
/// `ser`'s `c:cat` and `c:val` points replaced by `cats`/`vals`; `false`
/// when the series has no cache to write.
fn set_ser(ser: &mut Element, cats: &[String], vals: &[String]) -> bool {
    let mut ok = false;
    if let Some(cat) = find_mut(ser, "cat")
        && let Some(cache) = first_mut(cat, &["strCache", "strLit"])
    {
        set_cache(cache, cats);
        ok = true;
    }
    if let Some(val) = find_mut(ser, "val")
        && let Some(cache) = first_mut(val, &["numCache", "numLit"])
    {
        set_cache(cache, vals);
        ok = true;
    }
    ok
}

/// The first `names` descendant of `element`, mutably.
fn first_mut<'a>(element: &'a mut Element, names: &[&str]) -> Option<&'a mut Element> {
    if let Some(name) = names
        .iter()
        .find(|name| find_mut(element, name).is_some())
        .copied()
    {
        return find_mut(element, name);
    }
    None
}

/// `chart`'s `c:ser` elements.
fn sers_of(chart: &mut Element) -> Vec<&mut Element> {
    let mut out = Vec::new();
    find_all_mut(chart, "ser", &mut out);
    out
}

/// The edited fence's text applied to `chartSpace` XML: its series'
/// categories and values, its title, rewritten in place so everything
/// else — styling, axes, embedded data, rels — stays. `None` when the
/// Mermaid's shape and the chart's disagree (a `pie` of two series, an
/// `xychart-beta` against a part with no bar or line chart).
pub(crate) fn patch(xml: &str, text: &str) -> Option<String> {
    let mermaid = parse(text)?;
    let mut root = parse_xml(xml.as_bytes()).ok()?;
    if !root.is("chartSpace") {
        return None;
    }
    let chart = find_mut(&mut root, "chart")?;
    // The kinds the Mermaid names must be the kinds the part holds, in
    // the same count; anything else is a different chart, not an edit.
    for kind in ["pieChart", "barChart", "lineChart"] {
        let wants: Vec<&Vec<String>> = mermaid
            .sers
            .iter()
            .filter(|(k, _)| k == kind)
            .map(|(_, v)| v)
            .collect();
        let mut charts = Vec::new();
        find_all_mut(chart, kind, &mut charts);
        let mut sers = Vec::new();
        for chart_el in &mut charts {
            sers.extend(sers_of(chart_el));
        }
        if wants.len() != sers.len() {
            return None;
        }
        for (ser, vals) in sers.into_iter().zip(wants) {
            if !set_ser(ser, &mermaid.cats, vals) {
                return None;
            }
        }
    }
    if let Some(title) = &mermaid.title
        && let Some(t) = find_mut(chart, "title").and_then(|t| find_mut(t, "t"))
        && let Some(Node::Text(rich)) = t.children.first_mut()
    {
        *rich = title.clone();
    }
    Some(super::drawing::serialize(&root))
}

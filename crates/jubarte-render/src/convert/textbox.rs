// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Text boxes, drawing groups, custom geometry and diagrams.

use super::*;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
pub(super) fn collect_textboxes(
    src: Option<(&PartFs, &str)>,
    dom: &Dom,
    para: NodeId,
    base: &RunStyle,
    theme: &ThemeFonts,
) -> Vec<LaidTextBox> {
    collect_textboxes_styled(src, dom, para, base, theme, None, None)
}

/// `collect_textboxes` with the document's styles, so each text box
/// paragraph keeps its own style, spacing and jc.
pub(super) fn collect_textboxes_styled(
    src: Option<(&PartFs, &str)>,
    dom: &Dom,
    para: NodeId,
    base: &RunStyle,
    theme: &ThemeFonts,
    sheet: Option<&StyleSheet>,
    numbering: Option<&Numbering>,
) -> Vec<LaidTextBox> {
    let mut out = Vec::new();
    let shapes = shape_roots(dom, para);
    // image_out_of_folder: wrapSquare page-anchor PNG is logo-only.
    // Word Quartz paints the sibling VML txbx as overlay at the v:shape
    // style origin. Flowing it shoved Quantum (ITT 41); skipping it
    // dropped "Subscribe to DeepL Pro". Overlay when the VML is
    // position:absolute; still skip unpositioned pict chrome.
    let wrap_square_picture = shapes.iter().any(|&shape| {
        drawing_has_blip(dom, shape) && first_named_any(dom, shape, "wrapSquare").is_some()
    });
    for shape in shapes {
        if wrap_square_picture
            && dom.name_is(shape, &W::pict())
            && vml_absolute_slot(dom, shape).is_none()
        {
            continue;
        }
        // A picture is collect_images' to lay out: its a:ln outline is not a
        // second, empty stroked box (000f5278's QR code ran 188pt long).
        if dom.name_is(shape, &W::drawing())
            && graphic_data_uri_contains(dom, shape, "drawingml/2006/picture")
            && drawing_group(dom, shape).is_none()
        {
            continue;
        }
        let txbx = first_named_any(dom, shape, "txbxContent").or_else(|| {
            dom.descendants(shape, Some(&W::txbx_content()))
                .into_iter()
                .next()
        });
        let mut runs = txbx
            .map(|n| collect_runs(dom, n, base, theme))
            .unwrap_or_default();
        let mut text_dx = txbx.map(|n| first_para_content_dx(dom, n)).unwrap_or(0.0);
        let mut text_dy = txbx
            .map(|n| first_para_spacing_before(dom, n))
            .unwrap_or(0.0);
        let (paras, tables) = match (sheet, txbx) {
            (Some(sheet), Some(n)) if txbx_lays_out_paragraphs(dom, shape, n) => {
                txbx_paragraphs(dom, n, sheet, theme, numbering, src)
            }
            _ => (Vec::new(), Vec::new()),
        };
        if runs.iter().all(|r| r.text.trim().is_empty()) {
            let (linked, dx, dy) = linked_txbx_content(src, dom, shape, base, theme);
            runs = linked;
            if text_dx <= 0.0 {
                text_dx = dx;
            }
            if text_dy <= 0.0 {
                text_dy = dy;
            }
        }
        let object = drawing_is_chart_or_diagram(dom, shape);
        let diagram = graphic_data_uri_contains(dom, shape, "/diagram");
        if runs.iter().all(|r| r.text.trim().is_empty()) && diagram {
            runs = diagram_label_runs(src, dom, shape, base);
        }
        let empty = runs.iter().all(|r| r.text.trim().is_empty());
        let vml_slot = vml_absolute_slot(dom, shape);
        let vml = dom.name_is(shape, &W::pict());
        // An inline VML shape is sized by its style, not the DrawingML
        // default box (069252c3's 21x9pt logo letters ran 200x120).
        let vml_sized = vml
            && descendants_local(dom, shape, "shape").into_iter().any(|v| {
                attr_any(dom, v, "style").is_some_and(|st| {
                    vml_style_pt(st, "width").is_some() && vml_style_pt(st, "height").is_some()
                })
            });
        let (w, h) = if vml_slot.is_some() || vml_sized {
            vml_extent_pt(dom, shape)
        } else {
            drawing_extent_pt(dom, shape)
        };
        // wrapNone decorations (connectors, cover overlays) score worse when
        // stroked. Inline / wrapTopAndBottom frames with a real extent still
        // consume flow (Strict01 Rectangle 3 is 402×167 with no txbx).
        // Bare `w:pict`/`w:object` have no extent and must not invent 200×120
        // (sd_2517 jumped 94→135 pages when they reserved default boxes).
        let slot = if object {
            ImageSlot::Flow
        } else if let Some(vml) = vml_slot {
            vml
        } else {
            drawing_slot(dom, shape)
        };
        let chart = object
            .then(|| src.and_then(|(pkg, main)| load_chart(pkg, main, dom, shape, theme)))
            .flatten();
        let mut fill = shape_fill_color(dom, shape, theme);
        let line = shape_line_color(dom, shape, theme);
        let explicit_ln = descendants_local(dom, shape, "ln")
            .into_iter()
            .any(|ln| !descendants_local(dom, ln, "solidFill").is_empty());
        let geom = shape_geom(dom, shape);
        if matches!(
            geom,
            ShapeGeom::BentConnector | ShapeGeom::CurvedConnector | ShapeGeom::Line
        ) {
            fill = fill.or(line);
        }
        let (behind, z) = drawing_z(dom, shape);
        let (flip_h, flip_v) = shape_flip(dom, shape);
        let tail_end = shape_has_tail_end(dom, shape);
        let text_anchor = shape_text_anchor(dom, shape);
        let group = group_children(dom, shape, &GroupText { theme, base, sheet });
        // A group paints its shapes, not one box from its first child.
        let custom = if group.is_empty() {
            cust_geom(dom, shape).map(std::rc::Rc::new)
        } else {
            None
        };
        if !group.is_empty() {
            fill = None;
        }
        let line = if group.is_empty() { line } else { None };
        if chart.is_none() && !group.is_empty() {
            // An inline group's pictures already hold its line (the image
            // path); its shapes paint over the paragraph's start instead of
            // taking a second box below it (en a 5fb9cedf's logo dot).
            let slot = if matches!(slot, ImageSlot::Flow) && !group_pictures(dom, shape).is_empty()
            {
                ImageSlot::Float {
                    align: Align::Left,
                    page_x: None,
                    page_y: None,
                    col_x: Some(0.0),
                    col_in_column: false,
                    para_y: Some(0.0),
                    pct_x: None,
                    pct_y: None,
                    pct_w: None,
                    pct_h: None,
                    v_align: Align::Left,
                    wrap_square: false,
                    wrap_top_bottom: false,
                    wrap_polygon: false,
                    poly_bottom: 1.0,
                    dist_l: 0.0,
                    dist_r: 0.0,
                    dist_t: 0.0,
                    dist_b: 0.0,
                    effect_b: 0.0,
                    h_rel: RelFrame::Column,
                    v_rel: RelFrame::Paragraph,
                    v_off: None,
                }
            } else {
                slot
            };
            let mut boxed = group_box(w, h, slot, geom, behind, z, group);
            boxed.raise = run_raise_pt(dom, shape);
            // A canvas paints its own background and outline under its
            // shapes (00019a41's pool, isla's Venn frame).
            if let Some(wpc) = descendants_local(dom, shape, "wpc").into_iter().next() {
                let part = |name: &str| {
                    (0..dom.child_count(wpc))
                        .map(|i| dom.child_at(wpc, i))
                        .find(|c| local_name_is(dom, *c, name))
                };
                boxed.geom = ShapeGeom::Box;
                boxed.fill = part("bg").and_then(|bg| shape_fill_color(dom, bg, theme));
                if let Some(whole) = part("whole") {
                    boxed.line = shape_line_color(dom, whole, theme);
                    boxed.stroke = boxed.line.is_some() && !shape_ln_is_nofill(dom, whole);
                    boxed.line_width = shape_line_width(dom, whole, theme);
                }
            }
            out.push(boxed);
            continue;
        }
        if empty && chart.is_none() {
            if fill.is_some() || line.is_some() {
                // Preset polygons stroke their own closed outline in the
                // line colour, filled or not. Box keeps its tuned 4-edge
                // rules (KEEP 591), connectors their own stroke. A custom
                // path strokes itself like a polygon.
                let polygon = geom_is_preset_polygon(geom) || custom.is_some();
                let box_line = if polygon || (matches!(geom, ShapeGeom::Box) && fill.is_some()) {
                    line
                } else {
                    None
                };
                out.push(LaidTextBox {
                    w,
                    h,
                    runs: Vec::new(),
                    slot,
                    chart: None,
                    stroke: line.is_some()
                        && (fill.is_none() || polygon || matches!(geom, ShapeGeom::Box)),
                    fill,
                    line: box_line,
                    line_width: shape_line_width(dom, shape, theme),
                    geom,
                    reserve_only: false,
                    behind,
                    z,
                    flip_h,
                    flip_v,
                    tail_end,
                    diag_shapes: Vec::new(),
                    text_dx: 0.0,
                    text_dy: 0.0,
                    text_anchor,
                    adj: preset_adjustments(dom, shape),
                    prst: shape_prst(dom, shape),
                    paras: Vec::new(),
                    tables: Vec::new(),
                    insets: TXBX_INSETS,
                    custom: custom.clone(),
                    group: Vec::new(),
                    chrome_para_top: 0.0,
                    fit_width: false,
                    fit_height: false,
                    raise: run_raise_pt(dom, shape),
                    frame: false,
                    vml,
                });
                continue;
            }
            // Strict01 Rectangle 3: inline a:noFill 402×167. Word keeps the
            // hole above the chart so wrapNone fills sit in it, not on the
            // plot, and the WMF clipart lands on page 3.
            if !object
                && !diagram
                && !drawing_has_blip(dom, shape)
                && matches!(slot, ImageSlot::Flow)
                && h > 16.0
                && first_named_any(dom, shape, "extent").is_some()
            {
                out.push(LaidTextBox {
                    w,
                    h,
                    runs: Vec::new(),
                    slot,
                    chart: None,
                    stroke: false,
                    fill: None,
                    line: None,
                    line_width: 1.0,
                    geom,
                    reserve_only: true,
                    behind,
                    z,
                    flip_h,
                    flip_v,
                    tail_end,
                    diag_shapes: Vec::new(),
                    text_dx: 0.0,
                    text_dy: 0.0,
                    text_anchor,
                    adj: preset_adjustments(dom, shape),
                    prst: shape_prst(dom, shape),
                    paras: Vec::new(),
                    tables: Vec::new(),
                    insets: TXBX_INSETS,
                    custom: custom.clone(),
                    group: Vec::new(),
                    chrome_para_top: 0.0,
                    fit_width: false,
                    fit_height: false,
                    raise: run_raise_pt(dom, shape),
                    frame: false,
                    vml,
                });
                continue;
            }
            // A diagram without labels still paints its drawing's shapes.
            let drawn = diagram
                && src
                    .and_then(|(pkg, main)| load_diag_shapes(pkg, main, dom, shape, theme))
                    .is_some_and(|v| !v.is_empty());
            if !object || (diagram && !drawn) {
                continue;
            }
        }
        let diag_shapes = if diagram {
            src.and_then(|(pkg, main)| load_diag_shapes(pkg, main, dom, shape, theme))
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        let vml_unstroked = descendants_local(dom, shape, "shape").into_iter().any(|s| {
            attr_any(dom, s, "stroked").is_some_and(|v| v == "f" || v.eq_ignore_ascii_case("false"))
        });
        let wps = !descendants_local(dom, shape, "wsp").is_empty();
        out.push(LaidTextBox {
            w,
            h,
            runs,
            slot,
            chart,
            // WPS has no implicit outline: Word 16.115 paints the unstyled
            // 1975-textbox without one (#1979). mcdoc's outline comes from
            // its explicit a:ln, not a rendering default. Preserve the VML
            // default and the separate ChartSpace frame (mini 568).
            stroke: if wps {
                !shape_ln_is_nofill(dom, shape)
                    && (!descendants_local(dom, shape, "ln").is_empty()
                        || ln_ref_idx(dom, shape).is_some_and(|idx| idx > 0))
            } else {
                !(diagram
                    || (vml_slot.is_some() && vml_unstroked)
                    || shape_ln_is_nofill(dom, shape))
                    && (vml_slot.is_some() || !(fill.is_some() && line.is_none()))
            },
            fill,
            // WPS style references supply their colour just like explicit
            // a:ln fills (5fb9cedf's form boxes use 3pt accent blue).
            line: line.filter(|_| wps || geom_is_preset_polygon(geom) || explicit_ln),
            line_width: if wps || explicit_ln {
                shape_line_width(dom, shape, theme)
            } else {
                1.0
            },
            geom,
            reserve_only: false,
            behind,
            z,
            flip_h,
            flip_v,
            tail_end,
            diag_shapes,
            text_dx,
            text_dy,
            text_anchor,
            adj: preset_adjustments(dom, shape),
            prst: shape_prst(dom, shape),
            paras,
            tables,
            insets: textbox_text_insets(dom, shape, theme),
            custom,
            group,
            chrome_para_top: 0.0,
            fit_width: bodypr_fits_width(dom, shape),
            fit_height: descendants_local(dom, shape, "bodyPr")
                .first()
                .is_some_and(|&body| !descendants_local(dom, body, "spAutoFit").is_empty()),
            raise: run_raise_pt(dom, shape),
            frame: false,
            vml,
        });
    }
    // WrapNone accent fills on the same paragraph as an inline chart
    // paint on top of the plot (Strict01 page 1) and wreck SSIM. The
    // theme-fill test has no chart and still paints.
    if out.iter().any(|b| b.chart.is_some()) {
        out.retain(|b| {
            b.chart.is_some()
                || !b.runs.iter().all(|r| r.text.trim().is_empty())
                || matches!(b.slot, ImageSlot::Flow)
        });
    }
    // behindDoc first, then relativeHeight so the cover white paper (468)
    // stays under the dark abstract header (467).
    out.sort_by_key(|b| (!b.behind, b.z));
    out
}

/// A group's own box: no geometry of its own, its shapes in `group`.
pub(super) fn group_box(
    w: f32,
    h: f32,
    slot: ImageSlot,
    geom: ShapeGeom,
    behind: bool,
    z: u32,
    group: Vec<GroupChild>,
) -> LaidTextBox {
    LaidTextBox {
        w,
        h,
        runs: Vec::new(),
        slot,
        chart: None,
        stroke: false,
        fill: None,
        line: None,
        line_width: 1.0,
        geom,
        reserve_only: false,
        behind,
        z,
        flip_h: false,
        flip_v: false,
        tail_end: false,
        diag_shapes: Vec::new(),
        text_dx: 0.0,
        text_dy: 0.0,
        text_anchor: TextAnchor::Top,
        adj: Vec::new(),
        prst: String::new(),
        paras: Vec::new(),
        tables: Vec::new(),
        insets: TXBX_INSETS,
        custom: None,
        group,
        chrome_para_top: 0.0,
        fit_width: false,
        fit_height: false,
        raise: 0.0,
        frame: false,
        vml: false,
    }
}

/// The drawing's top group: a `wpg:wgp`, or a `wpc:wpc` canvas with its
/// `wp:extent` (EMU), whose children are placed like a group's.
pub(super) fn drawing_group(dom: &Dom, drawing: NodeId) -> Option<(NodeId, Option<[f64; 2]>)> {
    if let Some(wgp) = descendants_local(dom, drawing, "wgp").into_iter().next() {
        return Some((wgp, None));
    }
    let wpc = descendants_local(dom, drawing, "wpc").into_iter().next()?;
    let ext = descendants_local(dom, drawing, "extent")
        .into_iter()
        .next()?;
    let num = |k: &str| attr_any(dom, ext, k).and_then(parse_finite_f64);
    Some((wpc, Some([num("cx")?, num("cy")?])))
}

/// The shapes of the drawing's `wpg:wgp` group (nested groups flattened),
/// each placed through the group's `chOff`/`chExt` child space.
pub(super) fn group_children(dom: &Dom, shape: NodeId, text: &GroupText) -> Vec<GroupChild> {
    let mut out = Vec::new();
    if let Some((wgp, canvas)) = drawing_group(dom, shape) {
        collect_group(
            dom,
            wgp,
            GroupFrame {
                frac: [0.0, 0.0, 1.0, 1.0],
                fill: None,
                canvas,
            },
            text.theme,
            Some(text),
            &mut out,
            &mut Vec::new(),
        );
    }
    out
}

/// The pictures of the drawing's `wpg:wgp` group: each `pic:pic` with its
/// box as fractions of the group's box (x, y from the top-left, w, h).
pub(super) fn group_pictures(dom: &Dom, drawing: NodeId) -> Vec<([f32; 4], NodeId)> {
    let mut pics = Vec::new();
    if let Some((wgp, canvas)) = drawing_group(dom, drawing) {
        let theme = ThemeFonts::default();
        collect_group(
            dom,
            wgp,
            GroupFrame {
                frac: [0.0, 0.0, 1.0, 1.0],
                fill: None,
                canvas,
            },
            &theme,
            None,
            &mut Vec::new(),
            &mut pics,
        );
    }
    pics
}

/// `slot` moved `dx` right and `dy` down, for a group's picture placed
/// inside the group's box. Offsets that are fractions of the page or
/// alignments cannot move and stay.
pub(super) fn shift_slot(mut slot: ImageSlot, dx: f32, dy: f32) -> ImageSlot {
    if let ImageSlot::Float {
        page_x,
        col_x,
        page_y,
        para_y,
        v_off,
        ..
    } = &mut slot
    {
        if let Some(x) = page_x.as_mut().or(col_x.as_mut()) {
            *x += dx;
        }
        if let Some(y) = page_y.as_mut().or(para_y.as_mut()).or(v_off.as_mut()) {
            *y += dy;
        }
    }
    slot
}

/// What a group's text boxes need to lay their text out.
pub(super) struct GroupText<'a> {
    pub(super) theme: &'a ThemeFonts,
    pub(super) base: &'a RunStyle,
    pub(super) sheet: Option<&'a StyleSheet>,
}

/// `(off x, off y, ext cx, ext cy)` of an `a:xfrm`, EMU.
fn xfrm_box(dom: &Dom, xfrm: NodeId) -> Option<[f64; 4]> {
    let num = |node: Option<NodeId>, key: &str| {
        node.and_then(|n| attr_any(dom, n, key))
            .and_then(parse_finite_f64)
    };
    let off = descendants_local(dom, xfrm, "off").into_iter().next();
    let ext = descendants_local(dom, xfrm, "ext").into_iter().next();
    Some([
        num(off, "x").unwrap_or(0.0),
        num(off, "y").unwrap_or(0.0),
        num(ext, "cx")?,
        num(ext, "cy")?,
    ])
}

/// A group's place in the outer group's box and the fill its `grpFill`
/// children inherit from the groups around it.
#[derive(Clone, Copy)]
pub(super) struct GroupFrame {
    pub(super) frac: [f32; 4],
    pub(super) fill: Option<[f32; 3]>,
    /// A `wpc:wpc` drawing canvas's extent (EMU): it has no `grpSpPr`, its
    /// children sit at EMU offsets from its top-left.
    pub(super) canvas: Option<[f64; 2]>,
}

pub(super) fn collect_group(
    dom: &Dom,
    grp: NodeId,
    frame: GroupFrame,
    theme: &ThemeFonts,
    text: Option<&GroupText>,
    out: &mut Vec<GroupChild>,
    pics: &mut Vec<([f32; 4], NodeId)>,
) {
    let GroupFrame {
        frac,
        fill: parent_fill,
        canvas,
    } = frame;
    let children: Vec<NodeId> = (0..dom.child_count(grp))
        .map(|i| dom.child_at(grp, i))
        .collect();
    let grp_pr = children
        .iter()
        .copied()
        .find(|c| local_name_is(dom, *c, "grpSpPr"));
    let xfrm = grp_pr.and_then(|pr| descendants_local(dom, pr, "xfrm").into_iter().next());
    if xfrm.is_none() && canvas.is_none() {
        return;
    }
    // The group's own fill, which `a:grpFill` children take (003329b5's
    // light-green label panels), else the enclosing group's.
    let group_fill = grp_pr
        .and_then(|pr| {
            (0..dom.child_count(pr))
                .map(|i| dom.child_at(pr, i))
                .find(|c| local_name_is(dom, *c, "solidFill"))
        })
        .and_then(|f| scheme_color(dom, f, theme))
        .or(parent_fill);
    let num = |local: &str, key: &str| {
        xfrm.and_then(|x| descendants_local(dom, x, local).into_iter().next())
            .and_then(|n| attr_any(dom, n, key))
            .and_then(parse_finite_f64)
    };
    let Some(ext) = xfrm
        .and_then(|x| xfrm_box(dom, x))
        .or_else(|| canvas.map(|[cx, cy]| [0.0, 0.0, cx, cy]))
    else {
        return;
    };
    let (cox, coy) = (
        num("chOff", "x").unwrap_or(0.0),
        num("chOff", "y").unwrap_or(0.0),
    );
    let cex = num("chExt", "cx").filter(|v| *v > 0.0).unwrap_or(ext[2]);
    let cey = num("chExt", "cy").filter(|v| *v > 0.0).unwrap_or(ext[3]);
    if cex <= 0.0 || cey <= 0.0 {
        return;
    }
    let place = |b: [f64; 4]| -> [f32; 4] {
        // A ratio that does not fit in f32 is no offset: the child stays put.
        let px = |v: f64| narrow_finite_f32(v).unwrap_or(0.0);
        let fx = px((b[0] - cox) / cex);
        let fy = px((b[1] - coy) / cey);
        let fw = px(b[2] / cex);
        let fh = px(b[3] / cey);
        [
            frac[0] + fx * frac[2],
            frac[1] + fy * frac[3],
            fw * frac[2],
            fh * frac[3],
        ]
    };
    for child in children {
        if local_name_is(dom, child, "grpSp") || local_name_is(dom, child, "wgp") {
            let sub = (0..dom.child_count(child))
                .map(|i| dom.child_at(child, i))
                .find(|c| local_name_is(dom, *c, "grpSpPr"))
                .and_then(|pr| descendants_local(dom, pr, "xfrm").into_iter().next())
                .and_then(|x| xfrm_box(dom, x));
            if let Some(b) = sub {
                let frame = GroupFrame {
                    frac: place(b),
                    fill: group_fill,
                    canvas: None,
                };
                collect_group(dom, child, frame, theme, text, out, pics);
            }
        } else if local_name_is(dom, child, "wsp") {
            let Some(b) = descendants_local(dom, child, "xfrm")
                .into_iter()
                .next()
                .and_then(|x| xfrm_box(dom, x))
            else {
                continue;
            };
            let geom = shape_geom(dom, child);
            let takes_group_fill = descendants_local(dom, child, "spPr")
                .into_iter()
                .next()
                .is_some_and(|sp| {
                    (0..dom.child_count(sp))
                        .any(|i| local_name_is(dom, dom.child_at(sp, i), "grpFill"))
                });
            let fill = if takes_group_fill {
                group_fill
            } else {
                shape_fill_color(dom, child, theme)
            };
            let line = shape_line_color(dom, child, theme);
            let custom = cust_geom(dom, child).map(std::rc::Rc::new);
            let polygon = geom_is_preset_polygon(geom) || custom.is_some();
            let (flip_h, flip_v) = shape_flip(dom, child);
            let mut shape = group_box(
                narrow_finite_f32(b[2] / 12700.0).unwrap_or(0.0),
                narrow_finite_f32(b[3] / 12700.0).unwrap_or(0.0),
                ImageSlot::Flow,
                geom,
                false,
                0,
                Vec::new(),
            );
            shape.fill = fill;
            shape.line = line.filter(|_| polygon || matches!(geom, ShapeGeom::Box));
            shape.stroke = line.is_some() && !shape_ln_is_nofill(dom, child);
            shape.line_width = shape_line_width(dom, child, theme);
            shape.custom = custom;
            shape.adj = preset_adjustments(dom, child);
            shape.prst = shape_prst(dom, child);
            shape.flip_h = flip_h;
            shape.flip_v = flip_v;
            // A grouped text box carries its own text (the Achensee
            // redline header's banner lines).
            if let Some(text) = text
                && let Some(txbx) = first_named_any(dom, child, "txbxContent")
            {
                shape.runs = collect_runs(dom, txbx, text.base, theme);
                shape.text_dx = first_para_content_dx(dom, txbx);
                shape.text_dy = first_para_spacing_before(dom, txbx);
                if let Some(sheet) = text.sheet
                    && txbx_lays_out_paragraphs(dom, child, txbx)
                {
                    (shape.paras, shape.tables) =
                        txbx_paragraphs(dom, txbx, sheet, theme, None, None);
                }
                shape.insets = textbox_text_insets(dom, child, theme);
                shape.text_anchor = shape_text_anchor(dom, child);
            }
            out.push(GroupChild {
                frac: place(b),
                shape,
            });
        } else if local_name_is(dom, child, "pic")
            && let Some(b) = descendants_local(dom, child, "xfrm")
                .into_iter()
                .next()
                .and_then(|x| xfrm_box(dom, x))
        {
            pics.push((place(b), child));
        }
    }
}

/// `a:custGeom` as guides and paths (ECMA-376 §20.1.9.8): the evaluator
/// flattens it like a preset.
pub(super) fn cust_geom(dom: &Dom, shape: NodeId) -> Option<preset_geom::CustomGeom> {
    use preset_geom::{CustomGeom, CustomPath, Fill, OwnedCmd};
    let cg = descendants_local(dom, shape, "custGeom")
        .into_iter()
        .next()?;
    let guides = |list: &str| -> Vec<(String, String)> {
        descendants_local(dom, cg, list)
            .first()
            .map(|l| {
                descendants_local(dom, *l, "gd")
                    .into_iter()
                    .filter_map(|gd| {
                        Some((
                            attr_any(dom, gd, "name")?.to_string(),
                            attr_any(dom, gd, "fmla")?.to_string(),
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default()
    };
    let pts = |node: NodeId| -> Vec<String> {
        descendants_local(dom, node, "pt")
            .into_iter()
            .flat_map(|pt| {
                [
                    attr_any(dom, pt, "x").unwrap_or("0").to_string(),
                    attr_any(dom, pt, "y").unwrap_or("0").to_string(),
                ]
            })
            .collect()
    };
    let list = descendants_local(dom, cg, "pathLst").into_iter().next()?;
    // A path without w/h draws in the shape's own EMU extent.
    let ext = descendants_local(dom, shape, "ext")
        .into_iter()
        .find(|e| attr_any(dom, *e, "cx").is_some());
    let extent = |key: &str| {
        ext.and_then(|e| attr_any(dom, e, key))
            .and_then(parse_finite_f64)
            .unwrap_or(0.0)
    };
    let mut paths = Vec::new();
    for path in (0..dom.child_count(list))
        .map(|i| dom.child_at(list, i))
        .filter(|n| local_name_is(dom, *n, "path"))
    {
        let num = |key: &str| {
            attr_any(dom, path, key)
                .and_then(parse_finite_f64)
                .unwrap_or(0.0)
        };
        let fill = match attr_any(dom, path, "fill").unwrap_or("norm") {
            "none" => Fill::None,
            "lighten" => Fill::Lighten,
            "lightenLess" => Fill::LightenLess,
            "darken" => Fill::Darken,
            "darkenLess" => Fill::DarkenLess,
            _ => Fill::Norm,
        };
        let stroke = attr_any(dom, path, "stroke").is_none_or(|v| v != "0" && v != "false");
        let mut cmds = Vec::new();
        for i in 0..dom.child_count(path) {
            let node = dom.child_at(path, i);
            let p = pts(node);
            let cmd = if local_name_is(dom, node, "moveTo") && p.len() >= 2 {
                OwnedCmd::M(p[0].clone(), p[1].clone())
            } else if local_name_is(dom, node, "lnTo") && p.len() >= 2 {
                OwnedCmd::L(p[0].clone(), p[1].clone())
            } else if local_name_is(dom, node, "cubicBezTo") && p.len() >= 6 {
                OwnedCmd::C(std::array::from_fn(|k| p[k].clone()))
            } else if local_name_is(dom, node, "quadBezTo") && p.len() >= 4 {
                OwnedCmd::Q(std::array::from_fn(|k| p[k].clone()))
            } else if local_name_is(dom, node, "arcTo") {
                let a = |key: &str| attr_any(dom, node, key).unwrap_or("0").to_string();
                OwnedCmd::A(a("wR"), a("hR"), a("stAng"), a("swAng"))
            } else if local_name_is(dom, node, "close") {
                OwnedCmd::Z
            } else {
                continue;
            };
            cmds.push(cmd);
        }
        let space = |v: f64, key: &str| if v > 0.0 { v } else { extent(key) };
        paths.push(CustomPath {
            w: space(num("w"), "cx"),
            h: space(num("h"), "cy"),
            fill,
            stroke,
            cmds,
        });
    }
    (!paths.is_empty()).then(|| CustomGeom {
        av: guides("avLst"),
        gd: guides("gdLst"),
        paths,
    })
}

/// A horizontal text box holding only paragraphs lays them out as body
/// paragraphs; tables and vertical text keep the flat-run path.
fn txbx_lays_out_paragraphs(dom: &Dom, shape: NodeId, txbx: NodeId) -> bool {
    // bodyPr is wps:, outside first_named_any's namespaces.
    let horizontal = descendants_local(dom, shape, "bodyPr")
        .first()
        .and_then(|b| attr_any(dom, *b, "vert"))
        .is_none_or(|v| v == "horz");
    horizontal
        && dom
            .descendants(txbx, Some(&W::p()))
            .iter()
            .any(|p| !w_text(dom, *p).trim().is_empty())
}

/// Word's spacing between the paragraphs stacked in a cell or a text box.
/// HTML auto spacing does not reach the container's edges: the first
/// paragraph's auto before and the last one's auto after drop. Between two
/// paragraphs only max(after, next before) stands, as in the body, so the
/// next before keeps only its excess. `breaks` lists the indices a nested
/// table sits in front of: the paragraphs around it are not neighbours, so
/// they keep their after and before in full.
pub(super) fn fold_stacked_spacing(mut styles: Vec<&mut ParaStyle>, breaks: &[usize]) {
    if let Some(first) = styles.first_mut()
        && first.before_auto
    {
        first.before = 0.0;
    }
    if let Some(last) = styles.last_mut()
        && last.after_auto
    {
        last.after = 0.0;
    }
    for i in (1..styles.len()).filter(|i| !breaks.contains(i)) {
        let (head, rest) = styles.split_at_mut(i);
        let (prev, next) = (&mut *head[i - 1], &mut *rest[0]);
        // contextualSpacing drops the space between same-style paragraphs
        // here as in the body (Word 16 probe ctx_cell_0930: a cell's
        // contextual Title lines step 20pt, plain ones 28), each flagged
        // paragraph its own share only (`contextual_gap`).
        let after = prev.after;
        if same_contextual_pair(prev, next) {
            prev.after = 0.0;
        }
        next.before = if same_contextual_pair(next, prev) {
            0.0
        } else {
            before_past(next.before, after, next.sum_spacing)
        };
    }
}

/// The text box's own paragraphs, each from its own style, spaced as Word
/// stacks them (`fold_stacked_spacing`: air_pollution_permit_form's
/// auto-spaced form lines stand 14pt apart, not 28).
fn txbx_paragraphs(
    dom: &Dom,
    txbx: NodeId,
    sheet: &StyleSheet,
    theme: &ThemeFonts,
    numbering: Option<&Numbering>,
    media: Option<(&PartFs, &str)>,
) -> TxbxContent {
    // A box's lists number on their own copy of the document's lists.
    let mut numbering = numbering.cloned();
    // The box's own paragraphs: not a nested box's, nor a cell's of one of
    // its tables, which lay out with their table.
    let own: Vec<NodeId> = dom
        .descendants(txbx, Some(&W::p()))
        .into_iter()
        .filter(|&p| {
            let mut at = dom.parent(p);
            while let Some(n) = at.filter(|&n| n != txbx) {
                if dom.name_is(n, &W::tbl()) || dom.name_is(n, &W::txbx_content()) {
                    return false;
                }
                at = dom.parent(n);
            }
            true
        })
        .collect();
    let mut tables = Vec::new();
    let mut before = 0;
    for child in dom.elements(txbx, None) {
        if dom.name_is(child, &W::tbl()) {
            let mut lists = numbering.clone().unwrap_or_default();
            let block = table_block(
                dom,
                child,
                sheet,
                &mut lists,
                &mut AuthorColors::default(),
                &HashMap::new(),
                media,
            );
            tables.push((before, std::rc::Rc::new(block)));
        } else {
            before += own
                .iter()
                .filter(|&&p| p == child || dom.ancestors(p, None).contains(&child))
                .count();
        }
    }
    let mut paras = own
        .into_iter()
        .map(|p| {
            let (mut style, run) = para_base(dom, p, sheet, None);
            let mut runs = if let Some((pkg, part)) = media {
                let mut runs = collect_runs_in(
                    dom,
                    p,
                    &run,
                    theme,
                    None,
                    &mut RunBag {
                        authors: &mut AuthorColors::default(),
                        comments: &HashMap::new(),
                        in_table: false,
                        toc: false,
                        open: None,
                        flow_pics: true,
                    },
                );
                let mut images = collect_images(pkg, part, dom, p, &|drawing| {
                    picture_run_style(dom, drawing, &run, theme, &sheet.by_id)
                });
                bind_flow_pictures(&mut runs, &mut images);
                strip_page_marks(runs)
            } else {
                collect_runs(dom, p, &run, theme)
            };
            // An empty paragraph is a line of its mark (010300e3's 8pt
            // blank between "Dear Ms. Smith:" and the letter), and so is
            // one of only spaces: a fitted box grows by the mark's line
            // (Word 16 probes c5b/c5c: 23.1pt under a Times 20 mark).
            if runs.iter().all(|r| r.text.trim().is_empty()) {
                let mut mark = run.clone();
                if let Some(rpr) = dom
                    .element(p, &W::p_pr())
                    .and_then(|ppr| dom.element(ppr, &W::r_pr()))
                {
                    apply_mark_rpr(dom, rpr, &mut mark, sheet);
                }
                if runs.is_empty() {
                    runs.push(TextRun::new(" ", mark.clone()));
                }
                style.mark_run = Some(std::rc::Rc::new(mark));
            }
            // A list paragraph in a box keeps its bullet and the level's
            // indent, as in the body (003329b5's "*" items in the
            // "NOS METHODES" box).
            if let Some(numbering) = numbering.as_mut() {
                let (marker, num_id, ilvl) = list_marker(dom, p, sheet, numbering);
                if !marker.is_empty() && numbering.pic_image(&num_id, ilvl).is_none() {
                    style.list_num.clone_from(&num_id);
                    let lvl = numbering.level(&num_id, ilvl);
                    let marker_style = apply_list_level(dom, p, sheet, lvl, &run, &mut style);
                    let mut mark = TextRun::new(marker, marker_style);
                    mark.list_marker = true;
                    runs.insert(0, mark);
                }
            }
            (runs, style)
        })
        .collect::<Vec<_>>();
    let breaks: Vec<usize> = tables.iter().map(|(at, _)| *at).collect();
    fold_stacked_spacing(paras.iter_mut().map(|(_, style)| style).collect(), &breaks);
    (paras, tables)
}

/// A text box's paragraphs and its tables (`LaidTextBox::tables`).
type TxbxContent = (
    Vec<(Vec<TextRun>, ParaStyle)>,
    Vec<(usize, std::rc::Rc<Block>)>,
);

/// `bodyPr` insets (EMU) or VML `v:textbox/@inset`, defaulting to Word's.
/// `wps:bodyPr wrap="none"` with `a:spAutoFit`: the box takes its text's
/// width (Word's "resize shape to fit text" without wrapping).
/// Whether `run` holds an inline DrawingML shape or group (not a picture,
/// chart or diagram, which size their own line).
pub(super) fn run_holds_inline_shape(dom: &Dom, run: NodeId) -> bool {
    descendants_local(dom, run, "inline")
        .into_iter()
        .any(|inline| {
            descendants_local(dom, inline, "graphicData")
                .first()
                .and_then(|&data| attr_any(dom, data, "uri"))
                .is_some_and(|uri| {
                    uri.ends_with("/wordprocessingShape") || uri.ends_with("/wordprocessingGroup")
                })
        })
}

/// The `w:position` (pt) of the run holding `node`, 0 outside a run.
fn run_raise_pt(dom: &Dom, node: NodeId) -> f32 {
    let mut at = Some(node);
    while let Some(n) = at {
        if dom.name_is(n, &W::r()) {
            return dom
                .element(n, &W::r_pr())
                .and_then(|pr| first_named(dom, pr, "position"))
                .and_then(|pos| attr_any(dom, pos, "val"))
                .and_then(parse_finite_f32)
                .map_or(0.0, |half| half / 2.0);
        }
        at = dom.parent(n);
    }
    0.0
}

fn bodypr_fits_width(dom: &Dom, shape: NodeId) -> bool {
    descendants_local(dom, shape, "bodyPr")
        .first()
        .is_some_and(|&body| {
            attr_any(dom, body, "wrap") == Some("none")
                && !descendants_local(dom, body, "spAutoFit").is_empty()
        })
}

/// A list marker merged with its item's text (one style: "1.\tword …")
/// split after its tab, so the item starts at the indent and a justified
/// first line spreads from there (Word 16 probe c4num, 2026-10-01).
pub(super) fn split_merged_marker(mut line: Vec<TextRun>) -> Vec<TextRun> {
    let Some(first) = line.first().filter(|run| run.list_marker) else {
        return line;
    };
    let Some(tab) = first
        .text
        .find('\t')
        .filter(|&at| at + 1 < first.text.len())
    else {
        return line;
    };
    let mut item = first.with_text(&first.text[tab + 1..]);
    item.list_marker = false;
    line[0] = first.with_text(&first.text[..=tab]);
    line.insert(1, item);
    line
}

/// The text's insets inside its shape's outline: Word sets the text in
/// by half a painted `a:ln` too, the inner half of a stroke that straddles
/// the edge, so a fitted box grows by the whole width (Word 16 probes
/// c6/c7, 2026-10-01: a 4pt line moves "QQ" 1.9pt in and fits its box
/// 3.8pt taller, a 1pt line 0.5 and 0.9).
fn textbox_text_insets(dom: &Dom, shape: NodeId, theme: &ThemeFonts) -> [f32; 4] {
    let stroked = descendants_local(dom, shape, "ln")
        .into_iter()
        .any(|ln| !descendants_local(dom, ln, "solidFill").is_empty());
    let half = if stroked {
        shape_line_width(dom, shape, theme) / 2.0
    } else {
        0.0
    };
    textbox_insets(dom, shape).map(|inset| inset + half)
}

pub(super) fn textbox_insets(dom: &Dom, shape: NodeId) -> [f32; 4] {
    let mut ins = TXBX_INSETS;
    if let Some(body) = descendants_local(dom, shape, "bodyPr").first().copied() {
        for (i, key) in ["lIns", "tIns", "rIns", "bIns"].iter().enumerate() {
            if let Some(v) = attr_any(dom, body, key).and_then(parse_finite_f32) {
                ins[i] = v / 12700.0;
            }
        }
    } else if let Some(tb) = descendants_local(dom, shape, "textbox").first()
        && let Some(raw) = attr_any(dom, *tb, "inset")
    {
        for (i, part) in raw.split(',').take(4).enumerate() {
            if let Some(v) = vml_len_pt(part.trim()) {
                ins[i] = v;
            }
        }
    }
    ins
}

/// A VML length (`0.1in`, `7.2pt`, `2mm`, `0.5cm`, bare EMU) in points.
/// A non-finite operand, or a conversion that overflows, is absent.
pub(super) fn vml_len_pt(raw: &str) -> Option<f32> {
    let (num, unit) = raw
        .find(|c: char| c.is_ascii_alphabetic())
        .map_or((raw, ""), |i| raw.split_at(i));
    let v = parse_finite_f32(num.trim())?;
    let points = match unit {
        "in" => v * 72.0,
        "pt" => v,
        "cm" => v * 72.0 / 2.54,
        "mm" => v * 72.0 / 25.4,
        "px" => v * 0.75,
        "" => v / 12700.0,
        _ => return None,
    };
    points.is_finite().then_some(points)
}

/// Word 2008+ can park textbox paragraphs in `word/txbxN.xml` and leave
/// `<wps:txbx r:txbx="rIdN"/>` empty (mcdoc). Follow the rel.
fn first_para_content_dx(dom: &Dom, root: NodeId) -> f32 {
    let Some(p) = dom.descendants(root, Some(&W::p())).into_iter().next() else {
        return 0.0;
    };
    let Some(ppr) = dom.element(p, &W::p_pr()) else {
        return 0.0;
    };
    let Some(ind) = first_named(dom, ppr, "ind") else {
        return 0.0;
    };
    let left = attr_any(dom, ind, "left")
        .or_else(|| attr_any(dom, ind, "start"))
        .and_then(parse_len)
        .unwrap_or(0.0);
    let first = attr_any(dom, ind, "firstLine")
        .and_then(parse_len)
        .unwrap_or(0.0);
    left + first
}

fn first_para_spacing_before(dom: &Dom, root: NodeId) -> f32 {
    let Some(p) = dom.descendants(root, Some(&W::p())).into_iter().next() else {
        return 0.0;
    };
    let Some(ppr) = dom.element(p, &W::p_pr()) else {
        return 0.0;
    };
    let Some(sp) = first_named(dom, ppr, "spacing") else {
        return 0.0;
    };
    attr_any(dom, sp, "before")
        .and_then(parse_len)
        .unwrap_or(0.0)
}

fn linked_txbx_content(
    src: Option<(&PartFs, &str)>,
    dom: &Dom,
    shape: NodeId,
    base: &RunStyle,
    theme: &ThemeFonts,
) -> (Vec<TextRun>, f32, f32) {
    let Some((pkg, main)) = src else {
        return (Vec::new(), 0.0, 0.0);
    };
    for node in descendants_local(dom, shape, "txbx") {
        let Some(rid) = attr_any(dom, node, "txbx") else {
            continue;
        };
        let Some(bytes) = resolve_media(pkg, main, rid) else {
            continue;
        };
        let xml = String::from_utf8_lossy(&bytes);
        let mut part = Dom::new();
        let doc = part.parse_xdocument(&xml);
        let Some(root) = part.root(doc) else {
            continue;
        };
        let runs = collect_runs(&part, root, base, theme);
        if runs.iter().any(|r| !r.text.trim().is_empty()) {
            return (
                runs,
                first_para_content_dx(&part, root),
                first_para_spacing_before(&part, root),
            );
        }
    }
    (Vec::new(), 0.0, 0.0)
}

pub(super) fn diagram_label_runs(
    src: Option<(&PartFs, &str)>,
    dom: &Dom,
    shape: NodeId,
    base: &RunStyle,
) -> Vec<TextRun> {
    let Some((pkg, main)) = src else {
        return Vec::new();
    };
    let Some(ids) = descendants_local(dom, shape, "relIds").into_iter().next() else {
        return Vec::new();
    };
    let Some(rid) = attr_any(dom, ids, "dm") else {
        return Vec::new();
    };
    let Some(bytes) = resolve_media(pkg, main, rid) else {
        return Vec::new();
    };
    let xml = String::from_utf8_lossy(&bytes);
    let mut part = Dom::new();
    let doc = part.parse_xdocument(&xml);
    let Some(root) = part.root(doc) else {
        return Vec::new();
    };
    let labels: Vec<String> = descendants_local(&part, root, "t")
        .into_iter()
        .map(|n| element_text(&part, n))
        .filter(|s| !s.trim().is_empty())
        .collect();
    let last = labels.len().saturating_sub(1);
    labels
        .into_iter()
        .enumerate()
        .map(|(i, mut label)| {
            if i < last {
                label.push('\n');
            }
            TextRun::new(label, base.clone())
        })
        .collect()
}

/// The drawing part of this diagram: its `dgm:relIds/@r:dm` data part
/// names it in `dsp:dataModelExt/@relId` (docxide case59's four diagrams
/// each have their own; the first diagramDrawing painted all four).
fn diagram_drawing_xml(pkg: &PartFs, main: &str, dom: &Dom, shape: NodeId) -> Option<String> {
    let ids = descendants_local(dom, shape, "relIds").into_iter().next()?;
    let dm = attr_any(dom, ids, "dm")?;
    let data = pkg.part_string(&rel_target_path(pkg, main, dm)?)?;
    let ext = data.find("dataModelExt")?;
    let rest = &data[ext..];
    let at = rest.find("relId=\"")? + "relId=\"".len();
    let rid = &rest[at..at + rest[at..].find('"')?];
    pkg.part_string(&rel_target_path(pkg, main, rid)?)
}

pub(super) fn load_diag_shapes(
    pkg: &PartFs,
    main: &str,
    host: &Dom,
    shape: NodeId,
    theme: &ThemeFonts,
) -> Option<Vec<DiagShape>> {
    let xml = diagram_drawing_xml(pkg, main, host, shape)
        .or_else(|| part_xml_by_rel_kind(pkg, main, "diagramDrawing"))?;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc)?;
    let mut out = Vec::new();
    for sp in descendants_local(&dom, root, "sp") {
        let Some(xfrm) = descendants_local(&dom, sp, "xfrm").into_iter().next() else {
            continue;
        };
        let Some(off) = descendants_local(&dom, xfrm, "off").into_iter().next() else {
            continue;
        };
        let Some(ext) = descendants_local(&dom, xfrm, "ext").into_iter().next() else {
            continue;
        };
        let x = emu_attr(&dom, off, "x");
        let y = emu_attr(&dom, off, "y");
        let w = emu_attr(&dom, ext, "cx");
        let h = emu_attr(&dom, ext, "cy");
        if w < 8.0 || h < 8.0 {
            continue;
        }
        let stroke = diag_ln_stroke(&dom, sp, theme);
        // Word Diagram 1 lt1 bars are opaque white + accent1 stroke
        // (covers the behind-doc watermark). Skip fill-only near-white
        // and still skip lt1 *strokes* (roundRect halo). Same class as
        // ChartSpace white (KEEP 562).
        let fill =
            diag_solid_fill(&dom, sp, theme).filter(|c| !is_near_white(*c) || stroke.is_some());
        if fill.is_none() && stroke.is_none() {
            continue;
        }
        let label = descendants_local(&dom, sp, "t")
            .into_iter()
            .map(|n| element_text(&dom, n))
            .filter(|s| !s.trim().is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        let round = descendants_local(&dom, sp, "prstGeom")
            .into_iter()
            .next()
            .and_then(|n| attr_any(&dom, n, "prst"))
            .is_some_and(|p| p == "roundRect");
        out.push(DiagShape {
            x,
            y,
            w,
            h,
            fill,
            stroke,
            label,
            round,
        });
    }
    if out.is_empty() { None } else { Some(out) }
}

pub(super) fn is_near_white(c: [f32; 3]) -> bool {
    c[0] > 0.95 && c[1] > 0.95 && c[2] > 0.95
}

/// A `dsp:sp` fill: a scheme slot or an explicit sRGB (docxide case37/59's
/// srgbClr shapes were dropped as unfilled).
pub(super) fn diag_solid_fill(dom: &Dom, sp: NodeId, theme: &ThemeFonts) -> Option<[f32; 3]> {
    let fill = descendants_local(dom, sp, "solidFill")
        .into_iter()
        .find(|f| {
            !dom.ancestors(*f, None)
                .iter()
                .any(|a| local_name_is(dom, *a, "ln"))
        })?;
    scheme_color(dom, fill, theme)
}

pub(super) fn diag_ln_stroke(dom: &Dom, sp: NodeId, theme: &ThemeFonts) -> Option<([f32; 3], f32)> {
    let ln = descendants_local(dom, sp, "ln").into_iter().next()?;
    if !descendants_local(dom, ln, "noFill").is_empty() {
        return None;
    }
    let color = descendants_local(dom, ln, "solidFill")
        .into_iter()
        .next()
        .and_then(|f| scheme_color(dom, f, theme))?;
    if is_near_white(color) {
        return None;
    }
    let width = attr_any(dom, ln, "w")
        .and_then(parse_finite_f64)
        .and_then(|emu| narrow_finite_f32(emu / 12700.0))
        .unwrap_or(1.0)
        .clamp(0.4, 4.0);
    Some((color, width))
}

pub(super) fn emu_attr(dom: &Dom, node: NodeId, name: &str) -> f32 {
    attr_any(dom, node, name)
        .and_then(parse_finite_f64)
        .and_then(|v| narrow_finite_f32(v / 12700.0))
        .unwrap_or(0.0)
}

fn drawing_is_chart_or_diagram(dom: &Dom, node: NodeId) -> bool {
    graphic_data_uri_contains(dom, node, "/chart")
        || graphic_data_uri_contains(dom, node, "/diagram")
}

/// A drawing whose text box holds text of its own: pictures inside that
/// text are content, not the drawing's picture (00f49849's header sidebar
/// with a QR code). A text box holding only a picture still paints as it
/// (0049484c).
pub(super) fn nested_in_text(dom: &Dom, node: NodeId, drawing: NodeId) -> bool {
    // Deleted words are box text too: a redline shows them struck
    // (d20125ec's cover box, all w:delText, is no picture).
    inside_text_box(dom, node, drawing)
        && descendants_local(dom, drawing, "txbxContent")
            .into_iter()
            .any(|tx| {
                [W::t(), W::del_text()].iter().any(|name| {
                    dom.descendants(tx, Some(name))
                        .into_iter()
                        .any(|t| !element_text(dom, t).trim().is_empty())
                })
            })
}

/// The shape's own graphicData, not those of pictures inside its text box
/// (00f49849's header text box holding a QR code is no picture).
fn graphic_data_uri_contains(dom: &Dom, node: NodeId, needle: &str) -> bool {
    descendants_local(dom, node, "graphicData")
        .into_iter()
        .filter(|gd| !nested_in_text(dom, *gd, node))
        .any(|gd| attr_any(dom, gd, "uri").is_some_and(|uri| uri.contains(needle)))
}

pub(super) fn local_name_is(dom: &Dom, node: NodeId, local: &str) -> bool {
    dom.name(node).is_some_and(|n| n.local_name() == local)
}

pub(super) fn descendants_local(dom: &Dom, node: NodeId, local: &str) -> Vec<NodeId> {
    dom.descendants(node, None)
        .into_iter()
        .filter(|&n| local_name_is(dom, n, local))
        .collect()
}

/// The `w:t` text under `node` (a paragraph's words; `element_text` reads
/// only the node's own text children, so it is empty for a `w:p`).
/// A node's words, deleted ones included: a redline shows them struck
/// (d20125ec's cover text box, every paragraph deleted, still lays out
/// line by line).
pub(super) fn w_text(dom: &Dom, node: NodeId) -> String {
    let mut words: Vec<NodeId> = dom.descendants(node, Some(&W::t()));
    words.extend(dom.descendants(node, Some(&W::del_text())));
    words.into_iter().map(|t| element_text(dom, t)).collect()
}

/// A paragraph's own words: `w_text` without the text of text boxes it
/// hosts (0017dd5f's footer logo shape carries the page number).
pub(super) fn para_own_text(dom: &Dom, para: NodeId) -> String {
    dom.descendants(para, Some(&W::t()))
        .into_iter()
        .filter(|t| dom.ancestors(*t, Some(&W::txbx_content())).is_empty())
        .map(|t| element_text(dom, t))
        .collect()
}

/// A run's `w:rPr` elements that come before its content, in order: Word
/// applies each (001f2a51's hyperlink runs carry rStyle then sz in two)
/// and ignores one after the text (its `<w:t/><w:rPr><w:sz 18/>` titles
/// stay 11pt).
/// Whether a run with the leading `w:rPr` blocks `rprs` is hidden. The
/// blocks apply in order, each its character style's `w:vanish` and then
/// its own, over `start`, what the paragraph's style gives (PR #247 review:
/// a later style that turns vanish off shows the run again; a paragraph
/// style's vanish hides runs that say nothing).
pub(super) fn run_hidden(
    dom: &Dom,
    rprs: &[NodeId],
    styles: Option<&HashMap<String, NamedStyle>>,
    start: bool,
) -> bool {
    rprs.iter().fold(start, |on, &rpr| {
        let styled = first_named(dom, rpr, "rStyle")
            .and_then(|n| dom.attribute(n, &W::val()))
            .and_then(|sid| styles.and_then(|s| s.get(sid)))
            .and_then(|named| named.hidden);
        first_named(dom, rpr, "vanish")
            .map(|n| !val_is_false(dom, Some(n)))
            .or(styled)
            .unwrap_or(on)
    })
}

/// The `w:vanish` a paragraph's own `w:pStyle` chain gives its runs.
pub(super) fn para_style_hidden(
    dom: &Dom,
    para: NodeId,
    styles: &HashMap<String, NamedStyle>,
) -> bool {
    dom.element(para, &W::p_pr())
        .and_then(|ppr| first_named(dom, ppr, "pStyle"))
        .and_then(|n| dom.attribute(n, &W::val()))
        .and_then(|sid| styles.get(sid))
        .and_then(|named| named.hidden)
        .unwrap_or(false)
}

pub(crate) fn leading_rprs(dom: &Dom, run: NodeId) -> Vec<NodeId> {
    let mut out = Vec::new();
    for i in 0..dom.child_count(run) {
        let child = dom.child_at(run, i);
        if dom.name_is(child, &W::r_pr()) {
            out.push(child);
        } else if dom.text_value(child).is_none() {
            break;
        }
    }
    out
}

pub(super) fn element_text(dom: &Dom, node: NodeId) -> String {
    let mut out = String::new();
    for i in 0..dom.child_count(node) {
        if let Some(t) = dom.text_value(dom.child_at(node, i)) {
            out.push_str(t);
        }
    }
    out
}

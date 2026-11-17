// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Shape colours, fills and lines, and picture effects and decoding.

use super::*;

pub(super) fn scheme_color(dom: &Dom, node: NodeId, theme: &ThemeFonts) -> Option<[f32; 3]> {
    if let Some(srgb) = descendants_local(dom, node, "srgbClr").into_iter().next() {
        let mut color = parse_hex_color(attr_any(dom, srgb, "val")?)?;
        apply_lum(dom, srgb, &mut color);
        return Some(color);
    }
    if let Some(scheme) = descendants_local(dom, node, "schemeClr").into_iter().next() {
        let mut color = theme.slot_color(attr_any(dom, scheme, "val")?)?;
        apply_lum(dom, scheme, &mut color);
        return Some(color);
    }
    // 212a1c9d's form boxes stroke `a:prstClr val="black"`; sysClr paints
    // its lastClr.
    if let Some(sys) = descendants_local(dom, node, "sysClr").into_iter().next() {
        let mut color = parse_hex_color(attr_any(dom, sys, "lastClr")?)?;
        apply_lum(dom, sys, &mut color);
        return Some(color);
    }
    let preset = descendants_local(dom, node, "prstClr").into_iter().next()?;
    let mut color = vml_color(attr_any(dom, preset, "val")?)?;
    apply_lum(dom, preset, &mut color);
    Some(color)
}

pub(super) fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) * 0.5;
    let d = max - min;
    if d < 1e-8 {
        return (0.0, 0.0, l);
    }
    let s = d / (1.0 - (2.0 * l - 1.0).abs());
    let h = if (max - r).abs() <= (max - g).abs() && (max - r).abs() <= (max - b).abs() {
        ((g - b) / d + if g < b { 6.0 } else { 0.0 }) / 6.0
    } else if (max - g).abs() <= (max - b).abs() {
        ((b - r) / d + 2.0) / 6.0
    } else {
        ((r - g) / d + 4.0) / 6.0
    };
    (h, s, l)
}

pub(super) fn hsl_to_rgb(h: f32, s: f32, l: f32) -> [f32; 3] {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let hp = (h * 6.0).rem_euclid(6.0);
    let x = c * (1.0 - (hp.rem_euclid(2.0) - 1.0).abs());
    let (r1, g1, b1) = if hp < 1.0 {
        (c, x, 0.0)
    } else if hp < 2.0 {
        (x, c, 0.0)
    } else if hp < 3.0 {
        (0.0, c, x)
    } else if hp < 4.0 {
        (0.0, x, c)
    } else if hp < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    let m = l - c * 0.5;
    [r1 + m, g1 + m, b1 + m]
}

pub(super) fn apply_hsl_lum_mod(color: &mut [f32; 3], lum_mod: f32) {
    // DrawingML lumMod with no lumOff is HSL L*=mod, then 8-bit round
    // (Strict01 Rectangle 468 bg2 50% → Word 0.463 0.443 0.443).
    let (h, s, l) = rgb_to_hsl(color[0], color[1], color[2]);
    let rgb = hsl_to_rgb(h, s, (l * lum_mod).clamp(0.0, 1.0));
    for (dst, src) in color.iter_mut().zip(rgb) {
        *dst = ((src * 255.0).round() / 255.0).clamp(0.0, 1.0);
    }
}

fn apply_lum(dom: &Dom, node: NodeId, color: &mut [f32; 3]) {
    let lum_mod = descendants_local(dom, node, "lumMod")
        .into_iter()
        .find_map(|n| attr_any(dom, n, "val").and_then(parse_finite_f32))
        .unwrap_or(100_000.0)
        / 100_000.0;
    let lum_off = descendants_local(dom, node, "lumOff")
        .into_iter()
        .find_map(|n| attr_any(dom, n, "val").and_then(parse_finite_f32))
        .unwrap_or(0.0)
        / 100_000.0;
    if lum_off.abs() < 1e-6 && (lum_mod - 1.0).abs() > 1e-6 {
        apply_hsl_lum_mod(color, lum_mod);
    } else {
        for c in color.iter_mut() {
            *c = (*c * lum_mod + lum_off).clamp(0.0, 1.0);
        }
    }
    let Some(shade) = descendants_local(dom, node, "shade")
        .into_iter()
        .find_map(|n| attr_any(dom, n, "val").and_then(parse_finite_f32))
        .map(|v| (v / 100_000.0).clamp(0.0, 1.0))
    else {
        return;
    };
    for c in color.iter_mut() {
        *c = srgb_shade(*c, shade);
    }
}

pub(super) fn srgb_to_linear(channel: f32) -> f32 {
    if channel <= 0.04045 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

pub(super) fn linear_to_srgb(channel: f32) -> f32 {
    if channel <= 0.0031308 {
        12.92 * channel
    } else {
        1.055 * channel.powf(1.0 / 2.4) - 0.055
    }
}

pub(super) fn srgb_shade(channel: f32, shade: f32) -> f32 {
    // DrawingML a:shade is linear sRGB; Word Quartz then 8-bit rounds
    // (Strict01 lnRef accent1 shade 50000 → 0.255 0.443 0.612).
    let srgb = linear_to_srgb((srgb_to_linear(channel) * shade).clamp(0.0, 1.0));
    (srgb * 255.0).round() / 255.0
}

fn shape_has_no_fill(dom: &Dom, shape: NodeId) -> bool {
    descendants_local(dom, shape, "spPr").into_iter().any(|sp| {
        (0..dom.child_count(sp)).any(|i| local_name_is(dom, dom.child_at(sp, i), "noFill"))
    })
}

/// `spPr/a:blipFill`: the shape's fill is its picture, which paints as an
/// image; the style's fillRef colour never goes over it (00feb732's
/// photo rectangle, accent1 fillRef).
fn shape_has_picture_fill(dom: &Dom, shape: NodeId) -> bool {
    descendants_local(dom, shape, "spPr").into_iter().any(|sp| {
        (0..dom.child_count(sp)).any(|i| local_name_is(dom, dom.child_at(sp, i), "blipFill"))
    })
}

pub(super) fn shape_ln_is_nofill(dom: &Dom, shape: NodeId) -> bool {
    // Explicit `a:ln/a:noFill` overrides even a style-supplied outline
    // (Strict01 Text Box 465 Author).
    descendants_local(dom, shape, "ln")
        .into_iter()
        .any(|ln| !descendants_local(dom, ln, "noFill").is_empty())
}

fn fill_ref_idx(dom: &Dom, shape: NodeId) -> Option<i32> {
    descendants_local(dom, shape, "fillRef")
        .into_iter()
        .find_map(|n| attr_any(dom, n, "idx").and_then(|s| s.parse().ok()))
}

pub(super) fn ln_ref_idx(dom: &Dom, shape: NodeId) -> Option<i32> {
    descendants_local(dom, shape, "lnRef")
        .into_iter()
        .find_map(|n| attr_any(dom, n, "idx").and_then(|s| s.parse().ok()))
}

pub(super) fn shape_line_width(dom: &Dom, shape: NodeId, theme: &ThemeFonts) -> f32 {
    // a:ln/@w when present. Box emit still ignores this (mini 511).
    for ln in descendants_local(dom, shape, "ln") {
        if !descendants_local(dom, ln, "noFill").is_empty() {
            continue;
        }
        if let Some(w) = attr_any(dom, ln, "w").and_then(parse_finite_f64)
            && let Some(pt) = narrow_finite_f32(w / 12700.0)
        {
            // Word strokes the width as written (1f3856c4's 3175 EMU
            // arrows are 0.25pt in its PDF); w=0 keeps a visible hairline.
            return if pt > 0.0 { pt.min(4.0) } else { 0.4 };
        }
    }
    let idx = ln_ref_idx(dom, shape).unwrap_or(0);
    if let Some(w) = theme.ln_width_pt(idx) {
        return w;
    }
    // Office default lnStyleLst: idx 1/2/3 = 6350/12700/19050 EMU.
    match idx {
        1 => 0.5,
        2 => 1.0,
        3 => 1.5,
        _ => 1.0,
    }
}

/// The ECMA preset drawn through `preset_geom` for `geom`: shapes whose
/// Word rendering needs compound fills, shaded faces, a separate outline
/// path or open outlines.
pub(super) fn geom_preset_name(geom: ShapeGeom) -> Option<&'static str> {
    Some(match geom {
        ShapeGeom::UpDownArrow => "upDownArrow",
        ShapeGeom::Donut => "donut",
        ShapeGeom::Frame => "frame",
        ShapeGeom::Cube => "cube",
        ShapeGeom::FoldedCorner => "foldedCorner",
        ShapeGeom::Can => "can",
        ShapeGeom::Moon => "moon",
        ShapeGeom::CircularArrow => "circularArrow",
        ShapeGeom::SmileyFace => "smileyFace",
        ShapeGeom::NoSmoking => "noSmoking",
        ShapeGeom::Bevel => "bevel",
        ShapeGeom::LeftBracket => "leftBracket",
        ShapeGeom::RightBracket => "rightBracket",
        ShapeGeom::LeftBrace => "leftBrace",
        ShapeGeom::RightBrace => "rightBrace",
        ShapeGeom::BracePair => "bracePair",
        ShapeGeom::BracketPair => "bracketPair",
        ShapeGeom::LeftRightCircularArrow => "leftRightCircularArrow",
        ShapeGeom::LeftCircularArrow => "leftCircularArrow",
        ShapeGeom::FlowChartSort => "flowChartSort",
        ShapeGeom::FlowChartSummingJunction => "flowChartSummingJunction",
        ShapeGeom::FlowChartMagneticDrum => "flowChartMagneticDrum",
        ShapeGeom::FlowChartMagneticDisk => "flowChartMagneticDisk",
        ShapeGeom::FlowChartMagneticTape => "flowChartMagneticTape",
        ShapeGeom::BlockArc => "blockArc",
        ShapeGeom::Ribbon => "ribbon",
        ShapeGeom::Ribbon2 => "ribbon2",
        ShapeGeom::FlowChartOr => "flowChartOr",
        ShapeGeom::FlowChartOfflineStorage => "flowChartOfflineStorage",
        ShapeGeom::FlowChartInternalStorage => "flowChartInternalStorage",
        ShapeGeom::FlowChartPredefinedProcess => "flowChartPredefinedProcess",
        ShapeGeom::FlowChartMultidocument => "flowChartMultidocument",
        ShapeGeom::FlowChartPunchedTape => "flowChartPunchedTape",
        _ => return None,
    })
}

/// `a:prstGeom/a:avLst/a:gd` adjustments (`fmla="val N"`).
pub(super) fn preset_adjustments(dom: &Dom, shape: NodeId) -> Vec<(String, f64)> {
    let Some(prst) = descendants_local(dom, shape, "prstGeom").into_iter().next() else {
        return Vec::new();
    };
    descendants_local(dom, prst, "gd")
        .into_iter()
        .filter_map(|gd| {
            let name = attr_any(dom, gd, "name")?;
            let v = parse_finite_f64(attr_any(dom, gd, "fmla")?.strip_prefix("val ")?.trim())?;
            Some((name.to_string(), v))
        })
        .collect()
}

/// Every preset whose outline is a polygon stroke (not the tuned Box
/// rectangle, not the connectors that stroke themselves).
pub(super) fn geom_is_preset_polygon(geom: ShapeGeom) -> bool {
    !matches!(
        geom,
        ShapeGeom::Box | ShapeGeom::BentConnector | ShapeGeom::CurvedConnector | ShapeGeom::Line
    )
}

/// `node` sits inside an `a:ln` below `shape`: its fill paints the
/// outline, never the shape body.
fn under_line_props(dom: &Dom, shape: NodeId, node: NodeId) -> bool {
    let mut n = dom.parent(node);
    while let Some(p) = n {
        if p == shape {
            return false;
        }
        if local_name_is(dom, p, "ln") {
            return true;
        }
        n = dom.parent(p);
    }
    false
}

pub(super) fn shape_fill_color(dom: &Dom, shape: NodeId, theme: &ThemeFonts) -> Option<[f32; 3]> {
    if shape_has_no_fill(dom, shape) || shape_has_picture_fill(dom, shape) {
        return None;
    }
    if let Some(fill) = descendants_local(dom, shape, "solidFill")
        .into_iter()
        .filter(|n| !under_line_props(dom, shape, *n))
        .find_map(|n| scheme_color(dom, n, theme))
    {
        return Some(fill);
    }
    // Cover-page wash (Strict01 Rectangle 466): gradFill stops → first stop.
    // Mini 715 two-stop Type 2 axial was Word-faithful but ITT-neg
    // (Strict01 family −0.092 / 8 drops 0 gains). Keep first-stop solid.
    if let Some(gs) = descendants_local(dom, shape, "gs")
        .into_iter()
        .find(|n| !under_line_props(dom, shape, *n))
        && let Some(fill) = scheme_color(dom, gs, theme)
    {
        return Some(fill);
    }
    let idx = fill_ref_idx(dom, shape).unwrap_or(0);
    if idx == 0 {
        return None;
    }
    if let Some(c) = theme.fill_style(idx) {
        return Some(c);
    }
    descendants_local(dom, shape, "fillRef")
        .into_iter()
        .find_map(|n| scheme_color(dom, n, theme))
}

pub(super) fn shape_line_color(dom: &Dom, shape: NodeId, theme: &ThemeFonts) -> Option<[f32; 3]> {
    for ln in descendants_local(dom, shape, "ln") {
        if !descendants_local(dom, ln, "noFill").is_empty() {
            return None;
        }
        if let Some(c) = scheme_color(dom, ln, theme) {
            return Some(c);
        }
    }
    let idx = ln_ref_idx(dom, shape).unwrap_or(0);
    if idx == 0 {
        return None;
    }
    if let Some(c) = theme.ln_color(idx) {
        return Some(c);
    }
    descendants_local(dom, shape, "lnRef")
        .into_iter()
        .find_map(|n| scheme_color(dom, n, theme))
}

pub(super) fn pos_offset_pt(dom: &Dom, node: Option<NodeId>) -> Option<f32> {
    let parent = node?;
    let off = first_named_any(dom, parent, "posOffset")?;
    let emu = parse_finite_f64(local_text(dom, off).trim())?;
    narrow_finite_f32(emu / 12700.0)
}

/// `wp:wrapPolygon`'s lowest vertex as a fraction of the extent (21600
/// units); 1 when there is none.
pub(super) fn wrap_polygon_bottom(dom: &Dom, drawing: NodeId) -> f32 {
    let Some(poly) = first_named_any(dom, drawing, "wrapPolygon") else {
        return 1.0;
    };
    let lowest = ["start", "lineTo"]
        .iter()
        .flat_map(|local| dom.descendants(poly, Some(&WP::name(local))))
        .filter_map(|pt| attr_any(dom, pt, "y").and_then(parse_finite_f32))
        .fold(f32::MIN, f32::max);
    if lowest > 0.0 {
        (lowest / 21600.0).min(1.0)
    } else {
        1.0
    }
}

pub(super) fn first_named_any(dom: &Dom, node: NodeId, local: &str) -> Option<NodeId> {
    for idx_walk in [
        WP::name(local),
        W::name(local),
        A::name(local),
        WNE::name(local),
    ] {
        if let Some(found) = dom.find_descendant_element(node, Some(&idx_walk), |_| true) {
            return Some(found);
        }
    }
    None
}

pub(super) fn attr_any<'a>(dom: &'a Dom, node: NodeId, local: &str) -> Option<&'a str> {
    for name in [
        XName::get(local, ""),
        W::name(local),
        W14::name(local),
        WP::name(local),
        A::name(local),
        R::name(local),
    ] {
        if let Some(v) = dom.attribute(node, &name) {
            return Some(v);
        }
    }
    None
}

pub(super) fn resolve_media(pkg: &PartFs, source_part: &str, rel_id: &str) -> Option<Vec<u8>> {
    let path = rel_target_path(pkg, source_part, rel_id)?;
    pkg.part_bytes(&path).map(<[u8]>::to_vec)
}

/// The picture's own box inside its laid-out box (x, y bottom-left): the
/// `wp:effectExtent` margins stay empty.
pub(super) fn inset_box(img: &LaidImage, x: f32, y: f32, dw: f32, dh: f32) -> (f32, f32, f32, f32) {
    let [l, t, r, b] = img.inset;
    (x + l, y + b, (dw - l - r).max(1.0), (dh - t - b).max(1.0))
}

/// The width of the rule on a cell's left edge: its own left border, else
/// the table's left (first column) or insideV rule; 0 when none paints.
/// The width of a cell's right rule: its own `tcBorders`, else the
/// table's right rim (last column) or inside rule.
pub(super) fn cell_right_rule(cell: &TableCell, borders: Option<TblBorders>, ncols: usize) -> f32 {
    if let Some(cb) = cell.borders {
        return cb.right.map_or(0.0, |(_, w)| w);
    }
    match borders {
        Some(b) if cell.col + cell.colspan.max(1) >= ncols && b.right => b.outer_width.max(0.24),
        Some(b) if cell.col + cell.colspan.max(1) < ncols && b.inside_v => b.width.max(0.24),
        _ => 0.0,
    }
}

pub(super) fn cell_left_rule(cell: &TableCell, borders: Option<TblBorders>) -> f32 {
    if let Some(cb) = cell.borders {
        return cb.left.map_or(0.0, |(_, w)| w);
    }
    match borders {
        Some(b) if cell.col == 0 && b.left => b.outer_width.max(0.24),
        Some(b) if cell.col > 0 && b.inside_v => b.width.max(0.24),
        _ => 0.0,
    }
}

/// A VML fixed-point fraction (`19661f` = 19661/65536) or plain number.
fn vml_fraction(value: &str) -> Option<f32> {
    match value.strip_suffix('f') {
        Some(fixed) => parse_finite_f32(fixed.trim()).map(|v| v / 65536.0),
        None => parse_finite_f32(value.trim()),
    }
}

/// Word's "Washout" picture setting: `v:imagedata` gain 0.3 with
/// blacklevel 0.35, the only pair its watermark dialog writes
/// (LibreOffice maps the same pair to its watermark mode).
pub(super) fn vml_washout(dom: &Dom, im: NodeId) -> bool {
    let near = |name: &str, want: f32| {
        attr_any(dom, im, name)
            .and_then(vml_fraction)
            .is_some_and(|v| (v - want).abs() < 0.01)
    };
    near("gain", 0.3) && near("blacklevel", 0.35)
}

/// The samples Word paints for a washed-out picture: 0041dade's Word PDF
/// maps each 8-bit sample to 0.29 * v + 206.7, clipped to white.
pub(super) fn washed_out(kind: ImageKind) -> ImageKind {
    if let ImageKind::Metafile { raster, texts } = kind {
        let raster = Box::new(washed_out(*raster));
        return ImageKind::Metafile { raster, texts };
    }
    let pale = |v: &mut u8| *v = (f32::from(*v) * 0.29 + 206.7).round().min(255.0) as u8;
    match kind {
        ImageKind::Jpeg { bytes, .. } => {
            let Ok(img) = image::load_from_memory(&bytes) else {
                return ImageKind::Reserve;
            };
            let rgb = img.to_rgb8();
            let (width, height) = (rgb.width(), rgb.height());
            let mut bytes = rgb.into_raw();
            bytes.iter_mut().for_each(pale);
            ImageKind::Rgb {
                width,
                height,
                bytes,
                alpha: None,
            }
        }
        ImageKind::Rgb {
            width,
            height,
            mut bytes,
            alpha,
        } => {
            bytes.iter_mut().for_each(pale);
            ImageKind::Rgb {
                width,
                height,
                bytes,
                alpha,
            }
        }
        other => other,
    }
}

/// A picture's `a:softEdge` radius in points.
pub(super) fn soft_edge_pt(dom: &Dom, drawing: NodeId) -> Option<f32> {
    let pic = descendants_local(dom, drawing, "spPr").into_iter().next()?;
    let edge = descendants_local(dom, pic, "softEdge").into_iter().next()?;
    let rad = parse_finite_f32(attr_any(dom, edge, "rad")?)? / 12_700.0;
    (rad > 0.0).then_some(rad)
}

/// `a:softEdge`: Word fades the picture to nothing at its border over the
/// radius (case57's photos). `fx`/`fy` are the radius as a fraction of the
/// picture's width and height.
pub(super) fn soften_edges(kind: ImageKind, fx: f32, fy: f32) -> ImageKind {
    if let ImageKind::Metafile { raster, texts } = kind {
        let raster = Box::new(soften_edges(*raster, fx, fy));
        return ImageKind::Metafile { raster, texts };
    }
    let (width, height, bytes, alpha) = match kind {
        ImageKind::Jpeg { bytes, .. } => {
            let Ok(img) = image::load_from_memory(&bytes) else {
                return ImageKind::Reserve;
            };
            let rgb = img.to_rgb8();
            (rgb.width(), rgb.height(), rgb.into_raw(), None)
        }
        ImageKind::Rgb {
            width,
            height,
            bytes,
            alpha,
        } => (width, height, bytes, alpha),
        other => return other,
    };
    let (rx, ry) = ((fx * width as f32).max(1.0), (fy * height as f32).max(1.0));
    let mut alpha = alpha.unwrap_or_else(|| vec![255; (width * height) as usize]);
    for y in 0..height {
        for x in 0..width {
            let dx = (x.min(width - 1 - x) as f32 + 0.5) / rx;
            let dy = (y.min(height - 1 - y) as f32 + 0.5) / ry;
            let f = dx.min(dy).min(1.0);
            let a = &mut alpha[(y * width + x) as usize];
            *a = (f32::from(*a) * f).round() as u8;
        }
    }
    ImageKind::Rgb {
        width,
        height,
        bytes,
        alpha: Some(alpha),
    }
}

/// One `a:duotone` colour child with its lum/shade and `satMod`
/// transforms; `None` for a colour we cannot resolve.
pub(super) fn duotone_color(dom: &Dom, node: NodeId, pkg: &PartFs) -> Option<[f32; 3]> {
    let mut color = if local_name_is(dom, node, "srgbClr") {
        parse_hex_color(attr_any(dom, node, "val")?)?
    } else if local_name_is(dom, node, "schemeClr") {
        load_theme(pkg).slot_color(attr_any(dom, node, "val")?)?
    } else if local_name_is(dom, node, "sysClr") {
        parse_hex_color(attr_any(dom, node, "lastClr")?)?
    } else if local_name_is(dom, node, "prstClr") {
        match attr_any(dom, node, "val")? {
            "white" => [1.0; 3],
            "black" => [0.0; 3],
            _ => return None,
        }
    } else {
        return None;
    };
    apply_lum(dom, node, &mut color);
    if let Some(sat) = descendants_local(dom, node, "satMod")
        .into_iter()
        .find_map(|n| attr_any(dom, n, "val").and_then(parse_finite_f32))
    {
        let (h, s, l) = rgb_to_hsl(color[0], color[1], color[2]);
        let rgb = hsl_to_rgb(h, (s * sat / 100_000.0).clamp(0.0, 1.0), l);
        for (dst, src) in color.iter_mut().zip(rgb) {
            *dst = ((src * 255.0).round() / 255.0).clamp(0.0, 1.0);
        }
    }
    Some(color)
}

/// `a:blip/a:duotone`: Word recolours each pixel between the two colours
/// by its Rec.709 luma (English corpus c301012f: green 70AD47 under
/// accent5 shade 45% satMod 135% → white paints A4B6D6).
pub(super) fn duotone_image(kind: ImageKind, dom: &Dom, blip: NodeId, pkg: &PartFs) -> ImageKind {
    if let ImageKind::Metafile { raster, texts } = kind {
        let raster = Box::new(duotone_image(*raster, dom, blip, pkg));
        return ImageKind::Metafile { raster, texts };
    }
    let Some(duotone) = (0..dom.child_count(blip))
        .map(|i| dom.child_at(blip, i))
        .find(|&c| local_name_is(dom, c, "duotone"))
    else {
        return kind;
    };
    let colors: Vec<[f32; 3]> = (0..dom.child_count(duotone))
        .map(|i| dom.child_at(duotone, i))
        .filter_map(|c| duotone_color(dom, c, pkg))
        .collect();
    let [c1, c2] = colors[..] else {
        return kind;
    };
    let (width, height, mut bytes, alpha) = match kind {
        ImageKind::Jpeg { bytes, .. } => {
            let Ok(img) = image::load_from_memory(&bytes) else {
                return ImageKind::Reserve;
            };
            let rgb = img.to_rgb8();
            (rgb.width(), rgb.height(), rgb.into_raw(), None)
        }
        ImageKind::Rgb {
            width,
            height,
            bytes,
            alpha,
        } => (width, height, bytes, alpha),
        other => return other,
    };
    for px in bytes.as_chunks_mut::<3>().0 {
        let luma =
            (0.2126 * f32::from(px[0]) + 0.7152 * f32::from(px[1]) + 0.0722 * f32::from(px[2]))
                / 255.0;
        for (i, v) in px.iter_mut().enumerate() {
            *v = ((c1[i] + (c2[i] - c1[i]) * luma) * 255.0)
                .round()
                .clamp(0.0, 255.0) as u8;
        }
    }
    ImageKind::Rgb {
        width,
        height,
        bytes,
        alpha,
    }
}

pub(super) fn decode_image(bytes: Vec<u8>) -> Option<ImageKind> {
    if let Some((width, height, rgb, texts)) = metafile::render(&bytes) {
        let raster = ImageKind::Rgb {
            width,
            height,
            bytes: rgb,
            alpha: None,
        };
        if texts.is_empty() {
            return Some(raster);
        }
        return Some(ImageKind::Metafile {
            raster: Box::new(raster),
            texts: texts.into(),
        });
    }
    if bytes.len() > 3
        && bytes[0] == 0xFF
        && bytes[1] == 0xD8
        && let Some((width, height, components)) = jpeg_info(&bytes)
    {
        // Word converts a CMYK photo to RGB for its PDF; embedded as
        // DeviceCMYK it renders far darker (fixtures_500 00feb732: 99% of
        // the photo under the ink threshold against Word's 64%).
        if components == 4
            && let Ok(img) = image::load_from_memory(&bytes)
        {
            let rgb = img.to_rgb8();
            return Some(ImageKind::Rgb {
                width: rgb.width(),
                height: rgb.height(),
                bytes: rgb.into_raw(),
                alpha: None,
            });
        }
        return Some(ImageKind::Jpeg {
            width,
            height,
            bytes,
            components,
        });
    }
    let img = image::load_from_memory(&bytes).ok()?;
    if img.color().has_alpha() {
        let rgba = img.to_rgba8();
        let (width, height) = (rgba.width(), rgba.height());
        let mut rgb = Vec::with_capacity((width * height * 3) as usize);
        let mut alpha = Vec::with_capacity((width * height) as usize);
        for px in rgba.pixels() {
            rgb.extend_from_slice(&px.0[..3]);
            alpha.push(px.0[3]);
        }
        return Some(ImageKind::Rgb {
            width,
            height,
            bytes: rgb,
            alpha: Some(alpha),
        });
    }
    let rgb = img.to_rgb8();
    Some(ImageKind::Rgb {
        width: rgb.width(),
        height: rgb.height(),
        bytes: rgb.into_raw(),
        alpha: None,
    })
}

pub(super) fn jpeg_info(data: &[u8]) -> Option<(u32, u32, u8)> {
    if data.len() < 4 || data[0] != 0xFF || data[1] != 0xD8 {
        return None;
    }
    let mut idx = 2;
    while idx + 8 < data.len() {
        if data[idx] != 0xFF {
            idx += 1;
            continue;
        }
        let marker = data[idx + 1];
        if marker == 0xD8 || marker == 0xD9 || (0xD0..=0xD7).contains(&marker) {
            idx += 2;
            continue;
        }
        if idx + 3 >= data.len() {
            break;
        }
        let len = u16::from_be_bytes([data[idx + 2], data[idx + 3]]) as usize;
        if matches!(marker, 0xC0..=0xC2) {
            if idx + 9 >= data.len() {
                break;
            }
            let height = u16::from_be_bytes([data[idx + 5], data[idx + 6]]) as u32;
            let width = u16::from_be_bytes([data[idx + 7], data[idx + 8]]) as u32;
            let components = data[idx + 9];
            return Some((width, height, components));
        }
        idx += 2 + len;
    }
    None
}

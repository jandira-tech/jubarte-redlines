// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Preset shape outlines and chart polygons as point lists.

use super::*;

pub(super) struct ConnectorCubic {
    pub(super) start: (f32, f32),
    pub(super) segments: Vec<[(f32, f32); 3]>,
}

pub(super) fn curved_connector_cubics(
    x: f32,
    y: f32,
    dw: f32,
    dh: f32,
    flip_h: bool,
    flip_v: bool,
) -> ConnectorCubic {
    // OOXML curvedConnector3 default adj1=50000: two cubics from (l,t)
    // to (r,b). Word Quartz uses quarter-height c2 / three-quarter c1
    // (Strict01 fitz 338.2,170.2 then 338.2,132.3). Collapsing both
    // onto the midpoint flattened the S-curve. flipV sends the stroke
    // bottom-left → top-right.
    let map = |lx: f32, ly: f32| {
        let ox = if flip_h { dw - lx } else { lx };
        let oy = if flip_v { dh - ly } else { ly };
        (x + ox, y + dh - oy)
    };
    ConnectorCubic {
        start: map(0.0, 0.0),
        segments: vec![
            [
                map(dw * 0.25, 0.0),
                map(dw * 0.5, dh * 0.25),
                map(dw * 0.5, dh * 0.5),
            ],
            [map(dw * 0.5, dh * 0.75), map(dw * 0.75, dh), map(dw, dh)],
        ],
    }
}

pub(super) fn arrowhead_triangle(from: (f32, f32), to: (f32, f32)) -> [(f32, f32); 3] {
    // Word Quartz tailEnd=triangle: ~6pt tip past the stroke end, 3pt half-base.
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let len = (dx * dx + dy * dy).sqrt().max(0.001);
    let (ux, uy) = (dx / len, dy / len);
    let (px, py) = (-uy, ux);
    let tip = (to.0 + ux * 6.0, to.1 + uy * 6.0);
    let base = (to.0 - ux * 1.0, to.1 - uy * 1.0);
    [
        (base.0 + px * 3.0, base.1 + py * 3.0),
        tip,
        (base.0 - px * 3.0, base.1 - py * 3.0),
    ]
}

pub(super) fn wave_underline_segments(x: f32, y: f32, w: f32) -> Vec<(f32, f32, f32, f32)> {
    // Word `w:u val="wave"` is a ~5pt-period, ~0.9pt-amplitude sine.
    // Phase is in page x so per-glyph paints join into one wave.
    let amp = 0.9;
    let period = 5.0;
    let step = period / 4.0;
    let phase = |px: f32| y + amp * (px / period * std::f32::consts::TAU).sin();
    let mut out = Vec::new();
    let mut x0 = x;
    let mut y0 = phase(x);
    let mut t = 0.0;
    while t < w - 0.05 {
        t = (t + step).min(w);
        let x1 = x + t;
        let y1 = phase(x1);
        out.push((x0, y0, x1, y1));
        x0 = x1;
        y0 = y1;
    }
    if out.is_empty() {
        out.push((x, y, x + w, y));
    }
    out
}

pub(super) fn bent_connector_points(x: f32, y: f32, dw: f32, dh: f32) -> [(f32, f32); 4] {
    // OOXML bentConnector3 default adj1=50000: elbow at w/2, full height.
    let mid = x + dw * 0.5;
    [(x, y + dh), (mid, y + dh), (mid, y), (x + dw, y)]
}

/// OOXML `roundRect` default adj=16667: corner radius is min(w,h)×1/6.
pub(super) fn ellipse_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    const STEPS: i32 = 24;
    let cx = x + w * 0.5;
    let cy = y + h * 0.5;
    let rx = (w * 0.5).max(0.5);
    let ry = (h * 0.5).max(0.5);
    (0..STEPS)
        .map(|i| {
            let a = i as f32 * std::f32::consts::TAU / STEPS as f32;
            (cx + rx * a.cos(), cy + ry * a.sin())
        })
        .collect()
}

pub(super) fn triangle_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML `triangle` (y-down apex at top) → PDF y-up apex at y+h.
    vec![(x + w * 0.5, y + h), (x + w, y), (x, y)]
}

pub(super) fn diamond_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    vec![
        (x + w * 0.5, y + h),
        (x + w, y + h * 0.5),
        (x + w * 0.5, y),
        (x, y + h * 0.5),
    ]
}

pub(super) fn hexagon_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML hexagon default adj=25000: left/right inset is 25% of width.
    let x1 = w * 25_000.0 / 100_000.0;
    vec![
        (x + x1, y + h),
        (x + w - x1, y + h),
        (x + w, y + h * 0.5),
        (x + w - x1, y),
        (x + x1, y),
        (x, y + h * 0.5),
    ]
}

fn preset_ss(w: f32, h: f32) -> f32 {
    w.min(h)
}

pub(super) fn parallelogram_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML parallelogram adj=25000: x2 = ss*adj/100000.
    let x2 = preset_ss(w, h) * 25_000.0 / 100_000.0;
    vec![(x, y), (x + x2, y + h), (x + w, y + h), (x + w - x2, y)]
}

pub(super) fn trapezoid_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML trapezoid adj=25000: short edge on top.
    let x2 = preset_ss(w, h) * 25_000.0 / 100_000.0;
    vec![(x, y), (x + x2, y + h), (x + w - x2, y + h), (x + w, y)]
}

pub(super) fn chevron_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML chevron adj=50000: x1 = ss*adj/100000, x2 = r-x1.
    let x1 = preset_ss(w, h) * 50_000.0 / 100_000.0;
    let x2 = w - x1;
    vec![
        (x, y + h),
        (x + x2, y + h),
        (x + w, y + h * 0.5),
        (x + x2, y),
        (x, y),
        (x + x1, y + h * 0.5),
    ]
}

pub(super) fn plus_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML plus adj=25000: x1 = ss*adj/100000 (arm inset from each edge).
    let x1 = preset_ss(w, h) * 25_000.0 / 100_000.0;
    vec![
        (x, y + h - x1),
        (x + x1, y + h - x1),
        (x + x1, y + h),
        (x + w - x1, y + h),
        (x + w - x1, y + h - x1),
        (x + w, y + h - x1),
        (x + w, y + x1),
        (x + w - x1, y + x1),
        (x + w - x1, y),
        (x + x1, y),
        (x + x1, y + x1),
        (x, y + x1),
    ]
}

pub(super) fn home_plate_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML homePlate adj=50000: dx1 = ss*adj/100000, x1 = r-dx1.
    let dx1 = preset_ss(w, h) * 50_000.0 / 100_000.0;
    let x1 = w - dx1;
    vec![
        (x, y + h),
        (x + x1, y + h),
        (x + w, y + h * 0.5),
        (x + x1, y),
        (x, y),
    ]
}

fn ooxml_ang_rad(sixtieths: f32) -> f32 {
    (sixtieths / 60_000.0).to_radians()
}

pub(super) fn pentagon_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML pentagon hf=105146 vf=110557; Cos/Sin angles in 1/60000 deg.
    let hc = w * 0.5;
    let vc = h * 0.5;
    let swd2 = (w * 0.5) * 105_146.0 / 100_000.0;
    let shd2 = (h * 0.5) * 110_557.0 / 100_000.0;
    let svc = vc * 110_557.0 / 100_000.0;
    let a18 = ooxml_ang_rad(1_080_000.0);
    let a306 = ooxml_ang_rad(18_360_000.0);
    let dx1 = swd2 * a18.cos();
    let dx2 = swd2 * a306.cos();
    let dy1 = shd2 * a18.sin();
    let dy2 = shd2 * a306.sin();
    let x1 = hc - dx1;
    let x2 = hc - dx2;
    let x3 = hc + dx2;
    let x4 = hc + dx1;
    let y1 = svc - dy1;
    let y2 = svc - dy2;
    let py = |yd: f32| y + h - yd;
    vec![
        (x + x1, py(y1)),
        (x + hc, py(0.0)),
        (x + x4, py(y1)),
        (x + x3, py(y2)),
        (x + x2, py(y2)),
    ]
}

pub(super) fn octagon_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML octagon adj=29289: x1 = ss*adj/100000.
    let x1 = preset_ss(w, h) * 29_289.0 / 100_000.0;
    vec![
        (x, y + h - x1),
        (x + x1, y + h),
        (x + w - x1, y + h),
        (x + w, y + h - x1),
        (x + w, y + x1),
        (x + w - x1, y),
        (x + x1, y),
        (x, y + x1),
    ]
}

pub(super) fn star4_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML star4 adj=12500; Cos/Sin 2700000 = 45°.
    let a = 12_500.0;
    let iwd2 = (w * 0.5) * a / 50_000.0;
    let ihd2 = (h * 0.5) * a / 50_000.0;
    let ang = ooxml_ang_rad(2_700_000.0);
    let sdx = iwd2 * ang.cos();
    let sdy = ihd2 * ang.sin();
    let hc = w * 0.5;
    let vc = h * 0.5;
    let sx1 = hc - sdx;
    let sx2 = hc + sdx;
    let sy1 = vc - sdy;
    let sy2 = vc + sdy;
    let py = |yd: f32| y + h - yd;
    vec![
        (x, py(vc)),
        (x + sx1, py(sy1)),
        (x + hc, py(0.0)),
        (x + sx2, py(sy1)),
        (x + w, py(vc)),
        (x + sx2, py(sy2)),
        (x + hc, py(h)),
        (x + sx1, py(sy2)),
    ]
}

pub(super) fn star5_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML star5 adj=19098 hf=105146 vf=110557.
    let a = 19_098.0;
    let hf = 105_146.0;
    let vf = 110_557.0;
    let hc = w * 0.5;
    let vc = h * 0.5;
    let swd2 = (w * 0.5) * hf / 100_000.0;
    let shd2 = (h * 0.5) * vf / 100_000.0;
    let svc = vc * vf / 100_000.0;
    let a18 = ooxml_ang_rad(1_080_000.0);
    let a306 = ooxml_ang_rad(18_360_000.0);
    let dx1 = swd2 * a18.cos();
    let dx2 = swd2 * a306.cos();
    let dy1 = shd2 * a18.sin();
    let dy2 = shd2 * a306.sin();
    let x1 = hc - dx1;
    let x2 = hc - dx2;
    let x3 = hc + dx2;
    let x4 = hc + dx1;
    let y1 = svc - dy1;
    let y2 = svc - dy2;
    let iwd2 = swd2 * a / 50_000.0;
    let ihd2 = shd2 * a / 50_000.0;
    let a54 = ooxml_ang_rad(3_240_000.0);
    let a342 = ooxml_ang_rad(20_520_000.0);
    let sdx1 = iwd2 * a342.cos();
    let sdx2 = iwd2 * a54.cos();
    let sdy1 = ihd2 * a54.sin();
    let sdy2 = ihd2 * a342.sin();
    let sx1 = hc - sdx1;
    let sx2 = hc - sdx2;
    let sx3 = hc + sdx2;
    let sx4 = hc + sdx1;
    let sy1 = svc - sdy1;
    let sy2 = svc - sdy2;
    let sy3 = svc + ihd2;
    let py = |yd: f32| y + h - yd;
    vec![
        (x + x1, py(y1)),
        (x + sx2, py(sy1)),
        (x + hc, py(0.0)),
        (x + sx3, py(sy1)),
        (x + x4, py(y1)),
        (x + sx4, py(sy2)),
        (x + x3, py(y2)),
        (x + hc, py(sy3)),
        (x + x2, py(y2)),
        (x + sx1, py(sy2)),
    ]
}

pub(super) fn rt_triangle_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML rtTriangle: M l,b L l,t L r,b Z (right angle at bottom-left).
    vec![(x, y), (x, y + h), (x + w, y)]
}

fn sample_cubic(
    p0: (f32, f32),
    p1: (f32, f32),
    p2: (f32, f32),
    p3: (f32, f32),
    steps: i32,
    out: &mut Vec<(f32, f32)>,
) {
    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let u = 1.0 - t;
        let uu = u * u;
        let tt = t * t;
        out.push((
            uu * u * p0.0 + 3.0 * uu * t * p1.0 + 3.0 * u * tt * p2.0 + tt * t * p3.0,
            uu * u * p0.1 + 3.0 * uu * t * p1.1 + 3.0 * u * tt * p2.1 + tt * t * p3.1,
        ));
    }
}

pub(super) fn heart_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML heart: M hc,hd4 C x3,y1 x4,hd4 hc,b C x1,hd4 x2,y1 hc,hd4.
    // y1 = t − hd3 sits above the box; PDF y-up flips OOXML y-down.
    let hc = x + w * 0.5;
    let dx1 = w * 49.0 / 48.0;
    let dx2 = w * 10.0 / 48.0;
    let x1 = hc - dx1;
    let x2 = hc - dx2;
    let x3 = hc + dx2;
    let x4 = hc + dx1;
    let py = |yd: f32| y + h - yd;
    let hd4 = h * 0.25;
    let y1 = -h / 3.0;
    let start = (hc, py(hd4));
    let bottom = (hc, py(h));
    let mut pts = vec![start];
    sample_cubic(start, (x3, py(y1)), (x4, py(hd4)), bottom, 8, &mut pts);
    sample_cubic(bottom, (x1, py(hd4)), (x2, py(y1)), start, 8, &mut pts);
    pts
}

pub(super) fn flow_chart_terminator_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML flowChartTerminator in 21600 space: stadium, rx=3475, ry=10800.
    const STEPS: i32 = 8;
    let rx = w * 3475.0 / 21_600.0;
    let ry = h * 0.5;
    let cy = y + ry;
    let mut pts = vec![(x + rx, y + h), (x + w - rx, y + h)];
    let rcx = x + w - rx;
    for i in 1..=STEPS {
        let t = i as f32 / STEPS as f32;
        let a = std::f32::consts::FRAC_PI_2 * (1.0 - 2.0 * t);
        pts.push((rcx + rx * a.cos(), cy + ry * a.sin()));
    }
    let lcx = x + rx;
    for i in 1..=STEPS {
        let t = i as f32 / STEPS as f32;
        let a = -std::f32::consts::FRAC_PI_2 - std::f32::consts::PI * t;
        pts.push((lcx + rx * a.cos(), cy + ry * a.sin()));
    }
    pts
}

pub(super) fn heptagon_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML heptagon hf=102572 vf=105210.
    let hc = w * 0.5;
    let vc = h * 0.5;
    let swd2 = (w * 0.5) * 102_572.0 / 100_000.0;
    let shd2 = (h * 0.5) * 105_210.0 / 100_000.0;
    let svc = vc * 105_210.0 / 100_000.0;
    let dx1 = swd2 * 97_493.0 / 100_000.0;
    let dx2 = swd2 * 78_183.0 / 100_000.0;
    let dx3 = swd2 * 43_388.0 / 100_000.0;
    let dy1 = shd2 * 62_349.0 / 100_000.0;
    let dy2 = shd2 * 22_252.0 / 100_000.0;
    let dy3 = shd2 * 90_097.0 / 100_000.0;
    let x1 = hc - dx1;
    let x2 = hc - dx2;
    let x3 = hc - dx3;
    let x4 = hc + dx3;
    let x5 = hc + dx2;
    let x6 = hc + dx1;
    let y1 = svc - dy1;
    let y2 = svc + dy2;
    let y3 = svc + dy3;
    let py = |yd: f32| y + h - yd;
    vec![
        (x + x1, py(y2)),
        (x + x2, py(y1)),
        (x + hc, py(0.0)),
        (x + x5, py(y1)),
        (x + x6, py(y2)),
        (x + x4, py(y3)),
        (x + x3, py(y3)),
    ]
}

pub(super) fn star6_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML star6 adj=28868 hf=115470; Cos 30°, Sin 60°.
    let a = 28_868.0;
    let hf = 115_470.0;
    let hc = w * 0.5;
    let vc = h * 0.5;
    let hd4 = h * 0.25;
    let swd2 = (w * 0.5) * hf / 100_000.0;
    let dx1 = swd2 * ooxml_ang_rad(1_800_000.0).cos();
    let x1 = hc - dx1;
    let x2 = hc + dx1;
    let y2 = vc + hd4;
    let iwd2 = swd2 * a / 50_000.0;
    let ihd2 = (h * 0.5) * a / 50_000.0;
    let sdx2 = iwd2 * 0.5;
    let sx1 = hc - iwd2;
    let sx2 = hc - sdx2;
    let sx3 = hc + sdx2;
    let sx4 = hc + iwd2;
    let sdy1 = ihd2 * ooxml_ang_rad(3_600_000.0).sin();
    let sy1 = vc - sdy1;
    let sy2 = vc + sdy1;
    let py = |yd: f32| y + h - yd;
    vec![
        (x + x1, py(hd4)),
        (x + sx2, py(sy1)),
        (x + hc, py(0.0)),
        (x + sx3, py(sy1)),
        (x + x2, py(hd4)),
        (x + sx4, py(vc)),
        (x + x2, py(y2)),
        (x + sx3, py(sy2)),
        (x + hc, py(h)),
        (x + sx2, py(sy2)),
        (x + x1, py(y2)),
        (x + sx1, py(vc)),
    ]
}

pub(super) fn star7_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML star7 adj=34601 hf=102572 vf=105210: 7 tips + 7 inner vertices.
    let a = 34_601.0;
    let hf = 102_572.0;
    let vf = 105_210.0;
    let hc = w * 0.5;
    let vc = h * 0.5;
    let swd2 = (w * 0.5) * hf / 100_000.0;
    let shd2 = (h * 0.5) * vf / 100_000.0;
    let svc = vc * vf / 100_000.0;
    let dx1 = swd2 * 97_493.0 / 100_000.0;
    let dx2 = swd2 * 78_183.0 / 100_000.0;
    let dx3 = swd2 * 43_388.0 / 100_000.0;
    let dy1 = shd2 * 62_349.0 / 100_000.0;
    let dy2 = shd2 * 22_252.0 / 100_000.0;
    let dy3 = shd2 * 90_097.0 / 100_000.0;
    let x1 = hc - dx1;
    let x2 = hc - dx2;
    let x3 = hc - dx3;
    let x4 = hc + dx3;
    let x5 = hc + dx2;
    let x6 = hc + dx1;
    let y1 = svc - dy1;
    let y2v = svc + dy2;
    let y3 = svc + dy3;
    let iwd2 = swd2 * a / 50_000.0;
    let ihd2 = shd2 * a / 50_000.0;
    let sdx1 = iwd2 * 97_493.0 / 100_000.0;
    let sdx2 = iwd2 * 78_183.0 / 100_000.0;
    let sdx3 = iwd2 * 43_388.0 / 100_000.0;
    let sx1 = hc - sdx1;
    let sx2 = hc - sdx2;
    let sx3 = hc - sdx3;
    let sx4 = hc + sdx3;
    let sx5 = hc + sdx2;
    let sx6 = hc + sdx1;
    let sdy1 = ihd2 * 90_097.0 / 100_000.0;
    let sdy2 = ihd2 * 22_252.0 / 100_000.0;
    let sdy3 = ihd2 * 62_349.0 / 100_000.0;
    let sy1 = svc - sdy1;
    let sy2 = svc - sdy2;
    let sy3 = svc + sdy3;
    let sy4 = svc + ihd2;
    let py = |yd: f32| y + h - yd;
    vec![
        (x + x1, py(y2v)),
        (x + sx1, py(sy2)),
        (x + x2, py(y1)),
        (x + sx3, py(sy1)),
        (x + hc, py(0.0)),
        (x + sx4, py(sy1)),
        (x + x5, py(y1)),
        (x + sx6, py(sy2)),
        (x + x6, py(y2v)),
        (x + sx5, py(sy3)),
        (x + x4, py(y3)),
        (x + hc, py(sy4)),
        (x + x3, py(y3)),
        (x + sx2, py(sy3)),
    ]
}

pub(super) fn star8_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML star8 adj=37500: 8 tips + 8 inner vertices. Cos/Sin 2700000 = 45°.
    let a = 37_500.0;
    let hc = w * 0.5;
    let vc = h * 0.5;
    let ang = ooxml_ang_rad(2_700_000.0);
    let dx1 = hc * ang.cos();
    let dy1 = vc * ang.sin();
    let x1 = hc - dx1;
    let x2 = hc + dx1;
    let y1 = vc - dy1;
    let y2 = vc + dy1;
    let iwd2 = hc * a / 50_000.0;
    let ihd2 = vc * a / 50_000.0;
    let sdx1 = iwd2 * 92_388.0 / 100_000.0;
    let sdx2 = iwd2 * 38_268.0 / 100_000.0;
    let sdy1 = ihd2 * 92_388.0 / 100_000.0;
    let sdy2 = ihd2 * 38_268.0 / 100_000.0;
    let sx1 = hc - sdx1;
    let sx2 = hc - sdx2;
    let sx3 = hc + sdx2;
    let sx4 = hc + sdx1;
    let sy1 = vc - sdy1;
    let sy2 = vc - sdy2;
    let sy3 = vc + sdy2;
    let sy4 = vc + sdy1;
    let py = |yd: f32| y + h - yd;
    vec![
        (x, py(vc)),
        (x + sx1, py(sy2)),
        (x + x1, py(y1)),
        (x + sx2, py(sy1)),
        (x + hc, py(0.0)),
        (x + sx3, py(sy1)),
        (x + x2, py(y1)),
        (x + sx4, py(sy2)),
        (x + w, py(vc)),
        (x + sx4, py(sy3)),
        (x + x2, py(y2)),
        (x + sx3, py(sy4)),
        (x + hc, py(h)),
        (x + sx2, py(sy4)),
        (x + x1, py(y2)),
        (x + sx1, py(sy3)),
    ]
}

pub(super) fn star10_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML star10 adj=42533 hf=105146: 10 tips + 10 inner vertices.
    let a = 42_533.0;
    let hf = 105_146.0;
    let hc = w * 0.5;
    let vc = h * 0.5;
    let swd2 = hc * hf / 100_000.0;
    let dx1 = swd2 * 95_106.0 / 100_000.0;
    let dx2 = swd2 * 58_779.0 / 100_000.0;
    let x1 = hc - dx1;
    let x2 = hc - dx2;
    let x3 = hc + dx2;
    let x4 = hc + dx1;
    let dy1 = vc * 80_902.0 / 100_000.0;
    let dy2 = vc * 30_902.0 / 100_000.0;
    let y1 = vc - dy1;
    let y2 = vc - dy2;
    let y3 = vc + dy2;
    let y4 = vc + dy1;
    let iwd2 = swd2 * a / 50_000.0;
    let ihd2 = vc * a / 50_000.0;
    let sdx1 = iwd2 * 80_902.0 / 100_000.0;
    let sdx2 = iwd2 * 30_902.0 / 100_000.0;
    let sdy1 = ihd2 * 95_106.0 / 100_000.0;
    let sdy2 = ihd2 * 58_779.0 / 100_000.0;
    let sx1 = hc - iwd2;
    let sx2 = hc - sdx1;
    let sx3 = hc - sdx2;
    let sx4 = hc + sdx2;
    let sx5 = hc + sdx1;
    let sx6 = hc + iwd2;
    let sy1 = vc - sdy1;
    let sy2 = vc - sdy2;
    let sy3 = vc + sdy2;
    let sy4 = vc + sdy1;
    let py = |yd: f32| y + h - yd;
    vec![
        (x + x1, py(y2)),
        (x + sx2, py(sy2)),
        (x + x2, py(y1)),
        (x + sx3, py(sy1)),
        (x + hc, py(0.0)),
        (x + sx4, py(sy1)),
        (x + x3, py(y1)),
        (x + sx5, py(sy2)),
        (x + x4, py(y2)),
        (x + sx6, py(vc)),
        (x + x4, py(y3)),
        (x + sx5, py(sy3)),
        (x + x3, py(y4)),
        (x + sx4, py(sy4)),
        (x + hc, py(h)),
        (x + sx3, py(sy4)),
        (x + x2, py(y4)),
        (x + sx2, py(sy3)),
        (x + x1, py(y3)),
        (x + sx1, py(vc)),
    ]
}

pub(super) fn star12_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML star12 adj=37500: 12 tips + 12 inner vertices.
    let a = 37_500.0;
    let hc = w * 0.5;
    let vc = h * 0.5;
    let wd4 = w * 0.25;
    let hd4 = h * 0.25;
    let dx1 = hc * ooxml_ang_rad(1_800_000.0).cos();
    let dy1 = vc * ooxml_ang_rad(3_600_000.0).sin();
    let x1 = hc - dx1;
    let x3 = w * 0.75;
    let x4 = hc + dx1;
    let y1 = vc - dy1;
    let y3 = h * 0.75;
    let y4 = vc + dy1;
    let iwd2 = hc * a / 50_000.0;
    let ihd2 = vc * a / 50_000.0;
    let sdx1 = iwd2 * ooxml_ang_rad(900_000.0).cos();
    let sdx2 = iwd2 * ooxml_ang_rad(2_700_000.0).cos();
    let sdx3 = iwd2 * ooxml_ang_rad(4_500_000.0).cos();
    let sdy1 = ihd2 * ooxml_ang_rad(4_500_000.0).sin();
    let sdy2 = ihd2 * ooxml_ang_rad(2_700_000.0).sin();
    let sdy3 = ihd2 * ooxml_ang_rad(900_000.0).sin();
    let sx1 = hc - sdx1;
    let sx2 = hc - sdx2;
    let sx3 = hc - sdx3;
    let sx4 = hc + sdx3;
    let sx5 = hc + sdx2;
    let sx6 = hc + sdx1;
    let sy1 = vc - sdy1;
    let sy2 = vc - sdy2;
    let sy3 = vc - sdy3;
    let sy4 = vc + sdy3;
    let sy5 = vc + sdy2;
    let sy6 = vc + sdy1;
    let py = |yd: f32| y + h - yd;
    vec![
        (x, py(vc)),
        (x + sx1, py(sy3)),
        (x + x1, py(hd4)),
        (x + sx2, py(sy2)),
        (x + wd4, py(y1)),
        (x + sx3, py(sy1)),
        (x + hc, py(0.0)),
        (x + sx4, py(sy1)),
        (x + x3, py(y1)),
        (x + sx5, py(sy2)),
        (x + x4, py(hd4)),
        (x + sx6, py(sy3)),
        (x + w, py(vc)),
        (x + sx6, py(sy4)),
        (x + x4, py(y3)),
        (x + sx5, py(sy5)),
        (x + x3, py(y4)),
        (x + sx4, py(sy6)),
        (x + hc, py(h)),
        (x + sx3, py(sy6)),
        (x + wd4, py(y4)),
        (x + sx2, py(sy5)),
        (x + x1, py(y3)),
        (x + sx1, py(sy4)),
    ]
}

pub(super) fn star16_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML star16 adj=37500: 16 tips + 16 inner vertices.
    let a = 37_500.0;
    let hc = w * 0.5;
    let vc = h * 0.5;
    let dx1 = hc * 92_388.0 / 100_000.0;
    let dx2 = hc * 70_711.0 / 100_000.0;
    let dx3 = hc * 38_268.0 / 100_000.0;
    let dy1 = vc * 92_388.0 / 100_000.0;
    let dy2 = vc * 70_711.0 / 100_000.0;
    let dy3 = vc * 38_268.0 / 100_000.0;
    let x1 = hc - dx1;
    let x2 = hc - dx2;
    let x3 = hc - dx3;
    let x4 = hc + dx3;
    let x5 = hc + dx2;
    let x6 = hc + dx1;
    let y1 = vc - dy1;
    let y2 = vc - dy2;
    let y3 = vc - dy3;
    let y4 = vc + dy3;
    let y5 = vc + dy2;
    let y6 = vc + dy1;
    let iwd2 = hc * a / 50_000.0;
    let ihd2 = vc * a / 50_000.0;
    let sdx1 = iwd2 * 98_079.0 / 100_000.0;
    let sdx2 = iwd2 * 83_147.0 / 100_000.0;
    let sdx3 = iwd2 * 55_557.0 / 100_000.0;
    let sdx4 = iwd2 * 19_509.0 / 100_000.0;
    let sdy1 = ihd2 * 98_079.0 / 100_000.0;
    let sdy2 = ihd2 * 83_147.0 / 100_000.0;
    let sdy3 = ihd2 * 55_557.0 / 100_000.0;
    let sdy4 = ihd2 * 19_509.0 / 100_000.0;
    let sx1 = hc - sdx1;
    let sx2 = hc - sdx2;
    let sx3 = hc - sdx3;
    let sx4 = hc - sdx4;
    let sx5 = hc + sdx4;
    let sx6 = hc + sdx3;
    let sx7 = hc + sdx2;
    let sx8 = hc + sdx1;
    let sy1 = vc - sdy1;
    let sy2 = vc - sdy2;
    let sy3 = vc - sdy3;
    let sy4 = vc - sdy4;
    let sy5 = vc + sdy4;
    let sy6 = vc + sdy3;
    let sy7 = vc + sdy2;
    let sy8 = vc + sdy1;
    let py = |yd: f32| y + h - yd;
    vec![
        (x, py(vc)),
        (x + sx1, py(sy4)),
        (x + x1, py(y3)),
        (x + sx2, py(sy3)),
        (x + x2, py(y2)),
        (x + sx3, py(sy2)),
        (x + x3, py(y1)),
        (x + sx4, py(sy1)),
        (x + hc, py(0.0)),
        (x + sx5, py(sy1)),
        (x + x4, py(y1)),
        (x + sx6, py(sy2)),
        (x + x5, py(y2)),
        (x + sx7, py(sy3)),
        (x + x6, py(y3)),
        (x + sx8, py(sy4)),
        (x + w, py(vc)),
        (x + sx8, py(sy5)),
        (x + x6, py(y4)),
        (x + sx7, py(sy6)),
        (x + x5, py(y5)),
        (x + sx6, py(sy7)),
        (x + x4, py(y6)),
        (x + sx5, py(sy8)),
        (x + hc, py(h)),
        (x + sx4, py(sy8)),
        (x + x3, py(y6)),
        (x + sx3, py(sy7)),
        (x + x2, py(y5)),
        (x + sx2, py(sy6)),
        (x + x1, py(y4)),
        (x + sx1, py(sy5)),
    ]
}

pub(super) fn star24_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML star24 adj=37500: 24 tips + 24 inner vertices.
    let a = 37_500.0;
    let hc = w * 0.5;
    let vc = h * 0.5;
    let dx1 = hc * ooxml_ang_rad(900_000.0).cos();
    let dx2 = hc * ooxml_ang_rad(1_800_000.0).cos();
    let dx3 = hc * ooxml_ang_rad(2_700_000.0).cos();
    let dx4 = w * 0.25;
    let dx5 = hc * ooxml_ang_rad(4_500_000.0).cos();
    let dy1 = vc * ooxml_ang_rad(4_500_000.0).sin();
    let dy2 = vc * ooxml_ang_rad(3_600_000.0).sin();
    let dy3 = vc * ooxml_ang_rad(2_700_000.0).sin();
    let dy4 = h * 0.25;
    let dy5 = vc * ooxml_ang_rad(900_000.0).sin();
    let x1 = hc - dx1;
    let x2 = hc - dx2;
    let x3 = hc - dx3;
    let x4 = hc - dx4;
    let x5 = hc - dx5;
    let x6 = hc + dx5;
    let x7 = hc + dx4;
    let x8 = hc + dx3;
    let x9 = hc + dx2;
    let x10 = hc + dx1;
    let y1 = vc - dy1;
    let y2 = vc - dy2;
    let y3 = vc - dy3;
    let y4 = vc - dy4;
    let y5 = vc - dy5;
    let y6 = vc + dy5;
    let y7 = vc + dy4;
    let y8 = vc + dy3;
    let y9 = vc + dy2;
    let y10 = vc + dy1;
    let iwd2 = hc * a / 50_000.0;
    let ihd2 = vc * a / 50_000.0;
    let sdx1 = iwd2 * 99_144.0 / 100_000.0;
    let sdx2 = iwd2 * 92_388.0 / 100_000.0;
    let sdx3 = iwd2 * 79_335.0 / 100_000.0;
    let sdx4 = iwd2 * 60_876.0 / 100_000.0;
    let sdx5 = iwd2 * 38_268.0 / 100_000.0;
    let sdx6 = iwd2 * 13_053.0 / 100_000.0;
    let sdy1 = ihd2 * 99_144.0 / 100_000.0;
    let sdy2 = ihd2 * 92_388.0 / 100_000.0;
    let sdy3 = ihd2 * 79_335.0 / 100_000.0;
    let sdy4 = ihd2 * 60_876.0 / 100_000.0;
    let sdy5 = ihd2 * 38_268.0 / 100_000.0;
    let sdy6 = ihd2 * 13_053.0 / 100_000.0;
    let sx1 = hc - sdx1;
    let sx2 = hc - sdx2;
    let sx3 = hc - sdx3;
    let sx4 = hc - sdx4;
    let sx5 = hc - sdx5;
    let sx6 = hc - sdx6;
    let sx7 = hc + sdx6;
    let sx8 = hc + sdx5;
    let sx9 = hc + sdx4;
    let sx10 = hc + sdx3;
    let sx11 = hc + sdx2;
    let sx12 = hc + sdx1;
    let sy1 = vc - sdy1;
    let sy2 = vc - sdy2;
    let sy3 = vc - sdy3;
    let sy4 = vc - sdy4;
    let sy5 = vc - sdy5;
    let sy6 = vc - sdy6;
    let sy7 = vc + sdy6;
    let sy8 = vc + sdy5;
    let sy9 = vc + sdy4;
    let sy10 = vc + sdy3;
    let sy11 = vc + sdy2;
    let sy12 = vc + sdy1;
    let py = |yd: f32| y + h - yd;
    vec![
        (x, py(vc)),
        (x + sx1, py(sy6)),
        (x + x1, py(y5)),
        (x + sx2, py(sy5)),
        (x + x2, py(y4)),
        (x + sx3, py(sy4)),
        (x + x3, py(y3)),
        (x + sx4, py(sy3)),
        (x + x4, py(y2)),
        (x + sx5, py(sy2)),
        (x + x5, py(y1)),
        (x + sx6, py(sy1)),
        (x + hc, py(0.0)),
        (x + sx7, py(sy1)),
        (x + x6, py(y1)),
        (x + sx8, py(sy2)),
        (x + x7, py(y2)),
        (x + sx9, py(sy3)),
        (x + x8, py(y3)),
        (x + sx10, py(sy4)),
        (x + x9, py(y4)),
        (x + sx11, py(sy5)),
        (x + x10, py(y5)),
        (x + sx12, py(sy6)),
        (x + w, py(vc)),
        (x + sx12, py(sy7)),
        (x + x10, py(y6)),
        (x + sx11, py(sy8)),
        (x + x9, py(y7)),
        (x + sx10, py(sy9)),
        (x + x8, py(y8)),
        (x + sx9, py(sy10)),
        (x + x7, py(y9)),
        (x + sx8, py(sy11)),
        (x + x6, py(y10)),
        (x + sx7, py(sy12)),
        (x + hc, py(h)),
        (x + sx6, py(sy12)),
        (x + x5, py(y10)),
        (x + sx5, py(sy11)),
        (x + x4, py(y9)),
        (x + sx4, py(sy10)),
        (x + x3, py(y8)),
        (x + sx3, py(sy9)),
        (x + x2, py(y7)),
        (x + sx2, py(sy8)),
        (x + x1, py(y6)),
        (x + sx1, py(sy7)),
    ]
}

pub(super) fn star32_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML star32 adj=37500: 32 tips + 32 inner vertices.
    let a = 37_500.0;
    let hc = w * 0.5;
    let vc = h * 0.5;
    let dx1 = hc * 98_079.0 / 100_000.0;
    let dx2 = hc * 92_388.0 / 100_000.0;
    let dx3 = hc * 83_147.0 / 100_000.0;
    let dx4 = hc * ooxml_ang_rad(2_700_000.0).cos();
    let dx5 = hc * 55_557.0 / 100_000.0;
    let dx6 = hc * 38_268.0 / 100_000.0;
    let dx7 = hc * 19_509.0 / 100_000.0;
    let dy1 = vc * 98_079.0 / 100_000.0;
    let dy2 = vc * 92_388.0 / 100_000.0;
    let dy3 = vc * 83_147.0 / 100_000.0;
    let dy4 = vc * ooxml_ang_rad(2_700_000.0).sin();
    let dy5 = vc * 55_557.0 / 100_000.0;
    let dy6 = vc * 38_268.0 / 100_000.0;
    let dy7 = vc * 19_509.0 / 100_000.0;
    let x1 = hc - dx1;
    let x2 = hc - dx2;
    let x3 = hc - dx3;
    let x4 = hc - dx4;
    let x5 = hc - dx5;
    let x6 = hc - dx6;
    let x7 = hc - dx7;
    let x8 = hc + dx7;
    let x9 = hc + dx6;
    let x10 = hc + dx5;
    let x11 = hc + dx4;
    let x12 = hc + dx3;
    let x13 = hc + dx2;
    let x14 = hc + dx1;
    let y1 = vc - dy1;
    let y2 = vc - dy2;
    let y3 = vc - dy3;
    let y4 = vc - dy4;
    let y5 = vc - dy5;
    let y6 = vc - dy6;
    let y7 = vc - dy7;
    let y8 = vc + dy7;
    let y9 = vc + dy6;
    let y10 = vc + dy5;
    let y11 = vc + dy4;
    let y12 = vc + dy3;
    let y13 = vc + dy2;
    let y14 = vc + dy1;
    let iwd2 = hc * a / 50_000.0;
    let ihd2 = vc * a / 50_000.0;
    let sdx1 = iwd2 * 99_518.0 / 100_000.0;
    let sdx2 = iwd2 * 95_694.0 / 100_000.0;
    let sdx3 = iwd2 * 88_192.0 / 100_000.0;
    let sdx4 = iwd2 * 77_301.0 / 100_000.0;
    let sdx5 = iwd2 * 63_439.0 / 100_000.0;
    let sdx6 = iwd2 * 47_140.0 / 100_000.0;
    let sdx7 = iwd2 * 29_028.0 / 100_000.0;
    let sdx8 = iwd2 * 9_802.0 / 100_000.0;
    let sdy1 = ihd2 * 99_518.0 / 100_000.0;
    let sdy2 = ihd2 * 95_694.0 / 100_000.0;
    let sdy3 = ihd2 * 88_192.0 / 100_000.0;
    let sdy4 = ihd2 * 77_301.0 / 100_000.0;
    let sdy5 = ihd2 * 63_439.0 / 100_000.0;
    let sdy6 = ihd2 * 47_140.0 / 100_000.0;
    let sdy7 = ihd2 * 29_028.0 / 100_000.0;
    let sdy8 = ihd2 * 9_802.0 / 100_000.0;
    let sx1 = hc - sdx1;
    let sx2 = hc - sdx2;
    let sx3 = hc - sdx3;
    let sx4 = hc - sdx4;
    let sx5 = hc - sdx5;
    let sx6 = hc - sdx6;
    let sx7 = hc - sdx7;
    let sx8 = hc - sdx8;
    let sx9 = hc + sdx8;
    let sx10 = hc + sdx7;
    let sx11 = hc + sdx6;
    let sx12 = hc + sdx5;
    let sx13 = hc + sdx4;
    let sx14 = hc + sdx3;
    let sx15 = hc + sdx2;
    let sx16 = hc + sdx1;
    let sy1 = vc - sdy1;
    let sy2 = vc - sdy2;
    let sy3 = vc - sdy3;
    let sy4 = vc - sdy4;
    let sy5 = vc - sdy5;
    let sy6 = vc - sdy6;
    let sy7 = vc - sdy7;
    let sy8 = vc - sdy8;
    let sy9 = vc + sdy8;
    let sy10 = vc + sdy7;
    let sy11 = vc + sdy6;
    let sy12 = vc + sdy5;
    let sy13 = vc + sdy4;
    let sy14 = vc + sdy3;
    let sy15 = vc + sdy2;
    let sy16 = vc + sdy1;
    let py = |yd: f32| y + h - yd;
    vec![
        (x, py(vc)),
        (x + sx1, py(sy8)),
        (x + x1, py(y7)),
        (x + sx2, py(sy7)),
        (x + x2, py(y6)),
        (x + sx3, py(sy6)),
        (x + x3, py(y5)),
        (x + sx4, py(sy5)),
        (x + x4, py(y4)),
        (x + sx5, py(sy4)),
        (x + x5, py(y3)),
        (x + sx6, py(sy3)),
        (x + x6, py(y2)),
        (x + sx7, py(sy2)),
        (x + x7, py(y1)),
        (x + sx8, py(sy1)),
        (x + hc, py(0.0)),
        (x + sx9, py(sy1)),
        (x + x8, py(y1)),
        (x + sx10, py(sy2)),
        (x + x9, py(y2)),
        (x + sx11, py(sy3)),
        (x + x10, py(y3)),
        (x + sx12, py(sy4)),
        (x + x11, py(y4)),
        (x + sx13, py(sy5)),
        (x + x12, py(y5)),
        (x + sx14, py(sy6)),
        (x + x13, py(y6)),
        (x + sx15, py(sy7)),
        (x + x14, py(y7)),
        (x + sx16, py(sy8)),
        (x + w, py(vc)),
        (x + sx16, py(sy9)),
        (x + x14, py(y8)),
        (x + sx15, py(sy10)),
        (x + x13, py(y9)),
        (x + sx14, py(sy11)),
        (x + x12, py(y10)),
        (x + sx13, py(sy12)),
        (x + x11, py(y11)),
        (x + sx12, py(sy13)),
        (x + x10, py(y12)),
        (x + sx11, py(sy14)),
        (x + x9, py(y13)),
        (x + sx10, py(sy15)),
        (x + x8, py(y14)),
        (x + sx9, py(sy16)),
        (x + hc, py(h)),
        (x + sx8, py(sy16)),
        (x + x7, py(y14)),
        (x + sx7, py(sy15)),
        (x + x6, py(y13)),
        (x + sx6, py(sy14)),
        (x + x5, py(y12)),
        (x + sx5, py(sy13)),
        (x + x4, py(y11)),
        (x + sx4, py(sy12)),
        (x + x3, py(y10)),
        (x + sx3, py(sy11)),
        (x + x2, py(y9)),
        (x + sx2, py(sy10)),
        (x + x1, py(y8)),
        (x + sx1, py(sy9)),
    ]
}

pub(super) fn flow_chart_document_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML flowChartDocument in 21600 space: rectangle with a cubic
    // wave along the bottom (ctrl2 y=23922 hangs below b).
    let y1 = h * 17_322.0 / 21_600.0;
    let y2 = h * 20_172.0 / 21_600.0;
    let y3 = h * 23_922.0 / 21_600.0;
    let hc = w * 0.5;
    let py = |yd: f32| y + h - yd;
    let p0 = (x, py(0.0));
    let p1 = (x + w, py(0.0));
    let p2 = (x + w, py(y1));
    let p3 = (x, py(y2));
    let mut pts = vec![p0, p1, p2];
    sample_cubic(p2, (x + hc, py(y1)), (x + hc, py(y3)), p3, 8, &mut pts);
    pts
}

pub(super) fn flow_chart_offpage_connector_points(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) -> Vec<(f32, f32)> {
    // OOXML flowChartOffpageConnector in 10×10 space: rectangle with a
    // downward V at the bottom (y1 = 4/5 h).
    let y1 = h * 0.8;
    let py = |yd: f32| y + h - yd;
    vec![
        (x, py(0.0)),
        (x + w, py(0.0)),
        (x + w, py(y1)),
        (x + w * 0.5, py(h)),
        (x, py(y1)),
    ]
}

pub(super) fn flow_chart_delay_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML flowChartDelay: M l,t L hc,t arcTo wd2,hd2 stAng=3cd4 swAng=cd2 L l,b Z.
    const ST: f32 = 16_200_000.0;
    const SW: f32 = 10_800_000.0;
    let hc = w * 0.5;
    let wr = (w * 0.5).max(0.5);
    let hr = (h * 0.5).max(0.5);
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    let mut pts = vec![map(0.0, 0.0), map(hc, 0.0)];
    let mut cur = (hc, 0.0);
    ooxml_arc_to_y_down(&mut cur, wr, hr, ST, SW, &mut pts, map);
    pts.push(map(0.0, h));
    pts
}

pub(super) fn flow_chart_manual_input_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML flowChartManualInput in 5×5 space: slanted top (l,hd5)→(r,t).
    let py = |yd: f32| y + h - yd;
    vec![
        (x, py(h * 0.2)),
        (x + w, py(0.0)),
        (x + w, py(h)),
        (x, py(h)),
    ]
}

pub(super) fn flow_chart_punched_card_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML flowChartPunchedCard in 5×5 space: top-left corner cut.
    let py = |yd: f32| y + h - yd;
    vec![
        (x, py(h * 0.2)),
        (x + w * 0.2, py(0.0)),
        (x + w, py(0.0)),
        (x + w, py(h)),
        (x, py(h)),
    ]
}

pub(super) fn flow_chart_preparation_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML flowChartPreparation in 10×10 space: hexagon, x2=4/5 w.
    let py = |yd: f32| y + h - yd;
    vec![
        (x, py(h * 0.5)),
        (x + w * 0.2, py(0.0)),
        (x + w * 0.8, py(0.0)),
        (x + w, py(h * 0.5)),
        (x + w * 0.8, py(h)),
        (x + w * 0.2, py(h)),
    ]
}

pub(super) fn flow_chart_extract_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML flowChartExtract in 2×2 space: up-pointing triangle.
    let py = |yd: f32| y + h - yd;
    vec![(x, py(h)), (x + w * 0.5, py(0.0)), (x + w, py(h))]
}

pub(super) fn flow_chart_merge_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML flowChartMerge in 2×2 space: down-pointing triangle.
    let py = |yd: f32| y + h - yd;
    vec![(x, py(0.0)), (x + w, py(0.0)), (x + w * 0.5, py(h))]
}

pub(super) fn flow_chart_collate_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML flowChartCollate in 2×2 space: hourglass, waist at (hc,vc).
    let py = |yd: f32| y + h - yd;
    let waist = (x + w * 0.5, py(h * 0.5));
    vec![
        (x, py(0.0)),
        (x + w, py(0.0)),
        waist,
        (x + w, py(h)),
        (x, py(h)),
        waist,
    ]
}

pub(super) fn double_wave_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML doubleWave adj1=6250 adj2=0: two cubics on top, two on bottom.
    let y1 = h * 6_250.0 / 100_000.0;
    let dy2 = y1 * 10.0 / 3.0;
    let y2 = y1 - dy2;
    let y3 = y1 + dy2;
    let y4 = h - y1;
    let y5 = y4 - dy2;
    let y6 = y4 + dy2;
    let py = |yd: f32| y + h - yd;
    let x3 = x + w / 6.0;
    let x4 = x + w / 3.0;
    let x5 = x + w * 0.5;
    let x6 = x + w * 2.0 / 3.0;
    let x7 = x + w * 5.0 / 6.0;
    let x8 = x + w;
    let p0 = (x, py(y1));
    let p1 = (x5, py(y1));
    let p2 = (x8, py(y1));
    let p3 = (x8, py(y4));
    let p4 = (x5, py(y4));
    let p5 = (x, py(y4));
    let mut pts = vec![p0];
    sample_cubic(p0, (x3, py(y2)), (x4, py(y3)), p1, 6, &mut pts);
    sample_cubic(p1, (x6, py(y2)), (x7, py(y3)), p2, 6, &mut pts);
    pts.push(p3);
    sample_cubic(p3, (x7, py(y6)), (x6, py(y5)), p4, 6, &mut pts);
    sample_cubic(p4, (x4, py(y6)), (x3, py(y5)), p5, 6, &mut pts);
    pts
}

pub(super) fn flow_chart_online_storage_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML flowChartOnlineStorage in 6×6 space: horizontal cylinder,
    // right arc stAng=3cd4 swAng=-cd2, left arc stAng=cd4 swAng=cd2.
    const ST_R: f32 = 16_200_000.0;
    const SW_R: f32 = -10_800_000.0;
    const ST_L: f32 = 5_400_000.0;
    const SW_L: f32 = 10_800_000.0;
    let wr = (w / 6.0).max(0.5);
    let hr = (h * 0.5).max(0.5);
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    let mut pts = vec![map(w / 6.0, 0.0), map(w, 0.0)];
    let mut cur = (w, 0.0);
    ooxml_arc_to_y_down(&mut cur, wr, hr, ST_R, SW_R, &mut pts, map);
    pts.push(map(w / 6.0, h));
    cur = (w / 6.0, h);
    ooxml_arc_to_y_down(&mut cur, wr, hr, ST_L, SW_L, &mut pts, map);
    pts
}

pub(super) fn flow_chart_manual_operation_points(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) -> Vec<(f32, f32)> {
    // OOXML flowChartManualOperation in 5×5 space: trapezoid (l,t)→(r,t)→(4/5 w,b)→(wd5,b).
    let py = |yd: f32| y + h - yd;
    vec![
        (x, py(0.0)),
        (x + w, py(0.0)),
        (x + w * 0.8, py(h)),
        (x + w * 0.2, py(h)),
    ]
}

pub(super) fn flow_chart_input_output_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML flowChartInputOutput in 5×5 space: parallelogram (l,b)→(wd5,t)→(r,t)→(4/5 w,b).
    let py = |yd: f32| y + h - yd;
    vec![
        (x, py(h)),
        (x + w * 0.2, py(0.0)),
        (x + w, py(0.0)),
        (x + w * 0.8, py(h)),
    ]
}

pub(super) fn flow_chart_display_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML flowChartDisplay in 6×6 space: pointed left, right semicircle.
    const ST: f32 = 16_200_000.0;
    const SW: f32 = 10_800_000.0;
    let wr = (w / 6.0).max(0.5);
    let hr = (h * 0.5).max(0.5);
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    let mut pts = vec![
        map(0.0, h * 0.5),
        map(w / 6.0, 0.0),
        map(w * 5.0 / 6.0, 0.0),
    ];
    let mut cur = (w * 5.0 / 6.0, 0.0);
    ooxml_arc_to_y_down(&mut cur, wr, hr, ST, SW, &mut pts, map);
    pts.push(map(w / 6.0, h));
    pts
}

fn ooxml_arc_to_y_down(
    cur: &mut (f32, f32),
    wr: f32,
    hr: f32,
    st_ang: f32,
    sw_ang: f32,
    pts: &mut Vec<(f32, f32)>,
    map: impl Fn(f32, f32) -> (f32, f32),
) {
    let st = ooxml_ang_rad(st_ang);
    let sw = ooxml_ang_rad(sw_ang);
    let cx = cur.0 - wr * st.cos();
    let cy = cur.1 - hr * st.sin();
    let n = ((sw.abs() / std::f32::consts::FRAC_PI_2).ceil() as i32 * 4).max(4);
    for i in 1..=n {
        let a = st + sw * i as f32 / n as f32;
        *cur = (cx + wr * a.cos(), cy + hr * a.sin());
        pts.push(map(cur.0, cur.1));
    }
}

pub(super) fn cloud_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML cloud path w=h=43200: 11 arcTo lobes, then close.
    const PW: f32 = 43_200.0;
    let map = |ox: f32, oy: f32| (x + ox * w / PW, y + h - oy * h / PW);
    let mut cur = (3_900.0, 14_370.0);
    let mut pts = vec![map(cur.0, cur.1)];
    const ARCS: [(f32, f32, f32, f32); 11] = [
        (6_753.0, 9_190.0, -11_429_249.0, 7_426_832.0),
        (5_333.0, 7_267.0, -8_646_143.0, 5_396_714.0),
        (4_365.0, 5_945.0, -8_748_475.0, 5_983_381.0),
        (4_857.0, 6_595.0, -7_859_164.0, 7_034_504.0),
        (5_333.0, 7_273.0, -4_722_533.0, 6_541_615.0),
        (6_775.0, 9_220.0, -2_776_035.0, 7_816_140.0),
        (5_785.0, 7_867.0, 37_501.0, 6_842_000.0),
        (6_752.0, 9_215.0, 1_347_096.0, 6_910_353.0),
        (7_720.0, 10_543.0, 3_974_558.0, 4_542_661.0),
        (4_360.0, 5_918.0, -16_496_525.0, 8_804_134.0),
        (4_345.0, 5_945.0, -14_809_710.0, 9_151_131.0),
    ];
    for (wr, hr, st, sw) in ARCS {
        ooxml_arc_to_y_down(&mut cur, wr, hr, st, sw, &mut pts, map);
    }
    pts
}

pub(super) fn radar_chart_polygon_points(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    values: &[f32],
    axis_max: f32,
) -> Vec<(f32, f32)> {
    let n = values.len();
    if n < 3 {
        return Vec::new();
    }
    let axis_max = axis_max.max(1.0);
    let cx = x + w * 0.5;
    let cy = y + h * 0.5;
    let r = w.min(h) * 0.5;
    values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let frac = v.max(0.0) / axis_max;
            let deg = 90.0 - (i as f32) * 360.0 / n as f32;
            let a = deg.to_radians();
            (cx + r * frac * a.cos(), cy + r * frac * a.sin())
        })
        .collect()
}

/// Per series, `(x, y)` pairs matched by `c:pt/@idx`. A series without
/// its own `c:xVal` takes 1, 2, 3… (Excel's category index).
pub(super) fn scatter_series_pairs(chart: &ChartData) -> Vec<Vec<(f32, f32)>> {
    chart
        .indexed
        .iter()
        .enumerate()
        .map(|(si, ys)| {
            let xs = chart.x_indexed.get(si).filter(|x| !x.is_empty());
            ys.iter()
                .filter_map(|&(i, yv)| {
                    let xv = match xs {
                        Some(xs) => xs.iter().find(|p| p.0 == i)?.1,
                        None => i as f32 + 1.0,
                    };
                    Some((xv, yv))
                })
                .collect()
        })
        .collect()
}

/// Word's scatter value axis: explicit `c:scaling` min/max, else
/// `min(0, data_min)` .. `max(1, data_max)`; a zero span widens by 1.
pub(super) fn scatter_axis_bounds(
    values: impl Iterator<Item = f32>,
    scaling: (Option<f32>, Option<f32>),
) -> (f32, f32) {
    let (lo, hi) = values.fold((0.0_f32, 1.0_f32), |(lo, hi), v| (lo.min(v), hi.max(v)));
    let lo = scaling.0.unwrap_or(lo);
    let hi = scaling.1.unwrap_or(hi);
    if hi - lo > f32::EPSILON {
        (lo, hi)
    } else {
        (lo, lo + 1.0)
    }
}

pub(super) fn scatter_chart_marker_points(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    pts: &[(f32, f32)],
    (x_lo, x_hi): (f32, f32),
    (y_lo, y_hi): (f32, f32),
) -> Vec<(f32, f32)> {
    pts.iter()
        .map(|(xv, yv)| {
            (
                x + (xv - x_lo) / (x_hi - x_lo) * w,
                y + (yv - y_lo) / (y_hi - y_lo) * h,
            )
        })
        .collect()
}

pub(super) fn area_chart_fill_points(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    values: &[f32],
    axis_max: f32,
) -> Vec<(f32, f32)> {
    let mut pts = line_chart_polyline_points(x, y, w, h, values, axis_max);
    if pts.len() < 2 {
        return pts;
    }
    let first_x = pts[0].0;
    let last_x = pts[pts.len() - 1].0;
    pts.push((last_x, y));
    pts.push((first_x, y));
    pts
}

/// A line series' points in category order, split where an idx is
/// missing (Word's default `dispBlanksAs="gap"`), each within `slots`.
pub(super) fn line_chart_runs(points: &[(usize, f32)], slots: usize) -> Vec<Vec<(usize, f32)>> {
    let mut pts: Vec<(usize, f32)> = points.iter().copied().filter(|p| p.0 < slots).collect();
    pts.sort_by_key(|p| p.0);
    let mut runs: Vec<Vec<(usize, f32)>> = Vec::new();
    for p in pts {
        match runs.last_mut() {
            Some(run) if run.last().is_some_and(|q| q.0 + 1 == p.0) => run.push(p),
            _ => runs.push(vec![p]),
        }
    }
    runs
}

pub(super) fn line_chart_polyline_points(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    values: &[f32],
    axis_max: f32,
) -> Vec<(f32, f32)> {
    let n = values.len() as f32;
    if n < 1.0 {
        return Vec::new();
    }
    let axis_max = axis_max.max(1.0);
    values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let px = x + (i as f32 + 0.5) / n * w;
            let py = y + (v.max(0.0) / axis_max) * h;
            (px, py)
        })
        .collect()
}

pub(super) fn pie_chart_wedge_points(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    st_ang: f32,
    sw_ang: f32,
) -> Vec<(f32, f32)> {
    let hc = w * 0.5;
    let vc = h * 0.5;
    let wr = hc.max(0.5);
    let hr = vc.max(0.5);
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    let st = ooxml_ang_rad(st_ang);
    let mut cur = (hc + wr * st.cos(), vc + hr * st.sin());
    let mut pts = vec![map(cur.0, cur.1)];
    ooxml_arc_to_y_down(&mut cur, wr, hr, st_ang, sw_ang, &mut pts, map);
    pts.push(map(hc, vc));
    pts
}

pub(super) fn pie_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML pie adj1=0 adj2=16200000: 270° wedge from 3 o'clock clockwise.
    let hc = w * 0.5;
    let vc = h * 0.5;
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    let mut cur = (w, vc);
    let mut pts = vec![map(cur.0, cur.1)];
    ooxml_arc_to_y_down(&mut cur, hc, vc, 0.0, 16_200_000.0, &mut pts, map);
    pts.push(map(hc, vc));
    pts
}

pub(super) fn left_right_arrow_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML leftRightArrow adj1=adj2=50000.
    let ss = preset_ss(w, h);
    let x2 = ss * 50_000.0 / 100_000.0;
    let x3 = w - x2;
    let dy = h * 50_000.0 / 200_000.0;
    let vc = h * 0.5;
    let y1 = vc - dy;
    let y2 = vc + dy;
    let py = |yd: f32| y + h - yd;
    vec![
        (x, py(vc)),
        (x + x2, py(0.0)),
        (x + x2, py(y1)),
        (x + x3, py(y1)),
        (x + x3, py(0.0)),
        (x + w, py(vc)),
        (x + x3, py(h)),
        (x + x3, py(y2)),
        (x + x2, py(y2)),
        (x + x2, py(h)),
    ]
}

pub(super) fn quad_arrow_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML quadArrow adj1=adj2=adj3=22500. Locals are the ECMA guide
    // names, which cross the adjustments: x1 = ss*a3 (head length),
    // dx2 = ss*a2 (half head spread), dx3 = ss*a1/2 (half shaft width).
    let ss = preset_ss(w, h);
    let hc = w * 0.5;
    let vc = h * 0.5;
    let x1 = ss * 22_500.0 / 100_000.0;
    let dx2 = ss * 22_500.0 / 100_000.0;
    let dx3 = ss * 22_500.0 / 200_000.0;
    let x2 = hc - dx2;
    let x3 = hc - dx3;
    let x4 = hc + dx3;
    let x5 = hc + dx2;
    let x6 = w - x1;
    let y2 = vc - dx2;
    let y3 = vc - dx3;
    let y4 = vc + dx3;
    let y5 = vc + dx2;
    let y6 = h - x1;
    let py = |yd: f32| y + h - yd;
    vec![
        (x, py(vc)),
        (x + x1, py(y2)),
        (x + x1, py(y3)),
        (x + x3, py(y3)),
        (x + x3, py(x1)),
        (x + x2, py(x1)),
        (x + hc, py(0.0)),
        (x + x5, py(x1)),
        (x + x4, py(x1)),
        (x + x4, py(y3)),
        (x + x6, py(y3)),
        (x + x6, py(y2)),
        (x + w, py(vc)),
        (x + x6, py(y5)),
        (x + x6, py(y4)),
        (x + x4, py(y4)),
        (x + x4, py(y6)),
        (x + x5, py(y6)),
        (x + hc, py(h)),
        (x + x2, py(y6)),
        (x + x3, py(y6)),
        (x + x3, py(y4)),
        (x + x1, py(y4)),
        (x + x1, py(y5)),
    ]
}

pub(super) fn lightning_bolt_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML lightningBolt path w=h=21600.
    const PW: f32 = 21_600.0;
    let map = |ox: f32, oy: f32| (x + ox * w / PW, y + h - oy * h / PW);
    [
        (8_472.0, 0.0),
        (12_860.0, 6_080.0),
        (11_050.0, 6_797.0),
        (16_577.0, 12_007.0),
        (14_767.0, 12_877.0),
        (21_600.0, 21_600.0),
        (10_012.0, 14_915.0),
        (12_222.0, 13_987.0),
        (5_022.0, 9_705.0),
        (7_602.0, 8_382.0),
        (0.0, 3_890.0),
    ]
    .into_iter()
    .map(|(ox, oy)| map(ox, oy))
    .collect()
}

pub(super) fn sun_ray_points(x: f32, y: f32, w: f32, h: f32) -> [Vec<(f32, f32)>; 8] {
    // OOXML sun adj=25000: eight triangular rays.
    let a = 25_000.0;
    let g0 = 50_000.0 - a;
    let g1 = g0 * 30_274.0 / 32_768.0;
    let g2 = g0 * 12_540.0 / 32_768.0;
    let g5 = 50_000.0 - g1;
    let g6 = 50_000.0 - g2;
    let g10 = g5 * 3.0 / 4.0;
    let g11 = g6 * 3.0 / 4.0;
    let g12 = g10 + 3_662.0;
    let g13 = g11 + 3_662.0;
    let g14 = g11 + 12_500.0;
    let g15 = 100_000.0 - g10;
    let g16 = 100_000.0 - g12;
    let g17 = 100_000.0 - g13;
    let g18 = 100_000.0 - g14;
    let ox1 = w * 18_436.0 / 21_600.0;
    let oy1 = h * 3_163.0 / 21_600.0;
    let ox2 = w * 3_163.0 / 21_600.0;
    let oy2 = h * 18_436.0 / 21_600.0;
    let x10 = w * g10 / 100_000.0;
    let x12 = w * g12 / 100_000.0;
    let x13 = w * g13 / 100_000.0;
    let x14 = w * g14 / 100_000.0;
    let x15 = w * g15 / 100_000.0;
    let x16 = w * g16 / 100_000.0;
    let x17 = w * g17 / 100_000.0;
    let x18 = w * g18 / 100_000.0;
    let y10 = h * g10 / 100_000.0;
    let y12 = h * g12 / 100_000.0;
    let y13 = h * g13 / 100_000.0;
    let y14 = h * g14 / 100_000.0;
    let y15 = h * g15 / 100_000.0;
    let y16 = h * g16 / 100_000.0;
    let y17 = h * g17 / 100_000.0;
    let y18 = h * g18 / 100_000.0;
    let hc = w * 0.5;
    let vc = h * 0.5;
    let py = |yd: f32| y + h - yd;
    [
        vec![(x + w, py(vc)), (x + x15, py(y18)), (x + x15, py(y14))],
        vec![(x + ox1, py(oy1)), (x + x16, py(y13)), (x + x17, py(y12))],
        vec![(x + hc, py(0.0)), (x + x18, py(y10)), (x + x14, py(y10))],
        vec![(x + ox2, py(oy1)), (x + x13, py(y12)), (x + x12, py(y13))],
        vec![(x, py(vc)), (x + x10, py(y14)), (x + x10, py(y18))],
        vec![(x + ox2, py(oy2)), (x + x12, py(y17)), (x + x13, py(y16))],
        vec![(x + hc, py(h)), (x + x14, py(y15)), (x + x18, py(y15))],
        vec![(x + ox1, py(oy2)), (x + x17, py(y16)), (x + x16, py(y17))],
    ]
}

pub(super) fn sun_disk_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    let wr = w * 25_000.0 / 100_000.0;
    let hr = h * 25_000.0 / 100_000.0;
    ellipse_points(x + w * 0.5 - wr, y + h * 0.5 - hr, wr * 2.0, hr * 2.0)
}

fn gear_points(x: f32, y: f32, w: f32, h: f32, teeth: i32, adj1: f32) -> Vec<(f32, f32)> {
    // OOXML gear6 adj1=15000 / gear9 adj1=10000; flat teeth, solid (no hub hole).
    let th = preset_ss(w, h) * adj1 / 100_000.0;
    let hc = w * 0.5;
    let vc = h * 0.5;
    let rw = (w * 0.5).max(0.5);
    let rh = (h * 0.5).max(0.5);
    let irw = (rw - th).max(0.5);
    let irh = (rh - th).max(0.5);
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    let mut pts = Vec::with_capacity(teeth as usize * 4);
    let step = std::f32::consts::TAU / teeth as f32;
    for i in 0..teeth {
        let mid = i as f32 * step;
        let half_tooth = step * 0.18;
        let half_gap = step * 0.5;
        for (a, rx, ry) in [
            (mid - half_gap, irw, irh),
            (mid - half_tooth, rw, rh),
            (mid + half_tooth, rw, rh),
            (mid + half_gap, irw, irh),
        ] {
            pts.push(map(hc + rx * a.cos(), vc + ry * a.sin()));
        }
    }
    pts
}

pub(super) fn gear6_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    gear_points(x, y, w, h, 6, 15_000.0)
}

pub(super) fn gear9_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    gear_points(x, y, w, h, 9, 10_000.0)
}

pub(super) fn plaque_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML plaque adj=16667: square with concave quarter-circles at corners.
    const CD4: f32 = 5_400_000.0;
    const CD2: f32 = 10_800_000.0;
    const CD3_4: f32 = 16_200_000.0;
    let r = (preset_ss(w, h) * 16_667.0 / 100_000.0).max(0.5);
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    let mut cur = (0.0, r);
    let mut pts = vec![map(cur.0, cur.1)];
    ooxml_arc_to_y_down(&mut cur, r, r, CD4, -CD4, &mut pts, map);
    cur = (w - r, 0.0);
    pts.push(map(cur.0, cur.1));
    ooxml_arc_to_y_down(&mut cur, r, r, CD2, -CD4, &mut pts, map);
    cur = (w, h - r);
    pts.push(map(cur.0, cur.1));
    ooxml_arc_to_y_down(&mut cur, r, r, CD3_4, -CD4, &mut pts, map);
    cur = (r, h);
    pts.push(map(cur.0, cur.1));
    ooxml_arc_to_y_down(&mut cur, r, r, 0.0, -CD4, &mut pts, map);
    pts
}

pub(super) fn chord_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML chord adj1=2700000 adj2=16200000: 225° arc then close (no pie centre).
    const ST: f32 = 2_700_000.0;
    const SW: f32 = 13_500_000.0;
    let hc = w * 0.5;
    let vc = h * 0.5;
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    let st = ooxml_ang_rad(ST);
    let mut cur = (hc + hc * st.cos(), vc + vc * st.sin());
    let mut pts = vec![map(cur.0, cur.1)];
    ooxml_arc_to_y_down(&mut cur, hc, vc, ST, SW, &mut pts, map);
    pts
}

pub(super) fn arc_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML arc adj1=16200000 adj2=0: 90° wedge from 12 o'clock clockwise to 3.
    const ST: f32 = 16_200_000.0;
    const SW: f32 = 5_400_000.0;
    let hc = w * 0.5;
    let vc = h * 0.5;
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    let st = ooxml_ang_rad(ST);
    let mut cur = (hc + hc * st.cos(), vc + vc * st.sin());
    let mut pts = vec![map(cur.0, cur.1)];
    ooxml_arc_to_y_down(&mut cur, hc, vc, ST, SW, &mut pts, map);
    pts.push(map(hc, vc));
    pts
}

pub(super) fn snip1_rect_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML snip1Rect adj=16667: rectangle with the top-right corner cut.
    let dx1 = (preset_ss(w, h) * 16_667.0 / 100_000.0).max(0.5);
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    vec![
        map(0.0, 0.0),
        map(w - dx1, 0.0),
        map(w, dx1),
        map(w, h),
        map(0.0, h),
    ]
}

pub(super) fn round1_rect_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML round1Rect adj=16667: rectangle with a 90° arc at top-right.
    const CD4: f32 = 5_400_000.0;
    const CD3_4: f32 = 16_200_000.0;
    let dx1 = (preset_ss(w, h) * 16_667.0 / 100_000.0).max(0.5);
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    let mut cur = (0.0, 0.0);
    let mut pts = vec![map(cur.0, cur.1)];
    cur = (w - dx1, 0.0);
    pts.push(map(cur.0, cur.1));
    ooxml_arc_to_y_down(&mut cur, dx1, dx1, CD3_4, CD4, &mut pts, map);
    pts.push(map(w, h));
    pts.push(map(0.0, h));
    pts
}

pub(super) fn snip2_same_rect_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML snip2SameRect adj1=16667 adj2=0: both top corners cut;
    // bottom corners stay square.
    let tx1 = (preset_ss(w, h) * 16_667.0 / 100_000.0).max(0.5);
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    vec![
        map(tx1, 0.0),
        map(w - tx1, 0.0),
        map(w, tx1),
        map(w, h),
        map(0.0, h),
        map(0.0, tx1),
    ]
}

pub(super) fn round2_same_rect_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML round2SameRect adj1=16667 adj2=0: both top corners rounded;
    // bottom corners stay square (adj2 radius is 0).
    const CD4: f32 = 5_400_000.0;
    const CD2: f32 = 10_800_000.0;
    const CD3_4: f32 = 16_200_000.0;
    let tx1 = (preset_ss(w, h) * 16_667.0 / 100_000.0).max(0.5);
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    let mut cur = (tx1, 0.0);
    let mut pts = vec![map(cur.0, cur.1)];
    cur = (w - tx1, 0.0);
    pts.push(map(cur.0, cur.1));
    ooxml_arc_to_y_down(&mut cur, tx1, tx1, CD3_4, CD4, &mut pts, map);
    pts.push(map(w, h));
    pts.push(map(0.0, h));
    cur = (0.0, tx1);
    pts.push(map(cur.0, cur.1));
    ooxml_arc_to_y_down(&mut cur, tx1, tx1, CD2, CD4, &mut pts, map);
    pts
}

pub(super) fn snip2_diag_rect_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML snip2DiagRect adj1=0 adj2=16667: snip top-right and bottom-left;
    // the other two corners stay square.
    let rx1 = (preset_ss(w, h) * 16_667.0 / 100_000.0).max(0.5);
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    vec![
        map(0.0, 0.0),
        map(w - rx1, 0.0),
        map(w, rx1),
        map(w, h),
        map(rx1, h),
        map(0.0, h - rx1),
    ]
}

pub(super) fn round2_diag_rect_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML round2DiagRect adj1=16667 adj2=0: round top-left and bottom-right;
    // the other two corners stay square (adj2 radius is 0).
    const CD4: f32 = 5_400_000.0;
    const CD2: f32 = 10_800_000.0;
    let x1 = (preset_ss(w, h) * 16_667.0 / 100_000.0).max(0.5);
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    let mut cur = (x1, 0.0);
    let mut pts = vec![map(cur.0, cur.1)];
    pts.push(map(w, 0.0));
    cur = (w, h - x1);
    pts.push(map(cur.0, cur.1));
    ooxml_arc_to_y_down(&mut cur, x1, x1, 0.0, CD4, &mut pts, map);
    pts.push(map(0.0, h));
    cur = (0.0, x1);
    pts.push(map(cur.0, cur.1));
    ooxml_arc_to_y_down(&mut cur, x1, x1, CD2, CD4, &mut pts, map);
    pts
}

pub(super) fn wave_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML wave adj1=12500 adj2=0: two cubics (top then bottom reverse).
    let y1 = h * 12_500.0 / 100_000.0;
    let dy2 = y1 * 10.0 / 3.0;
    let y2 = y1 - dy2;
    let y3 = y1 + dy2;
    let y4 = h - y1;
    let y5 = y4 - dy2;
    let y6 = y4 + dy2;
    let py = |yd: f32| y + h - yd;
    let p0 = (x, py(y1));
    let p1 = (x + w, py(y1));
    let p2 = (x + w, py(y4));
    let p3 = (x, py(y4));
    let mut pts = vec![p0];
    sample_cubic(
        p0,
        (x + w / 3.0, py(y2)),
        (x + w * 2.0 / 3.0, py(y3)),
        p1,
        8,
        &mut pts,
    );
    pts.push(p2);
    sample_cubic(
        p2,
        (x + w * 2.0 / 3.0, py(y6)),
        (x + w / 3.0, py(y5)),
        p3,
        8,
        &mut pts,
    );
    pts
}

pub(super) fn teardrop_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML teardrop adj=100000: 3/4 ellipse + two quads to the top-right tip.
    const CD4: f32 = 5_400_000.0;
    const CD2: f32 = 10_800_000.0;
    let hc = w * 0.5;
    let vc = h * 0.5;
    let wd2 = (w * 0.5).max(0.5);
    let hd2 = (h * 0.5).max(0.5);
    let a = 100_000.0;
    let r2 = std::f32::consts::SQRT_2;
    let tw = wd2 * r2;
    let th = hd2 * r2;
    let sw = tw * a / 100_000.0;
    let sh = th * a / 100_000.0;
    let a45 = ooxml_ang_rad(2_700_000.0);
    let x1 = hc + sw * a45.cos();
    let y1 = vc - sh * a45.sin();
    let x2 = (hc + x1) * 0.5;
    let y2 = (vc + y1) * 0.5;
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    let mut cur = (0.0, vc);
    let mut pts = vec![map(cur.0, cur.1)];
    ooxml_arc_to_y_down(&mut cur, wd2, hd2, CD2, CD4, &mut pts, map);
    sample_quad_y_down(&mut cur, (x2, 0.0), (x1, y1), 8, &mut pts, map);
    sample_quad_y_down(&mut cur, (w, y2), (w, vc), 8, &mut pts, map);
    ooxml_arc_to_y_down(&mut cur, wd2, hd2, 0.0, CD4, &mut pts, map);
    ooxml_arc_to_y_down(&mut cur, wd2, hd2, CD4, CD4, &mut pts, map);
    pts
}

fn sample_quad_y_down(
    cur: &mut (f32, f32),
    ctrl: (f32, f32),
    end: (f32, f32),
    steps: i32,
    pts: &mut Vec<(f32, f32)>,
    map: impl Fn(f32, f32) -> (f32, f32),
) {
    let p0 = *cur;
    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let u = 1.0 - t;
        *cur = (
            u * u * p0.0 + 2.0 * u * t * ctrl.0 + t * t * end.0,
            u * u * p0.1 + 2.0 * u * t * ctrl.1 + t * t * end.1,
        );
        pts.push(map(cur.0, cur.1));
    }
}

pub(super) fn round_rect_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    let r = (w.min(h) * 16_667.0 / 100_000.0).clamp(0.5, w.min(h) * 0.49);
    let mut pts = Vec::with_capacity(24);
    pts.push((x + r, y));
    pts.push((x + w - r, y));
    quarter_arc(&mut pts, x + w - r, y + r, r, -90.0, 0.0);
    pts.push((x + w, y + h - r));
    quarter_arc(&mut pts, x + w - r, y + h - r, r, 0.0, 90.0);
    pts.push((x + r, y + h));
    quarter_arc(&mut pts, x + r, y + h - r, r, 90.0, 180.0);
    pts.push((x, y + r));
    quarter_arc(&mut pts, x + r, y + r, r, 180.0, 270.0);
    pts
}

fn quarter_arc(pts: &mut Vec<(f32, f32)>, cx: f32, cy: f32, r: f32, deg0: f32, deg1: f32) {
    const STEPS: i32 = 4;
    for i in 1..=STEPS {
        let t = i as f32 / STEPS as f32;
        let a = (deg0 + (deg1 - deg0) * t).to_radians();
        pts.push((cx + r * a.cos(), cy + r * a.sin()));
    }
}

/// rightArrow, leftArrow, upArrow and downArrow at their default
/// adjustments: the shaft is the middle half across the arrow, the head
/// min(w, h)/2 long (English part a 1f3856c4's down arrows point down).
pub(super) fn block_arrow_points(prst: &str, x: f32, y: f32, dw: f32, dh: f32) -> Vec<(f32, f32)> {
    let head = dw.min(dh) * 0.5;
    let hc = x + dw * 0.5;
    let (x1, x2) = (x + dw * 0.25, x + dw * 0.75);
    match prst {
        "leftArrow" => right_arrow_points(x, y, dw, dh)
            .into_iter()
            .map(|(px, py)| (2.0 * x + dw - px, py))
            .collect(),
        "downArrow" => vec![
            (x1, y + dh),
            (x2, y + dh),
            (x2, y + head),
            (x + dw, y + head),
            (hc, y),
            (x, y + head),
            (x1, y + head),
        ],
        "upArrow" => vec![
            (x1, y),
            (x2, y),
            (x2, y + dh - head),
            (x + dw, y + dh - head),
            (hc, y + dh),
            (x, y + dh - head),
            (x1, y + dh - head),
        ],
        _ => right_arrow_points(x, y, dw, dh),
    }
}

pub(super) fn right_arrow_points(x: f32, y: f32, dw: f32, dh: f32) -> Vec<(f32, f32)> {
    let head = dw.min(dh) * 0.5;
    let body_w = (dw - head).max(1.0);
    let shaft_bot = y + dh * 0.25;
    let shaft_top = y + dh * 0.75;
    let vc = y + dh * 0.5;
    vec![
        (x, shaft_top),
        (x + body_w, shaft_top),
        (x + body_w, y + dh),
        (x + dw, vc),
        (x + body_w, y),
        (x + body_w, shaft_bot),
        (x, shaft_bot),
    ]
}

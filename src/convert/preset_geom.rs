// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! DrawingML preset geometry evaluator (ECMA-376 Part 1 §20.1.9): guide
//! formulas, `a:avLst` adjustments and the path list of a preset, flattened
//! to polylines in shape space (points, y down, origin at the box's top-left).
//!
//! Only presets listed in `preset_geom_data` go through here: the ones whose
//! Word rendering needs compound fills (donut, frame, noSmoking), shaded
//! faces (cube, bevel, can, foldedCorner, ribbon), a separate stroke-only
//! outline or open outlines (brackets, braces), or exact guides (moon,
//! circular arrows, upDownArrow).

use std::collections::HashMap;

use super::preset_geom_data::PRESETS;
use super::preset_text_rect_data::TEXT_RECTS;

/// `a:path/@fill`: how the path's fill is shaded from the shape fill.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Fill {
    None,
    Norm,
    Lighten,
    LightenLess,
    Darken,
    DarkenLess,
}

impl Fill {
    /// Word's shade of `base` for this path (ECMA-376 ST_PathFillMode):
    /// darken 60%, darkenLess 80%, lighten 40% / lightenLess 20% toward
    /// white.
    pub(crate) fn shade(self, base: [f32; 3]) -> Option<[f32; 3]> {
        let mix = |t: f32| base.map(|c| c + (1.0 - c) * t);
        match self {
            Fill::None => None,
            Fill::Norm => Some(base),
            Fill::Darken => Some(base.map(|c| c * 0.6)),
            Fill::DarkenLess => Some(base.map(|c| c * 0.8)),
            Fill::Lighten => Some(mix(0.4)),
            Fill::LightenLess => Some(mix(0.2)),
        }
    }
}

/// One path command; operands are guide names or numeric literals.
pub(crate) enum Cmd {
    M(&'static str, &'static str),
    L(&'static str, &'static str),
    /// `arcTo wR hR stAng swAng`.
    A(&'static str, &'static str, &'static str, &'static str),
    C([&'static str; 6]),
    Q([&'static str; 4]),
    Z,
}

pub(crate) struct PathDef {
    /// Path coordinate space; 0 means the shape's own w / h.
    pub w: f64,
    pub h: f64,
    pub fill: Fill,
    pub stroke: bool,
    pub cmds: &'static [Cmd],
}

pub(crate) struct Preset {
    pub av: &'static [(&'static str, &'static str)],
    pub gd: &'static [(&'static str, &'static str)],
    pub paths: &'static [PathDef],
}

/// A path command borrowed from a preset or from a document's
/// `a:custGeom`, so one evaluator serves both.
#[derive(Clone, Copy)]
enum CmdRef<'a> {
    M(&'a str, &'a str),
    L(&'a str, &'a str),
    A(&'a str, &'a str, &'a str, &'a str),
    C([&'a str; 6]),
    Q([&'a str; 4]),
    Z,
}

impl Cmd {
    fn view(&self) -> CmdRef<'static> {
        match *self {
            Cmd::M(x, y) => CmdRef::M(x, y),
            Cmd::L(x, y) => CmdRef::L(x, y),
            Cmd::A(a, b, c, d) => CmdRef::A(a, b, c, d),
            Cmd::C(p) => CmdRef::C(p),
            Cmd::Q(p) => CmdRef::Q(p),
            Cmd::Z => CmdRef::Z,
        }
    }
}

/// One `a:custGeom` path command; operands are guide names or numbers.
pub(crate) enum OwnedCmd {
    M(String, String),
    L(String, String),
    A(String, String, String, String),
    C([String; 6]),
    Q([String; 4]),
    Z,
}

impl OwnedCmd {
    fn view(&self) -> CmdRef<'_> {
        match self {
            OwnedCmd::M(x, y) => CmdRef::M(x, y),
            OwnedCmd::L(x, y) => CmdRef::L(x, y),
            OwnedCmd::A(a, b, c, d) => CmdRef::A(a, b, c, d),
            OwnedCmd::C(p) => CmdRef::C(p.each_ref().map(String::as_str)),
            OwnedCmd::Q(p) => CmdRef::Q(p.each_ref().map(String::as_str)),
            OwnedCmd::Z => CmdRef::Z,
        }
    }
}

/// One `a:custGeom/a:pathLst/a:path`.
pub(crate) struct CustomPath {
    pub w: f64,
    pub h: f64,
    pub fill: Fill,
    pub stroke: bool,
    pub cmds: Vec<OwnedCmd>,
}

/// A document's `a:custGeom`: guides and paths like a preset's.
pub(crate) struct CustomGeom {
    pub av: Vec<(String, String)>,
    pub gd: Vec<(String, String)>,
    pub paths: Vec<CustomPath>,
}

struct PathView<'a> {
    w: f64,
    h: f64,
    fill: Fill,
    stroke: bool,
    cmds: Vec<CmdRef<'a>>,
}

/// A flattened subpath in shape space (pt, y down).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Subpath {
    pub pts: Vec<(f32, f32)>,
    pub closed: bool,
}

/// One evaluated `a:path`.
#[derive(Clone, Debug)]
pub(crate) struct EvalPath {
    pub fill: Fill,
    pub stroke: bool,
    pub subpaths: Vec<Subpath>,
}

/// A preset's `a:rect`: the guides its text rectangle reads.
pub(crate) struct TextRect {
    pub av: &'static [(&'static str, &'static str)],
    pub gd: &'static [(&'static str, &'static str)],
    pub rect: [&'static str; 4],
}

/// The text rectangle `[l, t, r, b]` (points, y down) of preset `name`
/// in a `w`×`h` box, or `None` when it is the whole box.
pub(crate) fn text_rect(name: &str, w: f32, h: f32, adj: &[(String, f64)]) -> Option<[f32; 4]> {
    let (_, def) = TEXT_RECTS.iter().find(|(n, _)| *n == name)?;
    let g = guides(def.av, def.gd, f64::from(w), f64::from(h), adj);
    let at = |k: &str| operand(k, &g) as f32;
    Some(def.rect.map(at))
}

pub(crate) fn preset(name: &str) -> Option<&'static Preset> {
    PRESETS.iter().find(|(n, _)| *n == name).map(|(_, p)| p)
}

/// DrawingML angles are 60000ths of a degree.
fn ang_rad(v: f64) -> f64 {
    (v / 60_000.0).to_radians()
}

fn rad_ang(r: f64) -> f64 {
    r.to_degrees() * 60_000.0
}

/// Guide values for a `w`×`h` shape with `adj` overrides (`a:avLst`).
fn guides(
    av: &[(&str, &str)],
    gd: &[(&str, &str)],
    w: f64,
    h: f64,
    adj: &[(String, f64)],
) -> HashMap<String, f64> {
    let mut g: HashMap<String, f64> = HashMap::new();
    let ss = w.min(h);
    for (name, v) in [
        ("w", w),
        ("h", h),
        ("l", 0.0),
        ("t", 0.0),
        ("r", w),
        ("b", h),
        ("hc", w / 2.0),
        ("vc", h / 2.0),
        ("ss", ss),
        ("ls", w.max(h)),
        ("cd2", 10_800_000.0),
        ("cd4", 5_400_000.0),
        ("cd8", 2_700_000.0),
        ("3cd4", 16_200_000.0),
        ("3cd8", 8_100_000.0),
        ("5cd8", 13_500_000.0),
        ("7cd8", 18_900_000.0),
    ] {
        g.insert(name.to_string(), v);
    }
    for n in [2, 3, 4, 5, 6, 8, 10, 12, 16, 32] {
        g.insert(format!("wd{n}"), w / f64::from(n));
        g.insert(format!("hd{n}"), h / f64::from(n));
        g.insert(format!("ssd{n}"), ss / f64::from(n));
    }
    for (name, fmla) in av {
        let v = adj
            .iter()
            .find(|(k, _)| k == name)
            .map_or_else(|| formula(fmla, &g), |(_, v)| *v);
        g.insert((*name).to_string(), v);
    }
    for (name, fmla) in gd {
        let v = formula(fmla, &g);
        g.insert((*name).to_string(), v);
    }
    g
}

fn operand(tok: &str, g: &HashMap<String, f64>) -> f64 {
    tok.parse::<f64>()
        .ok()
        .or_else(|| g.get(tok).copied())
        .unwrap_or(0.0)
}

/// ECMA-376 §20.1.9.11 guide formula.
fn formula(fmla: &str, g: &HashMap<String, f64>) -> f64 {
    let mut parts = fmla.split_whitespace();
    let op = parts.next().unwrap_or("val");
    let a: Vec<f64> = parts.map(|t| operand(t, g)).collect();
    let x = a.first().copied().unwrap_or(0.0);
    let y = a.get(1).copied().unwrap_or(0.0);
    let z = a.get(2).copied().unwrap_or(0.0);
    match op {
        "*/" => {
            if z == 0.0 {
                0.0
            } else {
                x * y / z
            }
        }
        "+-" => x + y - z,
        "+/" => {
            if z == 0.0 {
                0.0
            } else {
                (x + y) / z
            }
        }
        "?:" => {
            if x > 0.0 {
                y
            } else {
                z
            }
        }
        "abs" => x.abs(),
        "at2" => rad_ang(y.atan2(x)),
        "cat2" => x * z.atan2(y).cos(),
        "sat2" => x * z.atan2(y).sin(),
        "cos" => x * ang_rad(y).cos(),
        "sin" => x * ang_rad(y).sin(),
        "tan" => x * ang_rad(y).tan(),
        "max" => x.max(y),
        "min" => x.min(y),
        "mod" => (x * x + y * y + z * z).sqrt(),
        "pin" => {
            if y < x {
                x
            } else if y > z {
                z
            } else {
                y
            }
        }
        "sqrt" => x.max(0.0).sqrt(),
        _ => x,
    }
}

/// The parametric angle of the point at ray angle `a` on an ellipse with
/// radii `wr`, `hr` (DrawingML arc angles are ray angles, not parameters).
fn param_angle(a: f64, wr: f64, hr: f64) -> f64 {
    (wr * a.sin()).atan2(hr * a.cos())
}

/// Evaluate `preset` for a `w`×`h` box (points) with `a:avLst` overrides.
pub(crate) fn evaluate(preset: &Preset, w: f32, h: f32, adj: &[(String, f64)]) -> Vec<EvalPath> {
    let paths: Vec<PathView<'static>> = preset
        .paths
        .iter()
        .map(|p| PathView {
            w: p.w,
            h: p.h,
            fill: p.fill,
            stroke: p.stroke,
            cmds: p.cmds.iter().map(Cmd::view).collect(),
        })
        .collect();
    evaluate_views(preset.av, preset.gd, &paths, w, h, adj, 1.0)
}

/// EMU per point: a custGeom's guides and unsized paths live in the
/// shape's EMU space (ECMA-376 20.1.9.8 / 20.1.9.15).
const EMU_PER_PT: f64 = 12_700.0;

/// Evaluate a document's `a:custGeom` for a `w`×`h` box (points).
pub(crate) fn evaluate_custom(geom: &CustomGeom, w: f32, h: f32) -> Vec<EvalPath> {
    fn pairs(v: &[(String, String)]) -> Vec<(&str, &str)> {
        v.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect()
    }
    let paths: Vec<PathView<'_>> = geom
        .paths
        .iter()
        .map(|p| PathView {
            w: p.w,
            h: p.h,
            fill: p.fill,
            stroke: p.stroke,
            cmds: p.cmds.iter().map(OwnedCmd::view).collect(),
        })
        .collect();
    evaluate_views(
        &pairs(&geom.av),
        &pairs(&geom.gd),
        &paths,
        w,
        h,
        &[],
        EMU_PER_PT,
    )
}

/// `units` is guide units per point: presets keep points (1), a custGeom
/// works in EMU, and a path without its own w/h takes the guides' space.
fn evaluate_views(
    av: &[(&str, &str)],
    gd: &[(&str, &str)],
    paths: &[PathView<'_>],
    w: f32,
    h: f32,
    adj: &[(String, f64)],
    units: f64,
) -> Vec<EvalPath> {
    let (w, h) = (f64::from(w.max(0.01)), f64::from(h.max(0.01)));
    let g = guides(av, gd, w * units, h * units, adj);
    let val = |tok: &str| operand(tok, &g);
    let mut out = Vec::new();
    for path in paths {
        let sx = if path.w > 0.0 {
            w / path.w
        } else {
            1.0 / units
        };
        let sy = if path.h > 0.0 {
            h / path.h
        } else {
            1.0 / units
        };
        let pt = |x: f64, y: f64| ((x * sx) as f32, (y * sy) as f32);
        let mut subpaths: Vec<Subpath> = Vec::new();
        let mut cur = (0.0_f64, 0.0_f64);
        let mut start = (0.0_f64, 0.0_f64);
        let begin = |subpaths: &mut Vec<Subpath>, p: (f64, f64)| {
            subpaths.push(Subpath {
                pts: vec![pt(p.0, p.1)],
                closed: false,
            });
        };
        for cmd in &path.cmds {
            match *cmd {
                CmdRef::M(x, y) => {
                    cur = (val(x), val(y));
                    start = cur;
                    begin(&mut subpaths, cur);
                }
                CmdRef::L(x, y) => {
                    cur = (val(x), val(y));
                    if subpaths.is_empty() {
                        begin(&mut subpaths, cur);
                    } else if let Some(s) = subpaths.last_mut() {
                        s.pts.push(pt(cur.0, cur.1));
                    }
                }
                CmdRef::A(wr, hr, st, sw) => {
                    let (wr, hr) = (val(wr), val(hr));
                    let (st, sw) = (ang_rad(val(st)), ang_rad(val(sw)));
                    let t0 = param_angle(st, wr, hr);
                    // Keep the sweep's direction and turns in parameter space.
                    let mut t1 = param_angle(st + sw, wr, hr);
                    let turns = (sw / std::f64::consts::TAU).trunc();
                    t1 += turns * std::f64::consts::TAU;
                    if sw > 0.0 && t1 < t0 {
                        t1 += std::f64::consts::TAU;
                    } else if sw < 0.0 && t1 > t0 {
                        t1 -= std::f64::consts::TAU;
                    }
                    let (cx, cy) = (cur.0 - wr * t0.cos(), cur.1 - hr * t0.sin());
                    // `swAng` is document data: past a few turns the arc only
                    // retraces its ellipse, so cap the tessellation (8 turns
                    // at 8 points a quarter) and skip a non-finite sweep.
                    let quarters = (t1 - t0).abs() / std::f64::consts::FRAC_PI_2;
                    if !quarters.is_finite() {
                        continue;
                    }
                    let n = (quarters.ceil().min(32.0) as usize * 8).max(2);
                    if subpaths.is_empty() {
                        begin(&mut subpaths, cur);
                    }
                    for i in 1..=n {
                        let t = t0 + (t1 - t0) * i as f64 / n as f64;
                        cur = (cx + wr * t.cos(), cy + hr * t.sin());
                        if let Some(s) = subpaths.last_mut() {
                            s.pts.push(pt(cur.0, cur.1));
                        }
                    }
                }
                CmdRef::C(p) => {
                    let c1 = (val(p[0]), val(p[1]));
                    let c2 = (val(p[2]), val(p[3]));
                    let e = (val(p[4]), val(p[5]));
                    let p0 = cur;
                    if subpaths.is_empty() {
                        begin(&mut subpaths, cur);
                    }
                    for i in 1..=12 {
                        let t = f64::from(i) / 12.0;
                        let u = 1.0 - t;
                        let x = u * u * u * p0.0
                            + 3.0 * u * u * t * c1.0
                            + 3.0 * u * t * t * c2.0
                            + t * t * t * e.0;
                        let y = u * u * u * p0.1
                            + 3.0 * u * u * t * c1.1
                            + 3.0 * u * t * t * c2.1
                            + t * t * t * e.1;
                        if let Some(s) = subpaths.last_mut() {
                            s.pts.push(pt(x, y));
                        }
                    }
                    cur = e;
                }
                CmdRef::Q(p) => {
                    let c = (val(p[0]), val(p[1]));
                    let e = (val(p[2]), val(p[3]));
                    let p0 = cur;
                    if subpaths.is_empty() {
                        begin(&mut subpaths, cur);
                    }
                    for i in 1..=10 {
                        let t = f64::from(i) / 10.0;
                        let u = 1.0 - t;
                        let x = u * u * p0.0 + 2.0 * u * t * c.0 + t * t * e.0;
                        let y = u * u * p0.1 + 2.0 * u * t * c.1 + t * t * e.1;
                        if let Some(s) = subpaths.last_mut() {
                            s.pts.push(pt(x, y));
                        }
                    }
                    cur = e;
                }
                CmdRef::Z => {
                    if let Some(s) = subpaths.last_mut() {
                        s.closed = true;
                    }
                    cur = start;
                }
            }
        }
        subpaths.retain(|s| s.pts.len() >= 2);
        out.push(EvalPath {
            fill: path.fill,
            stroke: path.stroke,
            subpaths,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval(name: &str, w: f32, h: f32) -> Vec<EvalPath> {
        evaluate(preset(name).expect(name), w, h, &[])
    }

    fn bounds(paths: &[EvalPath]) -> (f32, f32, f32, f32) {
        let mut b = (
            f32::INFINITY,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NEG_INFINITY,
        );
        for p in paths {
            for s in &p.subpaths {
                for &(x, y) in &s.pts {
                    b = (b.0.min(x), b.1.min(y), b.2.max(x), b.3.max(y));
                }
            }
        }
        b
    }

    #[test]
    fn custom_guides_live_in_the_shapes_emu_space() {
        // PR #167 review: a custGeom path in EMU (w=1270000 for a 100pt
        // box) that names the built-in `hc` / `b` guides took them in
        // points, so L hc,b landed 12700x too close to the origin.
        let s = |v: &str| v.to_string();
        let path = |w: f64, h: f64| CustomPath {
            w,
            h,
            fill: Fill::Norm,
            stroke: true,
            cmds: vec![OwnedCmd::M(s("0"), s("0")), OwnedCmd::L(s("hc"), s("b"))],
        };
        for (w, h) in [(1_270_000.0, 1_270_000.0), (0.0, 0.0)] {
            let geom = CustomGeom {
                av: Vec::new(),
                gd: Vec::new(),
                paths: vec![path(w, h)],
            };
            let out = evaluate_custom(&geom, 100.0, 100.0);
            let end = *out[0].subpaths[0].pts.last().expect("L point");
            assert!(
                (end.0 - 50.0).abs() < 0.01 && (end.1 - 100.0).abs() < 0.01,
                "path {w}x{h}: L hc,b ends at the box's bottom centre; got {end:?}"
            );
        }
    }

    #[test]
    fn a_custom_arc_sweep_is_tessellated_within_a_bound() {
        // PR #167 review: `swAng` is document-controlled and the arc's
        // point count grew with it unbounded (a guide formula can make it
        // any f64, and an infinite sweep never ended).
        let s = |v: &str| v.to_string();
        let arc = |sw: &str| CustomGeom {
            av: Vec::new(),
            gd: vec![(s("big"), s(sw))],
            paths: vec![CustomPath {
                w: 1_270_000.0,
                h: 1_270_000.0,
                fill: Fill::None,
                stroke: true,
                cmds: vec![
                    OwnedCmd::M(s("1270000"), s("635000")),
                    OwnedCmd::A(s("635000"), s("635000"), s("0"), s("big")),
                ],
            }],
        };
        for sw in ["*/ 21600000 1000 1", "*/ 21600000 21600000 1"] {
            let out = evaluate_custom(&arc(sw), 100.0, 100.0);
            let n: usize = out[0].subpaths.iter().map(|p| p.pts.len()).sum();
            assert!(n <= 300, "sweep {sw}: {n} points");
        }
    }

    #[test]
    fn formulas_follow_ecma_20_1_9_11() {
        let g = HashMap::from([("a".to_string(), 3.0), ("b".to_string(), 4.0)]);
        assert_eq!(formula("*/ a b 2", &g), 6.0);
        assert_eq!(formula("+- a b 1", &g), 6.0);
        assert_eq!(formula("+/ a b 7", &g), 1.0);
        assert_eq!(formula("?: a 1 2", &g), 1.0);
        assert_eq!(formula("mod a b 0", &g), 5.0);
        assert_eq!(formula("pin 0 b 2", &g), 2.0);
        assert!((formula("at2 1 1", &g) - 2_700_000.0).abs() < 1.0);
        assert!((formula("cos 10 5400000", &g)).abs() < 1e-9);
    }

    #[test]
    fn up_down_arrow_keeps_a_shaft_on_wide_extents() {
        // #34: on a 2:1 box the old head length hd/2 met in the middle.
        let paths = eval("upDownArrow", 141.73, 70.87);
        let pts = &paths[0].subpaths[0].pts;
        let ys: Vec<f32> = pts.iter().map(|p| p.1).collect();
        let shaft_top = ys
            .iter()
            .copied()
            .filter(|y| *y > 1.0 && *y < 35.0)
            .fold(0.0_f32, f32::max);
        let shaft_bot = ys
            .iter()
            .copied()
            .filter(|y| *y > 35.0 && *y < 69.0)
            .fold(f32::INFINITY, f32::min);
        assert!(
            shaft_bot - shaft_top > 1.0,
            "a visible shaft between the heads: {ys:?}"
        );
    }

    #[test]
    fn donut_is_two_closed_contours() {
        let paths = eval("donut", 100.0, 100.0);
        assert_eq!(paths[0].subpaths.len(), 2, "outer ring and inner hole");
        assert!(paths[0].subpaths.iter().all(|s| s.closed));
    }

    #[test]
    fn brackets_stroke_an_open_path() {
        for name in [
            "leftBracket",
            "rightBracket",
            "leftBrace",
            "rightBrace",
            "bracePair",
            "bracketPair",
        ] {
            let paths = eval(name, 40.0, 100.0);
            let outline: Vec<&EvalPath> = paths.iter().filter(|p| p.stroke).collect();
            assert!(!outline.is_empty(), "{name} has an outline path");
            assert!(
                outline
                    .iter()
                    .all(|p| p.fill == Fill::None && p.subpaths.iter().all(|s| !s.closed)),
                "{name}: the outline is open (no closing chord)"
            );
        }
    }

    #[test]
    fn cube_shades_its_faces_and_strokes_one_outline() {
        let paths = eval("cube", 100.0, 100.0);
        let fills: Vec<Fill> = paths.iter().filter(|p| !p.stroke).map(|p| p.fill).collect();
        assert_eq!(fills, [Fill::Norm, Fill::DarkenLess, Fill::LightenLess]);
        assert_eq!(
            paths.iter().filter(|p| p.stroke).count(),
            1,
            "one fill=none outline"
        );
    }

    #[test]
    fn circular_arrow_stays_inside_its_extent() {
        // #42/#69: the default circular arrows fit their box.
        for name in [
            "circularArrow",
            "leftCircularArrow",
            "leftRightCircularArrow",
        ] {
            let (x0, y0, x1, y1) = bounds(&eval(name, 100.0, 100.0));
            assert!(
                x0 > -1.0 && y0 > -1.0 && x1 < 101.0 && y1 < 101.0,
                "{name} bounds {x0},{y0},{x1},{y1}"
            );
        }
    }

    #[test]
    fn moon_is_a_crescent_inside_its_box() {
        let paths = eval("moon", 50.0, 100.0);
        let (x0, y0, x1, y1) = bounds(&paths);
        assert!(
            x0 > -0.5 && y0 > -0.5 && x1 < 50.5 && y1 < 100.5,
            "moon bounds {x0},{y0},{x1},{y1}"
        );
        assert!(paths[0].subpaths[0].closed);
    }

    #[test]
    fn avlst_overrides_the_default_adjustment() {
        let p = preset("frame").expect("frame");
        let thin = evaluate(p, 100.0, 100.0, &[("adj1".into(), 5_000.0)]);
        let inner = &thin[0].subpaths[1].pts;
        assert!(
            (inner[0].0 - 5.0).abs() < 0.01,
            "adj1 5000 → 5pt border: {inner:?}"
        );
    }

    #[test]
    fn fill_modes_shade_toward_black_or_white() {
        let base = [0.5, 0.5, 0.5];
        assert_eq!(Fill::DarkenLess.shade(base), Some([0.4, 0.4, 0.4]));
        assert_eq!(Fill::Lighten.shade(base), Some([0.7, 0.7, 0.7]));
        assert_eq!(Fill::None.shade(base), None);
    }

    mod regression_tests {
        use super::*;

        #[test]
        fn text_rectangle_uses_shape_dimensions_in_points() {
            assert_eq!(
                text_rect("flowChartInternalStorage", 160.0, 80.0, &[]),
                Some([20.0, 10.0, 160.0, 80.0])
            );
            assert_eq!(
                text_rect("flowChartInternalStorage", 80.0, 160.0, &[]),
                Some([10.0, 20.0, 80.0, 160.0])
            );
            assert_eq!(text_rect("unknown-preset", 160.0, 80.0, &[]), None);
            assert_eq!(text_rect("rect", 160.0, 80.0, &[]), None);
        }

        #[test]
        fn pie_and_gear_text_rectangles_match_word() {
            // Live Word, 216pt square, zero insets: a pie sets its text in the
            // ellipse's inset rectangle (text at x = 32pt into the box),
            // though presetShapeDefinitions.xml lists `t="ir" r="it"`.
            let pie = text_rect("pie", 216.0, 216.0, &[]).expect("pie");
            assert_eq!(
                pie,
                text_rect("ellipse", 216.0, 216.0, &[]).expect("ellipse")
            );
            assert!((pie[0] - 31.63).abs() < 0.1, "{pie:?}");
            // gear6 / gear9 read adj1 (and gear9 adj2) through guides that
            // the table's defaults left out; Word starts their text at 55pt
            // and 44pt into the box.
            let g6 = text_rect("gear6", 216.0, 216.0, &[]).expect("gear6");
            assert!((g6[0] - 55.0).abs() < 1.0, "{g6:?}");
            let g9 = text_rect("gear9", 216.0, 216.0, &[]).expect("gear9");
            assert!((g9[0] - 44.0).abs() < 1.0, "{g9:?}");
        }

        #[test]
        fn adjusted_text_rectangle_clamps_to_the_shapes_limits() {
            for (adjustment, bottom) in [(-1.0, 100.0), (25000.0, 75.0), (75000.0, 50.0)] {
                assert_eq!(
                    text_rect("foldedCorner", 200.0, 100.0, &[("adj".into(), adjustment)]),
                    Some([0.0, 0.0, 200.0, bottom])
                );
            }
        }
    }
}

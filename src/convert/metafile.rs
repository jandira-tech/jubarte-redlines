// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Rasterize placeable WMF and EMF to RGB (Strict01 cliparts).
//!
//! Not a full GDI replay. Enough records to paint image1.bin (polygons)
//! and image2.emf (pen strokes + PATCOPY 1px BITBLT).

use std::collections::HashMap;

const PLACEABLE_KEY: [u8; 4] = [0xD7, 0xCD, 0xC6, 0x9A];
const EMF_SIGNATURE: &[u8] = b" EMF";
const MAX_SIDE: usize = 384;
const WHITE: [u8; 3] = [255, 255, 255];

pub(crate) fn rasterize(bytes: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    if looks_like_wmf(bytes) {
        return raster_wmf(bytes);
    }
    if looks_like_emf(bytes) {
        return raster_emf(bytes);
    }
    None
}

fn looks_like_wmf(bytes: &[u8]) -> bool {
    bytes.len() >= 22 && bytes[..4] == PLACEABLE_KEY
}

fn looks_like_emf(bytes: &[u8]) -> bool {
    bytes.len() >= 44 && bytes[40..44] == *EMF_SIGNATURE
}

struct Canvas {
    w: usize,
    h: usize,
    px: Vec<u8>,
}

impl Canvas {
    fn new(w: usize, h: usize) -> Self {
        let w = w.max(1);
        let h = h.max(1);
        Self {
            w,
            h,
            px: vec![255; w * h * 3],
        }
    }

    fn put(&mut self, x: i32, y: i32, color: [u8; 3]) {
        if x < 0 || y < 0 {
            return;
        }
        let (x, y) = (x as usize, y as usize);
        if x >= self.w || y >= self.h {
            return;
        }
        let i = (y * self.w + x) * 3;
        self.px[i] = color[0];
        self.px[i + 1] = color[1];
        self.px[i + 2] = color[2];
    }

    fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: [u8; 3]) {
        let x1 = x.max(0);
        let y1 = y.max(0);
        let x2 = x.saturating_add(w.max(1)).min(self.w as i32);
        let y2 = y.saturating_add(h.max(1)).min(self.h as i32);
        for yy in y1..y2 {
            for xx in x1..x2 {
                self.put(xx, yy, color);
            }
        }
    }

    fn stroke_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: [u8; 3], width: i32) {
        // A pen wider than the canvas paints the same pixels as one exactly
        // canvas-sized; capping keeps the per-step fill_rect bounded.
        let w = width.max(1).min(MAX_SIDE as i32);
        // Liang-Barsky clip to the canvas rectangle (grown by the pen
        // radius) BEFORE walking: mapped endpoints from a hostile metafile
        // sit up to i32::MIN..i32::MAX apart, and the walk must be
        // proportional to the canvas, not to the coordinate span.
        let r = f64::from(w / 2 + 1);
        let (min_x, min_y) = (-r, -r);
        let (max_x, max_y) = (self.w as f64 + r, self.h as f64 + r);
        let (fx0, fy0) = (f64::from(x0), f64::from(y0));
        let (fdx, fdy) = (f64::from(x1) - fx0, f64::from(y1) - fy0);
        let (mut t0, mut t1) = (0.0_f64, 1.0_f64);
        for (p, q) in [
            (-fdx, fx0 - min_x),
            (fdx, max_x - fx0),
            (-fdy, fy0 - min_y),
            (fdy, max_y - fy0),
        ] {
            if p == 0.0 {
                if q < 0.0 {
                    return; // parallel and fully outside
                }
            } else {
                let t = q / p;
                if p < 0.0 {
                    if t > t1 {
                        return;
                    }
                    t0 = t0.max(t);
                } else {
                    if t < t0 {
                        return;
                    }
                    t1 = t1.min(t);
                }
            }
        }
        let x0 = (fx0 + t0 * fdx).round() as i32;
        let y0 = (fy0 + t0 * fdy).round() as i32;
        let x1 = (fx0 + t1 * fdx).round() as i32;
        let y1 = (fy0 + t1 * fdy).round() as i32;
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let mut x = x0;
        let mut y = y0;
        loop {
            if w <= 1 {
                self.put(x, y, color);
            } else {
                let r = w / 2;
                self.fill_rect(x.saturating_sub(r), y.saturating_sub(r), w, w, color);
            }
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    fn fill_polygon(&mut self, pts: &[(i32, i32)], color: [u8; 3]) {
        if pts.len() < 3 {
            return;
        }
        let min_y = pts.iter().map(|p| p.1).min().unwrap_or(0).max(0);
        let max_y = pts
            .iter()
            .map(|p| p.1)
            .max()
            .unwrap_or(0)
            .min(self.h as i32 - 1);
        for y in min_y..=max_y {
            let mut xs = Vec::new();
            for i in 0..pts.len() {
                let (x0, y0) = pts[i];
                let (x1, y1) = pts[(i + 1) % pts.len()];
                if (y0 <= y && y1 > y) || (y1 <= y && y0 > y) {
                    let dy = i128::from(y1) - i128::from(y0);
                    if dy != 0 {
                        // Full-range i32 coords overflow the i32 product — and
                        // the i32 *subtraction* too: mapped points saturate to
                        // i32::MIN/MAX (`px.round() as i32`), so `y - y0` with
                        // `y0 == i32::MIN` panics in debug and wraps to a wrong
                        // intersection in release. Widen before subtracting;
                        // the product of two ~2^32 spans needs i128.
                        let x = i128::from(x0)
                            + (i128::from(y) - i128::from(y0)) * (i128::from(x1) - i128::from(x0))
                                / dy;
                        xs.push(x.clamp(-1, self.w as i128) as i32);
                    }
                }
            }
            xs.sort_unstable();
            for pair in xs.chunks(2) {
                if pair.len() < 2 {
                    break;
                }
                // Clamp the span to the canvas so the walk is bounded by
                // the canvas width, not the coordinate span.
                let a = pair[0].min(pair[1]).max(0);
                let b = pair[0].max(pair[1]).min(self.w as i32 - 1);
                for x in a..=b {
                    self.put(x, y, color);
                }
            }
        }
    }

    /// Fill every subpath of a path as one shape: even-odd (ALTERNATE)
    /// keeps letter counters open; `winding` is the nonzero rule.
    fn fill_path(&mut self, subpaths: &[Vec<(i32, i32)>], color: [u8; 3], winding: bool) {
        let edges: Vec<((i32, i32), (i32, i32))> = subpaths
            .iter()
            .filter(|sp| sp.len() >= 2)
            .flat_map(|sp| (0..sp.len()).map(move |i| (sp[i], sp[(i + 1) % sp.len()])))
            .collect();
        if edges.is_empty() {
            return;
        }
        let min_y = edges
            .iter()
            .map(|(a, b)| a.1.min(b.1))
            .min()
            .unwrap_or(0)
            .max(0);
        let max_y = edges
            .iter()
            .map(|(a, b)| a.1.max(b.1))
            .max()
            .unwrap_or(0)
            .min(self.h as i32 - 1);
        for y in min_y..=max_y {
            // (x, direction) crossings at the pixel row's centre line.
            let mut xs: Vec<(i64, i32)> = Vec::new();
            for &((x0, y0), (x1, y1)) in &edges {
                if (y0 <= y && y1 > y) || (y1 <= y && y0 > y) {
                    // i128: saturated points put the product past i64::MAX.
                    let dy = i128::from(y1) - i128::from(y0);
                    let x = i128::from(x0)
                        + (i128::from(y) - i128::from(y0)) * (i128::from(x1) - i128::from(x0)) / dy;
                    let x = x.clamp(-1, self.w as i128) as i64;
                    xs.push((x, if y1 > y0 { 1 } else { -1 }));
                }
            }
            xs.sort_unstable();
            let mut inside = 0_i32;
            let mut start: Option<i64> = None;
            for (x, dir) in xs {
                inside += if winding { dir } else { 1 };
                let on = if winding {
                    inside != 0
                } else {
                    inside % 2 != 0
                };
                match (start, on) {
                    (None, true) => start = Some(x),
                    (Some(a), false) => {
                        let a = (a as i32).max(0);
                        let b = (x as i32).min(self.w as i32 - 1);
                        for px in a..=b {
                            self.put(px, y, color);
                        }
                        start = None;
                    }
                    _ => {}
                }
            }
        }
    }

    fn finish(self) -> (u32, u32, Vec<u8>) {
        (self.w as u32, self.h as u32, self.px)
    }
}

struct Map {
    org_x: f32,
    org_y: f32,
    ext_x: f32,
    ext_y: f32,
    w: f32,
    h: f32,
}

impl Map {
    fn map(&self, x: i32, y: i32) -> (i32, i32) {
        let sx = if self.ext_x.abs() < f32::EPSILON {
            1.0
        } else {
            self.w / self.ext_x
        };
        let sy = if self.ext_y.abs() < f32::EPSILON {
            1.0
        } else {
            self.h / self.ext_y
        };
        let px = (x as f32 - self.org_x) * sx;
        let py = (y as f32 - self.org_y) * sy;
        (px.round() as i32, py.round() as i32)
    }
}

fn sized_canvas(bw: i32, bh: i32) -> (usize, usize) {
    let bw = bw.unsigned_abs().max(1) as usize;
    let bh = bh.unsigned_abs().max(1) as usize;
    if bw >= bh {
        let w = bw.min(MAX_SIDE);
        let h = (((bh as u64 * w as u64) / bw as u64).max(1)) as usize;
        (w, h)
    } else {
        let h = bh.min(MAX_SIDE);
        let w = (((bw as u64 * h as u64) / bh as u64).max(1)) as usize;
        (w, h)
    }
}

fn colorref(c: u32) -> [u8; 3] {
    // COLORREF is 0x00BBGGRR; WMF CREATEBRUSHINDIRECT packs hatch in the
    // high byte (image1.bin: `dadada02`). Mask to 24-bit or the CRT fill
    // becomes (218,218,2) instead of gray.
    let c = c & 0x00FF_FFFF;
    [
        (c & 0xFF) as u8,
        ((c >> 8) & 0xFF) as u8,
        ((c >> 16) & 0xFF) as u8,
    ]
}

fn read_u16(data: &[u8], off: usize) -> Option<u16> {
    let b: [u8; 2] = data.get(off..off + 2)?.try_into().ok()?;
    Some(u16::from_le_bytes(b))
}

fn read_i16(data: &[u8], off: usize) -> Option<i16> {
    let b: [u8; 2] = data.get(off..off + 2)?.try_into().ok()?;
    Some(i16::from_le_bytes(b))
}

fn read_u32(data: &[u8], off: usize) -> Option<u32> {
    let b: [u8; 4] = data.get(off..off + 4)?.try_into().ok()?;
    Some(u32::from_le_bytes(b))
}

fn read_i32(data: &[u8], off: usize) -> Option<i32> {
    let b: [u8; 4] = data.get(off..off + 4)?.try_into().ok()?;
    Some(i32::from_le_bytes(b))
}

#[derive(Clone, Copy)]
enum GdiObj {
    Empty,
    Brush([u8; 3]),
    Pen {
        color: [u8; 3],
        width: i32,
    },
    /// A palette, font, region or pattern brush: it holds its table slot
    /// but paints nothing we replay.
    Other,
}

fn raster_wmf(data: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    if data.len() < 40 {
        return None;
    }
    let left = i16::from_le_bytes(data[6..8].try_into().ok()?) as i32;
    let top = i16::from_le_bytes(data[8..10].try_into().ok()?) as i32;
    let right = i16::from_le_bytes(data[10..12].try_into().ok()?) as i32;
    let bottom = i16::from_le_bytes(data[12..14].try_into().ok()?) as i32;
    let (cw, ch) = sized_canvas(right - left, bottom - top);
    let mut canvas = Canvas::new(cw, ch);
    let mut map = Map {
        org_x: left as f32,
        org_y: top as f32,
        ext_x: (right - left) as f32,
        ext_y: (bottom - top) as f32,
        w: cw as f32,
        h: ch as f32,
    };
    let nobj = read_u16(data, 22 + 10).unwrap_or(4) as usize;
    let mut objects = vec![GdiObj::Empty; nobj.clamp(1, 64)];
    let mut brush = [0_u8, 0, 0];
    let mut pen = [0_u8, 0, 0];
    let mut pen_w = 1_i32;
    let mut winding = false;
    let mut off = 22 + 18;
    while off + 6 <= data.len() {
        let size = read_u32(data, off)? as usize;
        let func = read_u16(data, off + 4)?;
        // checked_mul: `size` is untrusted and `usize` is 32-bit on wasm32;
        // comparing against the remaining length keeps `off` from ever
        // moving past (or wrapping around) the buffer end.
        let Some(size2) = size.checked_mul(2) else {
            break;
        };
        if size < 3 || size2 > data.len() - off {
            break;
        }
        let payload = off + 6;
        match func {
            0x0000 => break,
            0x020B => {
                let y = read_i16(data, payload)? as i32;
                let x = read_i16(data, payload + 2)? as i32;
                map.org_x = x as f32;
                map.org_y = y as f32;
            }
            0x020C => {
                let y = read_i16(data, payload)? as i32;
                let x = read_i16(data, payload + 2)? as i32;
                if x != 0 {
                    map.ext_x = x as f32;
                }
                if y != 0 {
                    map.ext_y = y as f32;
                }
            }
            0x02FC => {
                let style = read_u16(data, payload).unwrap_or(0);
                let color = colorref(read_u32(data, payload + 2).unwrap_or(0));
                let slot = objects.iter().position(|o| matches!(o, GdiObj::Empty));
                if let Some(i) = slot {
                    objects[i] = if style == 1 {
                        GdiObj::Brush(WHITE)
                    } else {
                        GdiObj::Brush(color)
                    };
                }
            }
            0x02FA => {
                let color = colorref(read_u32(data, payload + 6).unwrap_or(0));
                let width = read_i16(data, payload + 2).unwrap_or(1) as i32;
                if let Some(i) = objects.iter().position(|o| matches!(o, GdiObj::Empty)) {
                    objects[i] = GdiObj::Pen {
                        color,
                        width: width.max(1),
                    };
                }
            }
            // CreatePalette / PatternBrush / DIBPatternBrush / Font /
            // Region take the lowest free slot like a brush or pen, so
            // later handles count them (Strict01's palette is slot 0).
            0x00F7 | 0x01F9 | 0x0142 | 0x02FB | 0x06FF => {
                if let Some(i) = objects.iter().position(|o| matches!(o, GdiObj::Empty)) {
                    objects[i] = GdiObj::Other;
                }
            }
            0x0106 => winding = read_u16(data, payload).unwrap_or(1) == 2,
            0x012D => {
                // Object handles are 0-based table indices (b88ac900).
                let idx = read_u16(data, payload).unwrap_or(0) as usize;
                if let Some(obj) = objects.get(idx) {
                    match *obj {
                        GdiObj::Brush(c) => brush = c,
                        GdiObj::Pen { color, width } => {
                            pen = color;
                            pen_w = width;
                        }
                        GdiObj::Empty | GdiObj::Other => {}
                    }
                }
            }
            0x01F0 => {
                let idx = read_u16(data, payload).unwrap_or(0) as usize;
                if let Some(slot) = objects.get_mut(idx) {
                    *slot = GdiObj::Empty;
                }
            }
            0x0324 => {
                let n = read_u16(data, payload).unwrap_or(0) as usize;
                let mut pts = Vec::with_capacity(n);
                let mut p = payload + 2;
                for _ in 0..n {
                    let x = read_i16(data, p)? as i32;
                    let y = read_i16(data, p + 2)? as i32;
                    pts.push(map.map(x, y));
                    p += 4;
                }
                canvas.fill_polygon(&pts, brush);
            }
            // META_POLYPOLYGON: ring count, each ring's point count, then
            // every ring's points, filled as one shape (b88ac900's logo).
            0x0538 => {
                let rings = read_u16(data, payload).unwrap_or(0) as usize;
                let mut p = payload + 2 + 2 * rings;
                let mut subpaths = Vec::with_capacity(rings);
                for r in 0..rings {
                    let n = read_u16(data, payload + 2 + 2 * r)? as usize;
                    let mut ring = Vec::with_capacity(n);
                    for _ in 0..n {
                        let x = read_i16(data, p)? as i32;
                        let y = read_i16(data, p + 2)? as i32;
                        ring.push(map.map(x, y));
                        p += 4;
                    }
                    subpaths.push(ring);
                }
                canvas.fill_path(&subpaths, brush, winding);
            }
            0x0325 => {
                let n = read_u16(data, payload).unwrap_or(0) as usize;
                let mut prev: Option<(i32, i32)> = None;
                let mut p = payload + 2;
                for _ in 0..n {
                    let x = read_i16(data, p)? as i32;
                    let y = read_i16(data, p + 2)? as i32;
                    let cur = map.map(x, y);
                    if let Some(pr) = prev {
                        canvas.stroke_line(pr.0, pr.1, cur.0, cur.1, pen, pen_w);
                    }
                    prev = Some(cur);
                    p += 4;
                }
            }
            _ => {}
        }
        off += size2;
    }
    Some(canvas.finish())
}

/// EMF logical → device coordinates (SETMAPMODE / window / viewport).
/// MM_TEXT and the fixed metric modes translate only; MM_ISOTROPIC (7) and
/// MM_ANISOTROPIC (8) also scale window extents onto viewport extents.
struct Xform {
    mode: u32,
    win_org: (f32, f32),
    win_ext: (f32, f32),
    vp_org: (f32, f32),
    vp_ext: (f32, f32),
}

impl Xform {
    fn dev(&self, x: i32, y: i32) -> (i32, i32) {
        let (mut dx, mut dy) = (x as f32 - self.win_org.0, y as f32 - self.win_org.1);
        if matches!(self.mode, 7 | 8)
            && self.win_ext.0.abs() > f32::EPSILON
            && self.win_ext.1.abs() > f32::EPSILON
        {
            dx *= self.vp_ext.0 / self.win_ext.0;
            dy *= self.vp_ext.1 / self.win_ext.1;
        }
        (
            (dx + self.vp_org.0).round() as i32,
            (dy + self.vp_org.1).round() as i32,
        )
    }
}

/// A cubic Bézier from `p0` flattened to line points (excluding `p0`).
fn flatten_bezier(
    p0: (i32, i32),
    c1: (i32, i32),
    c2: (i32, i32),
    p3: (i32, i32),
) -> Vec<(i32, i32)> {
    const STEPS: i32 = 12;
    (1..=STEPS)
        .map(|k| {
            let t = k as f32 / STEPS as f32;
            let u = 1.0 - t;
            let f = |a: i32, b: i32, c: i32, d: i32| {
                u * u * u * a as f32
                    + 3.0 * u * u * t * b as f32
                    + 3.0 * u * t * t * c as f32
                    + t * t * t * d as f32
            };
            (
                f(p0.0, c1.0, c2.0, p3.0).round() as i32,
                f(p0.1, c1.1, c2.1, p3.1).round() as i32,
            )
        })
        .collect()
}

fn raster_emf(data: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    if data.len() < 108 {
        return None;
    }
    let left = read_i32(data, 8)?;
    let top = read_i32(data, 12)?;
    let right = read_i32(data, 16)?;
    let bottom = read_i32(data, 20)?;
    let (cw, ch) = sized_canvas(right.saturating_sub(left), bottom.saturating_sub(top));
    let mut canvas = Canvas::new(cw, ch);
    let map = Map {
        org_x: left as f32,
        org_y: top as f32,
        ext_x: right.saturating_sub(left).max(1) as f32,
        ext_y: bottom.saturating_sub(top).max(1) as f32,
        w: cw as f32,
        h: ch as f32,
    };
    let mut xf = Xform {
        mode: 1,
        win_org: (0.0, 0.0),
        win_ext: (1.0, 1.0),
        vp_org: (0.0, 0.0),
        vp_ext: (1.0, 1.0),
    };
    // Logical point → canvas pixel.
    let px = |xf: &Xform, x: i32, y: i32| {
        let (dx, dy) = xf.dev(x, y);
        map.map(dx, dy)
    };
    let mut objects: HashMap<u32, GdiObj> = HashMap::new();
    let mut brush = [0_u8, 0, 0];
    let mut pen = [0_u8, 0, 0];
    let mut pen_w = 1_i32;
    let mut cx = 0_i32;
    let mut cy = 0_i32;
    let mut winding = false;
    let mut in_path = false;
    // Paths and polylines in logical coordinates.
    let mut subpaths: Vec<Vec<(i32, i32)>> = Vec::new();
    // EMR_CLOSEFIGURE ended the last figure: the next line starts another.
    let mut figure_closed = false;
    let stroke_poly =
        |canvas: &mut Canvas, xf: &Xform, pts: &[(i32, i32)], pen: [u8; 3], w: i32| {
            if w <= 0 {
                return;
            }
            for pair in pts.windows(2) {
                let a = px(xf, pair[0].0, pair[0].1);
                let b = px(xf, pair[1].0, pair[1].1);
                canvas.stroke_line(a.0, a.1, b.0, b.1, pen, w);
            }
        };
    let mut off = read_u32(data, 4)? as usize;
    while off + 8 <= data.len() {
        let typ = read_u32(data, off)?;
        let size = read_u32(data, off + 4)? as usize;
        if size < 8 || size > data.len() - off {
            break;
        }
        match typ {
            14 => break,
            9..=12 if size >= 16 => {
                let v = (
                    read_i32(data, off + 8)? as f32,
                    read_i32(data, off + 12)? as f32,
                );
                match typ {
                    9 => xf.win_ext = v,
                    10 => xf.win_org = v,
                    11 => xf.vp_ext = v,
                    _ => xf.vp_org = v,
                }
            }
            17 if size >= 12 => xf.mode = read_u32(data, off + 8)?,
            19 if size >= 12 => winding = read_u32(data, off + 8)? == 2,
            59 => {
                in_path = true;
                subpaths.clear();
                figure_closed = false;
            }
            // EMR_CLOSEFIGURE: a line back to the figure's start, which
            // becomes the current position.
            61 if in_path => {
                if let Some(sp) = subpaths.last_mut()
                    && sp.len() > 1
                    && let Some(&first) = sp.first()
                {
                    sp.push(first);
                    (cx, cy) = first;
                    figure_closed = true;
                }
            }
            60 => in_path = false,
            27 if size >= 16 => {
                cx = read_i32(data, off + 8)?;
                cy = read_i32(data, off + 12)?;
                if in_path {
                    subpaths.push(vec![(cx, cy)]);
                    figure_closed = false;
                }
            }
            54 if size >= 16 => {
                let x = read_i32(data, off + 8)?;
                let y = read_i32(data, off + 12)?;
                if in_path {
                    if subpaths.is_empty() || figure_closed {
                        subpaths.push(vec![(cx, cy)]);
                        figure_closed = false;
                    }
                    if let Some(sp) = subpaths.last_mut() {
                        sp.push((x, y));
                    }
                } else {
                    stroke_poly(&mut canvas, &xf, &[(cx, cy), (x, y)], pen, pen_w);
                }
                cx = x;
                cy = y;
            }
            // EMR_POLYBEZIERTO(16) / EMR_POLYLINETO(16): continue from the
            // current point; EMR_POLYBEZIER(16) / EMR_POLYLINE(16) start at
            // their first point.
            2 | 4 | 5 | 6 | 85 | 87 | 88 | 89 if size >= 28 => {
                let pts16 = matches!(typ, 85 | 87 | 88 | 89);
                let Some(pts) = read_emf_points(data, off, size, pts16) else {
                    off += size;
                    continue;
                };
                let bezier = matches!(typ, 2 | 5 | 85 | 88);
                let to = matches!(typ, 5 | 6 | 88 | 89);
                let (start, rest) = if to {
                    ((cx, cy), &pts[..])
                } else if let Some((first, rest)) = pts.split_first() {
                    (*first, rest)
                } else {
                    off += size;
                    continue;
                };
                let mut line = vec![start];
                if bezier {
                    let mut p0 = start;
                    for trio in rest.chunks_exact(3) {
                        line.extend(flatten_bezier(p0, trio[0], trio[1], trio[2]));
                        p0 = trio[2];
                    }
                } else {
                    line.extend_from_slice(rest);
                }
                if to && let Some(&(x, y)) = line.last() {
                    cx = x;
                    cy = y;
                }
                if in_path {
                    if to
                        && !figure_closed
                        && let Some(sp) = subpaths.last_mut()
                    {
                        sp.extend_from_slice(&line[1..]);
                    } else {
                        subpaths.push(line);
                    }
                    figure_closed = false;
                } else {
                    stroke_poly(&mut canvas, &xf, &line, pen, pen_w);
                }
            }
            // EMR_FILLPATH / EMR_STROKEANDFILLPATH / EMR_STROKEPATH
            62..=64 => {
                if typ != 64 {
                    let dev: Vec<Vec<(i32, i32)>> = subpaths
                        .iter()
                        .map(|sp| sp.iter().map(|&(x, y)| px(&xf, x, y)).collect())
                        .collect();
                    canvas.fill_path(&dev, brush, winding);
                }
                // GDI strokes each figure as built: only EMR_CLOSEFIGURE
                // closes one.
                if typ != 62 {
                    for sp in &subpaths {
                        stroke_poly(&mut canvas, &xf, sp, pen, pen_w);
                    }
                }
                subpaths.clear();
            }
            37 if size >= 12 => {
                let id = read_u32(data, off + 8)?;
                if id & 0x8000_0000 != 0 {
                    apply_stock(id, &mut brush, &mut pen);
                } else if let Some(obj) = objects.get(&id) {
                    match *obj {
                        GdiObj::Brush(c) => brush = c,
                        GdiObj::Pen { color, width } => {
                            pen = color;
                            pen_w = width;
                        }
                        GdiObj::Empty | GdiObj::Other => {}
                    }
                }
            }
            38 if size >= 28 => {
                let id = read_u32(data, off + 8)?;
                let width = read_i32(data, off + 16).unwrap_or(1);
                let color = colorref(read_u32(data, off + 24).unwrap_or(0));
                objects.insert(
                    id,
                    GdiObj::Pen {
                        color,
                        width: width.max(1),
                    },
                );
            }
            // EMR_EXTCREATEPEN: LOGPEN_EX after the four bitmap fields.
            95 if size >= 44 => {
                let id = read_u32(data, off + 8)?;
                let style = read_u32(data, off + 28).unwrap_or(0);
                let color = colorref(read_u32(data, off + 40).unwrap_or(0));
                // PS_NULL draws nothing.
                let width = if style & 0xF == 5 { 0 } else { 1 };
                objects.insert(id, GdiObj::Pen { color, width });
            }
            39 if size >= 24 => {
                let id = read_u32(data, off + 8)?;
                let style = read_u32(data, off + 12).unwrap_or(0);
                let color = colorref(read_u32(data, off + 16).unwrap_or(0));
                objects.insert(
                    id,
                    if style == 1 {
                        GdiObj::Brush(WHITE)
                    } else {
                        GdiObj::Brush(color)
                    },
                );
            }
            40 if size >= 12 => {
                objects.remove(&read_u32(data, off + 8)?);
            }
            76 if size >= 40 => {
                // EMR_BITBLT — Strict01 uses PATCOPY 1px rules.
                let x = read_i32(data, off + 24)?;
                let y = read_i32(data, off + 28)?;
                let w = read_i32(data, off + 32)?;
                let h = read_i32(data, off + 36)?;
                let a = px(&xf, x, y);
                let b = px(&xf, x.saturating_add(w.max(1)), y.saturating_add(h.max(1)));
                canvas.fill_rect(
                    a.0.min(b.0),
                    a.1.min(b.1),
                    b.0.saturating_sub(a.0).saturating_abs().max(1),
                    b.1.saturating_sub(a.1).saturating_abs().max(1),
                    brush,
                );
            }
            3 | 86 if size >= 28 => {
                // EMR_POLYGON / EMR_POLYGON16
                // Inside a path bracket it is a closed figure of the path and
                // paints nothing until FILLPATH / STROKEPATH.
                if let Some(mut pts) = read_emf_points(data, off, size, typ == 86) {
                    if in_path {
                        if let Some(&first) = pts.first() {
                            pts.push(first);
                        }
                        subpaths.push(pts);
                        figure_closed = true;
                    } else {
                        let mapped: Vec<(i32, i32)> =
                            pts.iter().map(|&(x, y)| px(&xf, x, y)).collect();
                        canvas.fill_polygon(&mapped, brush);
                    }
                }
            }
            _ => {}
        }
        off += size;
    }
    Some(canvas.finish())
}

fn apply_stock(id: u32, brush: &mut [u8; 3], pen: &mut [u8; 3]) {
    match id & 0xFF {
        0 => *brush = WHITE,
        4 => *brush = [0, 0, 0],
        5 => *brush = WHITE,
        6 => *pen = WHITE,
        7 => *pen = [0, 0, 0],
        _ => {}
    }
}

fn read_emf_points(data: &[u8], off: usize, size: usize, pts16: bool) -> Option<Vec<(i32, i32)>> {
    let count = read_u32(data, off + 24)? as usize;
    let mut pts = Vec::with_capacity(count.min(4096));
    let mut p = off + 28;
    for _ in 0..count {
        if pts16 {
            if p + 4 > off + size {
                break;
            }
            let x = read_i16(data, p)? as i32;
            let y = read_i16(data, p + 2)? as i32;
            pts.push((x, y));
            p += 4;
        } else {
            if p + 8 > off + size {
                break;
            }
            pts.push((read_i32(data, p)?, read_i32(data, p + 4)?));
            p += 8;
        }
    }
    Some(pts)
}

#[cfg(test)]
mod hostile_input_tests {
    //! CR PR#4 review: crafted WMF/EMF must terminate quickly without
    //! panicking — coordinate spans and record sizes are attacker-chosen.
    use super::*;

    fn emf_header(left: i32, top: i32, right: i32, bottom: i32) -> Vec<u8> {
        let mut d = vec![0u8; 108];
        d[0..4].copy_from_slice(&1u32.to_le_bytes());
        d[4..8].copy_from_slice(&108u32.to_le_bytes()); // first record offset
        d[8..12].copy_from_slice(&left.to_le_bytes());
        d[12..16].copy_from_slice(&top.to_le_bytes());
        d[16..20].copy_from_slice(&right.to_le_bytes());
        d[20..24].copy_from_slice(&bottom.to_le_bytes());
        d[40..44].copy_from_slice(b" EMF");
        d
    }

    fn rec(d: &mut Vec<u8>, typ: u32, fields: &[i32]) {
        d.extend_from_slice(&typ.to_le_bytes());
        d.extend_from_slice(&((8 + 4 * fields.len()) as u32).to_le_bytes());
        for f in fields {
            d.extend_from_slice(&f.to_le_bytes());
        }
    }

    /// A polygon edge starting at `i32::MIN` after mapping: the scanline
    /// subtraction `y - y0` must not overflow i32 (debug panic / release
    /// wrap-around to a wrong intersection).
    #[test]
    fn emf_polygon_with_extreme_edge_does_not_overflow_scanline() {
        let mut d = emf_header(0, 0, 64, 64);
        // EMR_POLYGON: bounds[4], count, then full-range i32 points.
        let pts: [i32; 8] = [0, i32::MIN, 32, i32::MAX, 64, 0, 0, 0];
        let mut fields: Vec<i32> = vec![0, 0, 64, 64, 4];
        fields.extend_from_slice(&pts);
        rec(&mut d, 3, &fields);
        rec(&mut d, 14, &[]); // EMR_EOF
        assert!(rasterize(&d).is_some());
    }

    /// Full-range header bounds: `right - left` must not overflow.
    #[test]
    fn emf_extreme_header_bounds_terminate() {
        let mut d = emf_header(i32::MIN, i32::MIN, i32::MAX, i32::MAX);
        rec(&mut d, 14, &[]); // EMR_EOF
        assert!(rasterize(&d).is_some());
    }

    /// A line whose mapped endpoints sit ~2^31 pixels apart: the walk must
    /// be clipped to the canvas, and the Bresenham deltas must not overflow.
    #[test]
    fn emf_offcanvas_line_terminates() {
        let big = 1 << 30;
        let mut d = emf_header(0, 0, 384, 384);
        rec(&mut d, 27, &[-big, -big]); // EMR_MOVETOEX
        rec(&mut d, 54, &[big, big]); // EMR_LINETO
        rec(&mut d, 14, &[]);
        assert!(rasterize(&d).is_some());
    }

    /// Polygon with full-range vertices: the scanline intersection product
    /// must be computed in i64 and the span clamped to the canvas.
    #[test]
    fn emf_offcanvas_polygon_terminates() {
        let big = 1 << 30;
        let mut d = emf_header(0, 0, 384, 384);
        // EMR_POLYGON: bounds rect (4 fields), count, then points.
        rec(&mut d, 3, &[0, 0, 0, 0, 3, -big, -big, big, -big, 0, big]);
        rec(&mut d, 14, &[]);
        assert!(rasterize(&d).is_some());
    }

    /// WMF record size near u32::MAX: `size * 2` wraps a 32-bit usize
    /// (wasm32). Natively this documents the guard; the checked_mul keeps
    /// wasm from looping forever on a wrapped offset.
    #[test]
    fn wmf_huge_record_size_terminates() {
        let mut d = vec![0u8; 40];
        d[0..4].copy_from_slice(&[0xD7, 0xCD, 0xC6, 0x9A]);
        d.extend_from_slice(&0x8000_0001u32.to_le_bytes()); // size (words)
        d.extend_from_slice(&0u16.to_le_bytes()); // func
        d.extend_from_slice(&[0u8; 32]);
        assert!(rasterize(&d).is_some());
    }
}

#[cfg(test)]
mod emf_text_tests {
    //! Strict01 OLE previews (image2.emf / image3.emf) store the Excel
    //! grid as EMR_EXTTEXTOUTW digits. Skipping those records leaves
    //! rules without 1–9/12/15/18. Not the xlsx-Calibri-grid ITT-neg.
    use super::*;

    fn emf_header(left: i32, top: i32, right: i32, bottom: i32) -> Vec<u8> {
        let mut d = vec![0u8; 108];
        d[0..4].copy_from_slice(&1u32.to_le_bytes());
        d[4..8].copy_from_slice(&108u32.to_le_bytes());
        d[8..12].copy_from_slice(&left.to_le_bytes());
        d[12..16].copy_from_slice(&top.to_le_bytes());
        d[16..20].copy_from_slice(&right.to_le_bytes());
        d[20..24].copy_from_slice(&bottom.to_le_bytes());
        d[40..44].copy_from_slice(b" EMF");
        d
    }

    fn exttextout_w(d: &mut Vec<u8>, x: i32, y: i32, text: &str) {
        let utf16: Vec<u16> = text.encode_utf16().collect();
        let n = utf16.len() as u32;
        let off_string = 76u32;
        let str_bytes = n as usize * 2;
        let rec_size = (76 + str_bytes).next_multiple_of(4) as u32;
        d.extend_from_slice(&84u32.to_le_bytes());
        d.extend_from_slice(&rec_size.to_le_bytes());
        d.extend_from_slice(&0i32.to_le_bytes()); // bounds
        d.extend_from_slice(&0i32.to_le_bytes());
        d.extend_from_slice(&64i32.to_le_bytes());
        d.extend_from_slice(&64i32.to_le_bytes());
        d.extend_from_slice(&1u32.to_le_bytes()); // iGraphicsMode
        d.extend_from_slice(&0u32.to_le_bytes()); // exScale
        d.extend_from_slice(&0u32.to_le_bytes()); // eyScale
        d.extend_from_slice(&x.to_le_bytes());
        d.extend_from_slice(&y.to_le_bytes());
        d.extend_from_slice(&n.to_le_bytes());
        d.extend_from_slice(&off_string.to_le_bytes());
        d.extend_from_slice(&0u32.to_le_bytes()); // fOptions
        for _ in 0..4 {
            d.extend_from_slice(&0i32.to_le_bytes()); // rcl
        }
        d.extend_from_slice(&0u32.to_le_bytes()); // offDx
        for u in utf16 {
            d.extend_from_slice(&u.to_le_bytes());
        }
        while !d.len().is_multiple_of(4) {
            d.push(0);
        }
    }

    fn dark_samples(rgb: &[u8]) -> usize {
        rgb.chunks(3)
            .filter(|px| px.iter().any(|&c| c < 200))
            .count()
    }

    #[test]
    fn emf_exttextoutw_stays_unpainted_after_mini_365() {
        // Strict01 OLE image2.emf stores 1–9/12/15/18 as EXTTEXTOUTW.
        // 5×7 bitmap digits (mini 365) were Word-shaped but ITT-neg:
        // Strict01 family −0.0056 / NR mean −0.0006 vs Quartz Calibri.
        // Not xlsx Calibri grid (also ITT-neg). Keep rules-only raster.
        let mut d = emf_header(0, 0, 64, 64);
        exttextout_w(&mut d, 8, 8, "8");
        d.extend_from_slice(&14u32.to_le_bytes());
        d.extend_from_slice(&8u32.to_le_bytes());
        let (_, _, rgb) = rasterize(&d).expect("raster EMF text lock");
        assert_eq!(
            dark_samples(&rgb),
            0,
            "mini 365 EXTTEXTOUTW bitmap ITT-neg; dark={}",
            dark_samples(&rgb)
        );
    }
}

#[cfg(test)]
mod emf_path_tests {
    //! fixtures_500 000ebd12: the Riksdag header logo is an EMF of filled
    //! Bézier paths (BEGINPATH … POLYBEZIERTO16 … FILLPATH) under a
    //! window/viewport mapping. We rasterized none of it.
    use super::*;

    #[test]
    fn saturated_edges_cross_where_the_line_does() {
        // PR #167 review: mapped points saturate to i32::MIN/MAX, and the
        // crossing product (y - y0) * (x1 - x0) then passes i64::MAX. The
        // diagonal from (MIN, MIN) to (MAX, MAX) crosses row y at x = y.
        let tri = [
            (i32::MIN, i32::MIN),
            (i32::MAX, i32::MAX),
            (i32::MIN, i32::MAX),
        ];
        let ink = |c: &Canvas, x: usize, y: usize| c.px[(y * c.w + x) * 3] == 0;
        let mut poly = Canvas::new(64, 64);
        poly.fill_polygon(&tri, [0, 0, 0]);
        let mut path = Canvas::new(64, 64);
        path.fill_path(&[tri.to_vec()], [0, 0, 0], false);
        for c in [&poly, &path] {
            assert!(ink(c, 5, 40), "left of the diagonal is inside");
            assert!(!ink(c, 60, 20), "right of the diagonal is outside");
        }
    }

    fn header(left: i32, top: i32, right: i32, bottom: i32) -> Vec<u8> {
        let mut d = vec![0u8; 108];
        d[0..4].copy_from_slice(&1u32.to_le_bytes());
        d[4..8].copy_from_slice(&108u32.to_le_bytes());
        d[8..12].copy_from_slice(&left.to_le_bytes());
        d[12..16].copy_from_slice(&top.to_le_bytes());
        d[16..20].copy_from_slice(&right.to_le_bytes());
        d[20..24].copy_from_slice(&bottom.to_le_bytes());
        d[40..44].copy_from_slice(b" EMF");
        d
    }

    fn rec(d: &mut Vec<u8>, typ: u32, body: &[u8]) {
        d.extend_from_slice(&typ.to_le_bytes());
        d.extend_from_slice(&((8 + body.len()) as u32).to_le_bytes());
        d.extend_from_slice(body);
    }

    fn ints(v: &[i32]) -> Vec<u8> {
        v.iter().flat_map(|x| x.to_le_bytes()).collect()
    }

    /// EMR_POLYBEZIERTO16 / EMR_POLYLINETO16: bounds, count, 16-bit points.
    fn pts16(pts: &[(i16, i16)]) -> Vec<u8> {
        let mut b = ints(&[0, 0, 0, 0, pts.len() as i32]);
        for (x, y) in pts {
            b.extend_from_slice(&x.to_le_bytes());
            b.extend_from_slice(&y.to_le_bytes());
        }
        b
    }

    fn square(d: &mut Vec<u8>, x0: i16, y0: i16, x1: i16, y1: i16) {
        rec(d, 27, &ints(&[i32::from(x0), i32::from(y0)]));
        rec(d, 89, &pts16(&[(x1, y0), (x1, y1), (x0, y1)]));
        rec(d, 61, &[]);
    }

    fn dark(rgb: &[u8], w: u32, x: u32, y: u32) -> bool {
        let i = ((y * w + x) * 3) as usize;
        rgb[i] < 128
    }

    #[test]
    fn a_stroked_path_closes_only_the_figures_closefigure_closes() {
        // PR #167 review: EMR_STROKEPATH closed every subpath, so an open
        // polyline gained a chord back to its start. GDI closes a figure
        // only on EMR_CLOSEFIGURE, which also moves the pen to the
        // figure's start for the next LINETO.
        let stroked = |close: bool| {
            let mut d = header(0, 0, 100, 100);
            rec(&mut d, 59, &[]);
            rec(&mut d, 27, &ints(&[10, 10]));
            rec(&mut d, 54, &ints(&[90, 10]));
            if close {
                rec(&mut d, 61, &[]);
                rec(&mut d, 54, &ints(&[10, 90]));
            } else {
                rec(&mut d, 54, &ints(&[90, 90]));
            }
            rec(&mut d, 60, &[]);
            rec(&mut d, 64, &ints(&[0, 0, 0, 0]));
            rec(&mut d, 14, &ints(&[0, 0, 0]));
            rasterize(&d).expect("raster")
        };
        let (w, _, open) = stroked(false);
        assert!(dark(&open, w, 50, 10), "the open figure's first edge inks");
        assert!(!dark(&open, w, 50, 50), "no chord back to the start");
        let (w, _, closed) = stroked(true);
        assert!(
            dark(&closed, w, 10, 50),
            "after the close, LINETO starts at the figure start"
        );
        assert!(
            !dark(&closed, w, 50, 50),
            "not from the last point (90, 10)"
        );
    }

    #[test]
    fn a_polygon_inside_a_path_bracket_joins_the_path() {
        // PR #167 review: EMR_POLYGON between BEGINPATH and ENDPATH is a
        // closed figure of the path and paints nothing itself; only the
        // path's FILLPATH inks it (a clip-only path never does).
        let polygon_path = |fill: bool| {
            let mut d = header(0, 0, 100, 100);
            rec(&mut d, 39, &ints(&[1, 0, 0, 0]));
            rec(&mut d, 37, &ints(&[1]));
            rec(&mut d, 59, &[]);
            rec(
                &mut d,
                86,
                &pts16(&[(10, 10), (90, 10), (90, 90), (10, 90)]),
            );
            rec(&mut d, 60, &[]);
            if fill {
                rec(&mut d, 62, &ints(&[0, 0, 0, 0]));
            }
            rec(&mut d, 14, &ints(&[0, 0, 0]));
            rasterize(&d).expect("raster")
        };
        let (w, _, unfilled) = polygon_path(false);
        assert!(
            !dark(&unfilled, w, 50, 50),
            "a path polygon paints nothing on its own"
        );
        let (w, _, filled) = polygon_path(true);
        assert!(
            dark(&filled, w, 50, 50),
            "FILLPATH fills the polygon figure"
        );
    }

    #[test]
    fn a_mapped_bezier_path_fills() {
        // Device bounds 0..100; logical window 0..1000 mapped onto it.
        let mut d = header(0, 0, 100, 100);
        rec(&mut d, 17, &ints(&[8])); // MM_ANISOTROPIC
        rec(&mut d, 9, &ints(&[1000, 1000])); // window ext
        rec(&mut d, 11, &ints(&[100, 100])); // viewport ext
        rec(&mut d, 39, &ints(&[1, 0, 0, 0])); // black solid brush #1
        rec(&mut d, 37, &ints(&[1]));
        rec(&mut d, 59, &[]);
        rec(&mut d, 27, &ints(&[100, 500]));
        rec(&mut d, 88, &pts16(&[(100, 100), (900, 100), (900, 500)]));
        rec(&mut d, 88, &pts16(&[(900, 900), (100, 900), (100, 500)]));
        rec(&mut d, 61, &[]);
        rec(&mut d, 60, &[]);
        rec(&mut d, 62, &ints(&[0, 0, 0, 0]));
        rec(&mut d, 14, &ints(&[0, 0, 0]));
        let (w, h, rgb) = rasterize(&d).expect("raster");
        assert!(
            dark(&rgb, w, w / 2, h / 2),
            "the filled path inks its centre"
        );
        assert!(!dark(&rgb, w, 2, 2), "outside the path stays white");
    }

    #[test]
    fn an_even_odd_path_keeps_its_hole() {
        let mut d = header(0, 0, 100, 100);
        rec(&mut d, 19, &ints(&[1])); // ALTERNATE
        rec(&mut d, 39, &ints(&[1, 0, 0, 0]));
        rec(&mut d, 37, &ints(&[1]));
        rec(&mut d, 59, &[]);
        square(&mut d, 10, 10, 90, 90);
        square(&mut d, 40, 40, 60, 60);
        rec(&mut d, 60, &[]);
        rec(&mut d, 62, &ints(&[0, 0, 0, 0]));
        rec(&mut d, 14, &ints(&[0, 0, 0]));
        let (w, h, rgb) = rasterize(&d).expect("raster");
        assert!(dark(&rgb, w, w / 5, h / 2), "the ring inks");
        assert!(!dark(&rgb, w, w / 2, h / 2), "the counter stays open");
    }
}

#[cfg(test)]
mod wmf_object_tests {
    //! English corpus b88ac900: the kennel logo's clipart is a placeable
    //! WMF of META_POLYPOLYGON records selecting brushes by 0-based
    //! object-table index. We skipped the records (a blank picture) and
    //! read index n as slot n-1.
    use super::*;

    fn wmf(records: &[(u16, Vec<u16>)], nobj: u16) -> Vec<u8> {
        let mut d = vec![0u8; 22];
        d[0..4].copy_from_slice(&PLACEABLE_KEY);
        for (i, v) in [0_i16, 0, 100, 100].iter().enumerate() {
            d[6 + 2 * i..8 + 2 * i].copy_from_slice(&v.to_le_bytes());
        }
        d[14..16].copy_from_slice(&1440u16.to_le_bytes());
        let mut header = vec![0u8; 18];
        header[0..2].copy_from_slice(&1u16.to_le_bytes());
        header[2..4].copy_from_slice(&9u16.to_le_bytes());
        header[10..12].copy_from_slice(&nobj.to_le_bytes());
        d.extend_from_slice(&header);
        for (func, params) in records.iter().chain([(0_u16, Vec::new())].iter()) {
            d.extend_from_slice(&(3 + params.len() as u32).to_le_bytes());
            d.extend_from_slice(&func.to_le_bytes());
            for p in params {
                d.extend_from_slice(&p.to_le_bytes());
            }
        }
        d
    }

    fn brush(r: u8, g: u8, b: u8) -> (u16, Vec<u16>) {
        (0x02FC, vec![0, u16::from_le_bytes([r, g]), u16::from(b), 0])
    }

    #[test]
    fn a_polypolygon_fills_with_the_brush_selected_by_zero_based_index() {
        let square = vec![1, 4, 10, 10, 90, 10, 90, 90, 10, 90];
        let d = wmf(
            &[
                brush(255, 0, 0),
                (0x012D, vec![0]),
                // PS_NULL pen into slot 1.
                (0x02FA, vec![5, 0, 0, 0, 0]),
                (0x012D, vec![1]),
                brush(0, 0, 255),
                (0x012D, vec![2]),
                (0x01F0, vec![0]),
                (0x0106, vec![2]),
                (0x0538, square),
            ],
            3,
        );
        let (w, h, px) = rasterize(&d).expect("wmf");
        let at = |x: u32, y: u32| {
            let i = ((y * h / 100) * w + x * w / 100) as usize * 3;
            [px[i], px[i + 1], px[i + 2]]
        };
        assert_eq!(
            at(50, 50),
            [0, 0, 255],
            "the square fills with the blue brush"
        );
        assert_eq!(at(5, 5), [255, 255, 255], "outside stays white");
    }
}

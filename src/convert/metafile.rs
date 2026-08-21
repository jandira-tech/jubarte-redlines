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
                    let dy = i64::from(y1) - i64::from(y0);
                    if dy != 0 {
                        // Full-range i32 coords overflow the i32 product.
                        let x = i64::from(x0)
                            + i64::from(y - y0) * (i64::from(x1) - i64::from(x0)) / dy;
                        xs.push(x.clamp(-1, self.w as i64) as i32);
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
    Pen { color: [u8; 3], width: i32 },
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
            0x012D => {
                let idx = read_u16(data, payload).unwrap_or(0) as usize;
                if let Some(obj) = objects.get(idx) {
                    match *obj {
                        GdiObj::Brush(c) => brush = c,
                        GdiObj::Pen { color, width } => {
                            pen = color;
                            pen_w = width;
                        }
                        GdiObj::Empty => {}
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
    let mut objects: HashMap<u32, GdiObj> = HashMap::new();
    let mut brush = [0_u8, 0, 0];
    let mut pen = [0_u8, 0, 0];
    let mut pen_w = 1_i32;
    let mut cx = 0_i32;
    let mut cy = 0_i32;
    let mut off = read_u32(data, 4)? as usize;
    while off + 8 <= data.len() {
        let typ = read_u32(data, off)?;
        let size = read_u32(data, off + 4)? as usize;
        if size < 8 || size > data.len() - off {
            break;
        }
        match typ {
            14 => break,
            27 if size >= 16 => {
                cx = read_i32(data, off + 8)?;
                cy = read_i32(data, off + 12)?;
            }
            54 if size >= 16 => {
                let x = read_i32(data, off + 8)?;
                let y = read_i32(data, off + 12)?;
                let a = map.map(cx, cy);
                let b = map.map(x, y);
                canvas.stroke_line(a.0, a.1, b.0, b.1, pen, pen_w);
                cx = x;
                cy = y;
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
                        GdiObj::Empty => {}
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
                let a = map.map(x, y);
                let b = map.map(x.saturating_add(w.max(1)), y.saturating_add(h.max(1)));
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
                if let Some(pts) = read_emf_points(data, off, size, typ == 86) {
                    let mapped: Vec<(i32, i32)> = pts.iter().map(|&(x, y)| map.map(x, y)).collect();
                    canvas.fill_polygon(&mapped, brush);
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

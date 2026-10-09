// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Page raster over the layout display list: the same [`Page`]/[`Op`] stream
//! the PDF writer serializes is painted into a bitmap, so a PNG page shows
//! exactly the layout the PDF has, without a PDF round trip or an external
//! rasterizer. Glyphs are filled from the face's TrueType outlines; images
//! are decoded with the same decoders the PDF writer embeds.

use std::collections::HashMap;

use tiny_skia::{
    Color, FillRule, LineCap, LineJoin, Mask, Paint, PathBuilder, Pixmap, PixmapPaint, Rect,
    Stroke, Transform,
};

use super::font::{FaceRef, Fonts, word_device_track};
use super::pdf::{Op, Page, PdfComment, markup_chrome};

/// Points per inch.
const PT_PER_INCH: f32 = 72.0;

/// Largest page raster, in pixels (1 GiB of RGBA): Letter and A4 fit at
/// 1200 dpi. A failed allocation would abort the process, Python host included.
const MAX_PAGE_PIXELS: u64 = 1 << 28;

/// Paint one laid-out page at `dpi` into RGBA pixels. Returns `None` for a
/// degenerate page size or one over [`MAX_PAGE_PIXELS`].
pub(crate) fn paint_page(fonts: &Fonts<'_>, page: &Page, dpi: f32) -> Option<Pixmap> {
    let scale = dpi / PT_PER_INCH;
    let (out_w, out_h) = if page.vertical {
        (page.height, page.width)
    } else {
        (page.width, page.height)
    };
    let width = (out_w * scale).ceil().max(1.0) as u32;
    let height = (out_h * scale).ceil().max(1.0) as u32;
    if u64::from(width) * u64::from(height) > MAX_PAGE_PIXELS {
        return None;
    }
    let mut pixmap = Pixmap::new(width, height)?;
    pixmap.fill(Color::WHITE);
    // PDF user space (origin bottom-left, points) to pixels.
    let mut ts = Transform::from_row(scale, 0.0, 0.0, -scale, 0.0, out_h * scale);
    if page.vertical {
        // The PDF writer turns the laid-out page: X = y, Y = width - x.
        let turn = Transform::from_row(0.0, -1.0, 1.0, 0.0, 0.0, page.width);
        ts = turn.post_concat(ts);
    }
    if page.markup_pane
        && let Some(m) = markup_chrome(page.width, page.height, page.margin_r)
    {
        fill_rect(
            &mut pixmap,
            m.gx,
            m.gy,
            m.gw,
            m.gh,
            [0.949, 0.949, 0.949],
            ts,
        );
        ts = Transform::from_row(m.k, 0.0, 0.0, m.k, m.tx, m.ty).post_concat(ts);
    }
    let mut faces: HashMap<FaceRef, ttf_parser::Face<'_>> = HashMap::new();
    for op in &page.ops {
        paint_op(&mut pixmap, fonts, &mut faces, op, ts);
    }
    for note in &page.comments {
        paint_comment(&mut pixmap, note, ts);
    }
    Some(pixmap)
}

/// PNG bytes of a painted page.
pub(crate) fn encode_png(pixmap: &Pixmap) -> Vec<u8> {
    pixmap.encode_png().unwrap_or_default()
}

/// Text painted on a page, one line per distinct baseline, pieces ordered by
/// x. The layout paints one glyph per op, so a space is written only where
/// the pen jumps by more than a fifth of the font size.
pub(crate) fn page_text(fonts: &Fonts<'_>, page: &Page) -> String {
    // (baseline key, pieces)
    let mut lines: Vec<(i64, Vec<Piece>)> = Vec::new();
    for op in &page.ops {
        let (face, size, x, y, glyphs, hscale, text) = match op {
            Op::Text {
                face,
                size,
                x,
                y,
                glyphs,
                hscale,
                text,
                ..
            } => (*face, *size, *x, *y, glyphs, *hscale, text),
            Op::Watermark {
                face,
                size,
                x,
                y,
                glyphs,
                text,
                ..
            } => (*face, *size, *x, *y, glyphs, 1.0, text),
            _ => continue,
        };
        if text.is_empty() {
            continue;
        }
        let f = fonts.get(face);
        let advance: f32 = glyphs
            .iter()
            .map(|&g| {
                f32::from(f.widths.get(usize::from(g)).copied().unwrap_or(0)) * size / f.upem
                    * hscale
            })
            .sum();
        let key = (y * 4.0).round() as i64;
        let piece = Piece {
            x,
            advance,
            size,
            text: text.clone(),
        };
        match lines.iter_mut().find(|(k, _)| *k == key) {
            Some((_, pieces)) => pieces.push(piece),
            None => lines.push((key, vec![piece])),
        }
    }
    let mut out = String::new();
    for (_, mut pieces) in lines {
        pieces.sort_by(|a, b| a.x.total_cmp(&b.x));
        let mut line = String::new();
        let mut pen: Option<f32> = None;
        for piece in pieces {
            if let Some(pen) = pen
                && piece.x - pen > piece.size * 0.2
                && !line.ends_with(char::is_whitespace)
                && !piece.text.starts_with(char::is_whitespace)
            {
                line.push(' ');
            }
            line.push_str(&piece.text);
            pen = Some(piece.x + piece.advance);
        }
        out.push_str(line.trim_end());
        out.push('\n');
    }
    out
}

/// One painted glyph string on a baseline.
struct Piece {
    x: f32,
    advance: f32,
    size: f32,
    text: String,
}

fn color(c: [f32; 3]) -> Color {
    Color::from_rgba(
        c[0].clamp(0.0, 1.0),
        c[1].clamp(0.0, 1.0),
        c[2].clamp(0.0, 1.0),
        1.0,
    )
    .unwrap_or(Color::BLACK)
}

fn paint_for(c: [f32; 3]) -> Paint<'static> {
    let mut paint = Paint::default();
    paint.set_color(color(c));
    paint.anti_alias = true;
    paint
}

fn stroke_for(width: f32) -> Stroke {
    Stroke {
        width: width.max(0.1),
        line_cap: LineCap::Butt,
        line_join: LineJoin::Miter,
        ..Stroke::default()
    }
}

fn fill_rect(pixmap: &mut Pixmap, x: f32, y: f32, w: f32, h: f32, c: [f32; 3], ts: Transform) {
    if let Some(rect) = Rect::from_xywh(x.min(x + w), y.min(y + h), w.abs(), h.abs()) {
        pixmap.fill_rect(rect, &paint_for(c), ts, None);
    }
}

fn polygon(points: &[(f32, f32)], close: bool) -> Option<tiny_skia::Path> {
    let mut pb = PathBuilder::new();
    let (first, rest) = points.split_first()?;
    pb.move_to(first.0, first.1);
    for (x, y) in rest {
        pb.line_to(*x, *y);
    }
    if close {
        pb.close();
    }
    pb.finish()
}

fn paint_op<'f>(
    pixmap: &mut Pixmap,
    fonts: &'f Fonts<'_>,
    faces: &mut HashMap<FaceRef, ttf_parser::Face<'f>>,
    op: &Op,
    ts: Transform,
) {
    match op {
        Op::Pin(_) => {}
        Op::Text {
            face,
            size,
            x,
            y,
            glyphs,
            color: c,
            hscale,
            ..
        } => paint_text(
            pixmap,
            fonts,
            faces,
            &TextRun {
                face: *face,
                size: *size,
                x: *x,
                y: *y,
                glyphs,
                color: *c,
                hscale: *hscale,
                rotate_deg: 0.0,
            },
            ts,
        ),
        Op::Watermark {
            face,
            size,
            x,
            y,
            glyphs,
            color: c,
            rotate_deg,
            ..
        } => paint_text(
            pixmap,
            fonts,
            faces,
            &TextRun {
                face: *face,
                size: *size,
                x: *x,
                y: *y,
                glyphs,
                color: *c,
                hscale: 1.0,
                rotate_deg: *rotate_deg,
            },
            ts,
        ),
        Op::Line {
            x1,
            y1,
            x2,
            y2,
            width,
            color: c,
        } => {
            if let Some(path) = polygon(&[(*x1, *y1), (*x2, *y2)], false) {
                pixmap.stroke_path(&path, &paint_for(*c), &stroke_for(*width), ts, None);
            }
        }
        Op::FillRect {
            x,
            y,
            w,
            h,
            color: c,
        } => fill_rect(pixmap, *x, *y, *w, *h, *c, ts),
        Op::StrokeRect {
            x,
            y,
            w,
            h,
            width,
            color: c,
        } => {
            if let Some(rect) = Rect::from_xywh(x.min(x + w), y.min(y + h), w.abs(), h.abs()) {
                let path = PathBuilder::from_rect(rect);
                pixmap.stroke_path(&path, &paint_for(*c), &stroke_for(*width), ts, None);
            }
        }
        Op::FillPoly { points, color: c } => {
            if let Some(path) = polygon(points, true) {
                pixmap.fill_path(&path, &paint_for(*c), FillRule::Winding, ts, None);
            }
        }
        Op::StrokePoly {
            points,
            width,
            color: c,
        } => {
            if let Some(path) = polygon(points, true) {
                pixmap.stroke_path(&path, &paint_for(*c), &stroke_for(*width), ts, None);
            }
        }
        Op::FillPath {
            contours,
            color: c,
            even_odd,
        } => {
            let mut pb = PathBuilder::new();
            for contour in contours {
                let Some((first, rest)) = contour.split_first() else {
                    continue;
                };
                pb.move_to(first.0, first.1);
                for (x, y) in rest {
                    pb.line_to(*x, *y);
                }
                pb.close();
            }
            if let Some(path) = pb.finish() {
                let rule = if *even_odd {
                    FillRule::EvenOdd
                } else {
                    FillRule::Winding
                };
                pixmap.fill_path(&path, &paint_for(*c), rule, ts, None);
            }
        }
        Op::StrokePath {
            subpaths,
            width,
            color: c,
        } => {
            let mut pb = PathBuilder::new();
            for (points, close) in subpaths {
                let Some((first, rest)) = points.split_first() else {
                    continue;
                };
                pb.move_to(first.0, first.1);
                for (x, y) in rest {
                    pb.line_to(*x, *y);
                }
                if *close {
                    pb.close();
                }
            }
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &paint_for(*c), &stroke_for(*width), ts, None);
            }
        }
        Op::Cubic {
            start,
            segments,
            width,
            color: c,
        } => {
            let mut pb = PathBuilder::new();
            pb.move_to(start.0, start.1);
            for [c1, c2, end] in segments {
                pb.cubic_to(c1.0, c1.1, c2.0, c2.1, end.0, end.1);
            }
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &paint_for(*c), &stroke_for(*width), ts, None);
            }
        }
        Op::Jpeg {
            x,
            y,
            dw,
            dh,
            bytes,
            crop,
            rotate_deg,
            oval,
            ..
        } => {
            if let Ok(img) = image::load_from_memory(bytes) {
                let rgba = img.to_rgba8();
                let place = Placement {
                    rect: [*x, *y, *dw, *dh],
                    crop: *crop,
                    rotate_deg: *rotate_deg,
                    oval: *oval,
                };
                paint_image(
                    pixmap,
                    (rgba.width(), rgba.height()),
                    rgba.as_raw(),
                    &place,
                    ts,
                );
            }
        }
        Op::Rgb {
            x,
            y,
            dw,
            dh,
            width,
            height,
            bytes,
            alpha,
            crop,
            rotate_deg,
            oval,
        } => {
            let n = (*width as usize) * (*height as usize);
            if bytes.len() < n * 3 {
                return;
            }
            let mut rgba = Vec::with_capacity(n * 4);
            for i in 0..n {
                rgba.extend_from_slice(&bytes[i * 3..i * 3 + 3]);
                rgba.push(
                    alpha
                        .as_ref()
                        .and_then(|a| a.get(i).copied())
                        .unwrap_or(255),
                );
            }
            let place = Placement {
                rect: [*x, *y, *dw, *dh],
                crop: *crop,
                rotate_deg: *rotate_deg,
                oval: *oval,
            };
            paint_image(pixmap, (*width, *height), &rgba, &place, ts);
        }
    }
}

/// One glyph string to paint.
struct TextRun<'g> {
    face: FaceRef,
    size: f32,
    x: f32,
    y: f32,
    glyphs: &'g [u16],
    color: [f32; 3],
    hscale: f32,
    rotate_deg: f32,
}

fn paint_text<'f>(
    pixmap: &mut Pixmap,
    fonts: &'f Fonts<'_>,
    faces: &mut HashMap<FaceRef, ttf_parser::Face<'f>>,
    run: &TextRun<'_>,
    ts: Transform,
) {
    let TextRun {
        face: face_ref,
        size,
        x,
        y,
        glyphs,
        color: c,
        hscale,
        rotate_deg,
    } = *run;
    if glyphs.is_empty() || size <= 0.0 {
        return;
    }
    let face = fonts.get(face_ref);
    let parsed = match faces.entry(face_ref) {
        std::collections::hash_map::Entry::Occupied(e) => e.into_mut(),
        std::collections::hash_map::Entry::Vacant(v) => {
            let Ok(parsed) = ttf_parser::Face::parse(face.bytes(), 0) else {
                return;
            };
            v.insert(parsed)
        }
    };
    let upem = f32::from(parsed.units_per_em().max(1));
    let unit = size / upem;
    let track = word_device_track(size) * hscale;
    let paint = paint_for(c);
    let base = if rotate_deg.abs() > 0.05 {
        Transform::from_rotate(rotate_deg)
            .post_translate(x, y)
            .post_concat(ts)
    } else {
        Transform::from_translate(x, y).post_concat(ts)
    };
    let mut pen = 0.0f32;
    for &gid in glyphs {
        let glyph = ttf_parser::GlyphId(gid);
        let mut builder = OutlineSink::default();
        if parsed.outline_glyph(glyph, &mut builder).is_some()
            && let Some(path) = builder.pb.finish()
        {
            let glyph_ts =
                Transform::from_row(unit * hscale, 0.0, 0.0, unit, pen, 0.0).post_concat(base);
            pixmap.fill_path(&path, &paint, FillRule::Winding, glyph_ts, None);
        }
        let advance = parsed
            .glyph_hor_advance(glyph)
            .map_or(0.0, |a| f32::from(a) * unit * hscale);
        pen += advance + track;
    }
}

#[derive(Default)]
struct OutlineSink {
    pb: PathBuilder,
}

impl ttf_parser::OutlineBuilder for OutlineSink {
    fn move_to(&mut self, x: f32, y: f32) {
        self.pb.move_to(x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.pb.line_to(x, y);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.pb.quad_to(x1, y1, x, y);
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.pb.cubic_to(x1, y1, x2, y2, x, y);
    }
    fn close(&mut self) {
        self.pb.close();
    }
}

/// Where a picture goes: its user-space box plus the writer's crop,
/// rotation and oval-clip flags.
struct Placement {
    rect: [f32; 4],
    crop: Option<[f32; 4]>,
    rotate_deg: f32,
    oval: bool,
}

/// Draw `rgba` (`iw`×`ih`, row-major) into `place` with the PDF writer's
/// crop, rotation and oval-clip semantics.
fn paint_image(
    pixmap: &mut Pixmap,
    (iw, ih): (u32, u32),
    rgba: &[u8],
    place: &Placement,
    ts: Transform,
) {
    let Placement {
        rect: [x, y, dw, dh],
        crop,
        rotate_deg,
        oval,
    } = *place;
    if iw == 0 || ih == 0 || dw <= 0.0 || dh <= 0.0 {
        return;
    }
    let Some(mut src) = Pixmap::new(iw, ih) else {
        return;
    };
    let dst = src.data_mut();
    let n = dst.len().min(rgba.len());
    // tiny-skia stores premultiplied RGBA.
    for i in (0..n).step_by(4) {
        let a = u32::from(rgba[i + 3]);
        dst[i] = ((u32::from(rgba[i]) * a + 127) / 255) as u8;
        dst[i + 1] = ((u32::from(rgba[i + 1]) * a + 127) / 255) as u8;
        dst[i + 2] = ((u32::from(rgba[i + 2]) * a + 127) / 255) as u8;
        dst[i + 3] = a as u8;
    }
    // Rotation about the box centre, as `rotate_about_centre` in the writer.
    let mut base = ts;
    if rotate_deg.abs() > 0.05 {
        let (cx, cy) = (x + dw * 0.5, y + dh * 0.5);
        base = Transform::from_translate(-cx, -cy)
            .post_concat(Transform::from_rotate(rotate_deg))
            .post_translate(cx, cy)
            .post_concat(ts);
    }
    // Image pixel space (y down) to the destination box (y up).
    let (sx, sy, x0, y0) = match crop {
        Some([l, t, r, b]) if l.abs() + r.abs() + t.abs() + b.abs() > 0.001 => {
            let fw = (1.0 - l - r).max(0.001);
            let fh = (1.0 - t - b).max(0.001);
            let sx = dw / fw;
            let sy = dh / fh;
            (sx, sy, x - sx * l, y - sy * b)
        }
        _ => (dw, dh, x, y),
    };
    let image_ts = Transform::from_row(sx / iw as f32, 0.0, 0.0, -(sy / ih as f32), x0, y0 + sy)
        .post_concat(base);
    // Clip to the box (crop) or to its inscribed ellipse (oval).
    let mask = {
        let mut pb = PathBuilder::new();
        if oval {
            if let Some(rect) = Rect::from_xywh(x, y, dw, dh) {
                pb.push_oval(rect);
            }
        } else if let Some(rect) = Rect::from_xywh(x, y, dw, dh) {
            pb.push_rect(rect);
        }
        pb.finish().and_then(|path| {
            let mut mask = Mask::new(pixmap.width(), pixmap.height())?;
            mask.fill_path(&path, FillRule::Winding, true, base);
            Some(mask)
        })
    };
    let paint = PixmapPaint {
        quality: tiny_skia::FilterQuality::Bilinear,
        ..PixmapPaint::default()
    };
    pixmap.draw_pixmap(0, 0, src.as_ref(), &paint, image_ts, mask.as_ref());
}

/// A comment balloon anchor: a light yellow box where the PDF puts its
/// sticky-note annotation, so a reviewer sees that a comment exists here.
fn paint_comment(pixmap: &mut Pixmap, note: &PdfComment, ts: Transform) {
    let Some(rect) = Rect::from_xywh(note.x, note.y, note.w.max(1.0), note.h.max(1.0)) else {
        return;
    };
    let path = PathBuilder::from_rect(rect);
    let mut fill = Paint::default();
    fill.set_color(Color::from_rgba(1.0, 0.95, 0.6, 0.7).unwrap_or(Color::WHITE));
    pixmap.fill_path(&path, &fill, FillRule::Winding, ts, None);
    pixmap.stroke_path(
        &path,
        &paint_for([0.8, 0.6, 0.0]),
        &stroke_for(0.75),
        ts,
        None,
    );
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::convert::font::FaceId;

    fn page_with(ops: Vec<Op>) -> Page {
        let mut page = Page::new(200.0, 100.0);
        page.ops = ops;
        page
    }

    fn pixel(pixmap: &Pixmap, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * pixmap.width() + x) * 4) as usize;
        let d = pixmap.data();
        [d[i], d[i + 1], d[i + 2], d[i + 3]]
    }

    #[test]
    fn page_size_follows_dpi_and_background_is_white() {
        let fonts = Fonts::new();
        let page = page_with(Vec::new());
        let px = paint_page(&fonts, &page, 72.0).unwrap();
        assert_eq!((px.width(), px.height()), (200, 100));
        let px = paint_page(&fonts, &page, 144.0).unwrap();
        assert_eq!((px.width(), px.height()), (400, 200));
        assert_eq!(pixel(&px, 10, 10), [255, 255, 255, 255]);
    }

    #[test]
    fn fill_rect_maps_pdf_bottom_left_origin_to_top_left_pixels() {
        let fonts = Fonts::new();
        // A 20x10 black box at the PDF origin (bottom-left corner).
        let page = page_with(vec![Op::FillRect {
            x: 0.0,
            y: 0.0,
            w: 20.0,
            h: 10.0,
            color: [0.0, 0.0, 0.0],
        }]);
        let px = paint_page(&fonts, &page, 72.0).unwrap();
        assert_eq!(pixel(&px, 5, 95), [0, 0, 0, 255], "bottom-left in pixels");
        assert_eq!(
            pixel(&px, 5, 5),
            [255, 255, 255, 255],
            "top-left stays white"
        );
    }

    #[test]
    fn vertical_pages_are_turned_and_swap_dimensions() {
        let fonts = Fonts::new();
        let mut page = page_with(vec![Op::FillRect {
            x: 0.0,
            y: 0.0,
            w: 20.0,
            h: 10.0,
            color: [0.0, 0.0, 0.0],
        }]);
        page.vertical = true;
        let px = paint_page(&fonts, &page, 72.0).unwrap();
        assert_eq!((px.width(), px.height()), (100, 200));
        // X = y, Y = width - x: the box lands at X∈[0,10], Y∈[180,200] of a
        // 100×200pt page (PDF space, y up), i.e. its top-left in pixels.
        assert_eq!(pixel(&px, 5, 5), [0, 0, 0, 255]);
        assert_eq!(pixel(&px, 5, 195), [255, 255, 255, 255]);
    }

    #[test]
    fn text_paints_dark_pixels_where_glyphs_sit() {
        let fonts = Fonts::new();
        let face = FaceRef::Catalogue(FaceId::CarlitoRegular);
        let glyphs = fonts.get(face).glyphs("MMMM");
        let page = page_with(vec![Op::Text {
            face,
            size: 40.0,
            x: 10.0,
            y: 20.0,
            glyphs,
            color: [0.0, 0.0, 0.0],
            text: "MMMM".into(),
            hscale: 1.0,
        }]);
        let px = paint_page(&fonts, &page, 72.0).unwrap();
        // Some pixel in the glyph band (baseline at PDF y=20 → pixel row 80,
        // cap height above it) is dark; far right of the text stays white.
        let band: Vec<u8> = (60..80)
            .flat_map(|row| (10..40).map(move |col| (col, row)))
            .map(|(c, r)| pixel(&px, c, r)[0])
            .collect();
        assert!(band.iter().any(|&v| v < 128), "glyph ink present");
        assert_eq!(pixel(&px, 190, 70), [255, 255, 255, 255]);
        assert!(!encode_png(&px).is_empty());
        assert_eq!(&encode_png(&px)[1..4], b"PNG");
    }

    #[test]
    fn page_text_groups_by_baseline_and_spaces_only_real_gaps() {
        let fonts = Fonts::new();
        let face = FaceRef::Catalogue(FaceId::CarlitoRegular);
        let f = fonts.get(face);
        let size = 10.0;
        let text = |x: f32, y: f32, s: &str| Op::Text {
            face,
            size,
            x,
            y,
            glyphs: f.glyphs(s),
            color: [0.0; 3],
            text: s.into(),
            hscale: 1.0,
        };
        let w = |s: &str| f.width_pt(s, size);
        // "Hello" then "world" a space-width apart on one baseline; a second
        // baseline painted glyph by glyph with no gaps.
        let hello_end = 72.0 + w("Hello");
        let mut ops = vec![
            text(hello_end + w(" "), 700.0, "world"),
            text(72.0, 700.0, "Hello"),
        ];
        let mut x = 72.0;
        for ch in "Second".chars() {
            let s = ch.to_string();
            ops.push(text(x, 680.0, &s));
            x += w(&s);
        }
        let page = page_with(ops);
        assert_eq!(page_text(&fonts, &page), "Hello world\nSecond\n");
    }

    #[test]
    fn images_and_shapes_paint_without_panicking() {
        let fonts = Fonts::new();
        let rgb: Vec<u8> = std::iter::repeat_n([255u8, 0, 0], 4).flatten().collect();
        let mut page = page_with(vec![
            Op::Rgb {
                x: 10.0,
                y: 10.0,
                dw: 50.0,
                dh: 50.0,
                width: 2,
                height: 2,
                bytes: rgb,
                alpha: None,
                crop: Some([0.1, 0.1, 0.1, 0.1]),
                rotate_deg: 30.0,
                oval: true,
            },
            Op::Line {
                x1: 0.0,
                y1: 0.0,
                x2: 200.0,
                y2: 100.0,
                width: 2.0,
                color: [0.0, 0.0, 1.0],
            },
            Op::StrokeRect {
                x: 100.0,
                y: 50.0,
                w: 40.0,
                h: 20.0,
                width: 1.0,
                color: [0.0; 3],
            },
            Op::FillPoly {
                points: vec![(150.0, 10.0), (190.0, 10.0), (170.0, 40.0)],
                color: [0.0, 1.0, 0.0],
            },
            Op::StrokePoly {
                points: vec![(150.0, 50.0), (190.0, 50.0), (170.0, 90.0)],
                width: 1.0,
                color: [0.0; 3],
            },
            Op::FillPath {
                contours: vec![
                    vec![(10.0, 60.0), (60.0, 60.0), (60.0, 90.0), (10.0, 90.0)],
                    vec![(20.0, 70.0), (50.0, 70.0), (50.0, 80.0), (20.0, 80.0)],
                ],
                color: [0.0; 3],
                even_odd: true,
            },
            Op::StrokePath {
                subpaths: vec![(vec![(70.0, 60.0), (90.0, 90.0)], false)],
                width: 1.0,
                color: [0.0; 3],
            },
            Op::Cubic {
                start: (0.0, 50.0),
                segments: vec![[(50.0, 100.0), (150.0, 0.0), (200.0, 50.0)]],
                width: 1.0,
                color: [0.0; 3],
            },
            Op::Jpeg {
                x: 0.0,
                y: 0.0,
                dw: 10.0,
                dh: 10.0,
                width: 1,
                height: 1,
                bytes: vec![0, 1, 2],
                components: 3,
                crop: None,
                rotate_deg: 0.0,
                oval: false,
            },
        ]);
        page.comments.push(PdfComment {
            id: "0".into(),
            x: 180.0,
            y: 80.0,
            w: 15.0,
            h: 15.0,
            top: 70.0,
            bottom: 83.0,
            contents: "note".into(),
            author: "a".into(),
            initials: "a".into(),
            label: "1".into(),
            color: [0.0; 3],
            resolved: false,
        });
        page.markup_pane = true;
        page.margin_r = 72.0;
        let px = paint_page(&fonts, &page, 72.0).unwrap();
        // The even-odd hole stays white; the red image and the pane painted.
        assert_eq!(pixel(&px, 35, 25), [255, 255, 255, 255]);
        let has_red = (0..px.width())
            .flat_map(|c| (0..px.height()).map(move |r| (c, r)))
            .any(|(c, r)| {
                let p = pixel(&px, c, r);
                p[0] > 200 && p[1] < 80 && p[2] < 80
            });
        assert!(has_red, "the RGB image was painted");
    }
}
#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod raster_payload_boundary_tests {
    use super::*;

    fn white() -> Pixmap {
        let mut p = Pixmap::new(40, 40).unwrap();
        p.fill(Color::WHITE);
        p
    }
    fn sample(p: &Pixmap, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * p.width() + x) * 4) as usize;
        p.data()[i..i + 4].try_into().unwrap()
    }

    #[test]
    fn authored_image_crop_rotation_and_oval_keep_owned_center_and_clip_corners() {
        let bytes: Vec<_> = std::iter::repeat_n([255, 0, 0, 255], 4).flatten().collect();
        for crop in [None, Some([0.0; 4]), Some([0.25; 4])] {
            for rotate_deg in [0.0, 0.01, 180.0] {
                for oval in [false, true] {
                    let mut p = white();
                    paint_image(
                        &mut p,
                        (2, 2),
                        &bytes,
                        &Placement {
                            rect: [10.0, 10.0, 20.0, 20.0],
                            crop,
                            rotate_deg,
                            oval,
                        },
                        Transform::identity(),
                    );
                    assert_eq!(sample(&p, 20, 20), [255, 0, 0, 255]);
                    assert_eq!(sample(&p, 2, 2), [255; 4]);
                    if oval {
                        assert_eq!(sample(&p, 10, 10), [255; 4]);
                    } else {
                        assert_eq!(sample(&p, 12, 12), [255, 0, 0, 255]);
                    }
                }
            }
        }
    }

    #[test]
    fn image_alpha_composites_into_white_and_degenerate_placement_is_identity() {
        for alpha in [0, 128, 255] {
            let mut p = white();
            paint_image(
                &mut p,
                (1, 1),
                &[255, 0, 0, alpha],
                &Placement {
                    rect: [10.0, 10.0, 20.0, 20.0],
                    crop: None,
                    rotate_deg: 0.0,
                    oval: false,
                },
                Transform::identity(),
            );
            assert_eq!(sample(&p, 20, 20), [255, 255 - alpha, 255 - alpha, 255]);
        }
        for (w, h, dw, dh) in [
            (0, 1, 20.0, 20.0),
            (1, 0, 20.0, 20.0),
            (1, 1, 0.0, 20.0),
            (1, 1, 20.0, 0.0),
            (1, 1, -1.0, 20.0),
        ] {
            let mut p = white();
            let before = p.data().to_vec();
            paint_image(
                &mut p,
                (w, h),
                &[255, 0, 0, 255],
                &Placement {
                    rect: [10.0, 10.0, dw, dh],
                    crop: None,
                    rotate_deg: 0.0,
                    oval: false,
                },
                Transform::identity(),
            );
            assert_eq!(p.data(), before);
        }
    }

    #[test]
    fn rgb_missing_alpha_defaults_only_missing_pixels_to_opaque() {
        let fonts = Fonts::new();
        for alpha in [None, Some(vec![]), Some(vec![0]), Some(vec![0, 128])] {
            let mut p = white();
            let mut faces = HashMap::new();
            let op = Op::Rgb {
                x: 10.0,
                y: 10.0,
                dw: 20.0,
                dh: 20.0,
                width: 2,
                height: 1,
                bytes: vec![255, 0, 0, 255, 0, 0],
                alpha: alpha.clone(),
                crop: None,
                rotate_deg: 0.0,
                oval: false,
            };
            paint_op(&mut p, &fonts, &mut faces, &op, Transform::identity());
            let left = alpha
                .as_ref()
                .and_then(|a| a.first())
                .copied()
                .unwrap_or(255);
            let right = alpha
                .as_ref()
                .and_then(|a| a.get(1))
                .copied()
                .unwrap_or(255);
            assert_eq!(sample(&p, 13, 20), [255, 255 - left, 255 - left, 255]);
            assert_eq!(sample(&p, 27, 20), [255, 255 - right, 255 - right, 255]);
        }
        let mut p = white();
        let before = p.data().to_vec();
        let mut faces = HashMap::new();
        paint_op(
            &mut p,
            &fonts,
            &mut faces,
            &Op::Rgb {
                x: 0.0,
                y: 0.0,
                dw: 20.0,
                dh: 20.0,
                width: 2,
                height: 1,
                bytes: vec![255, 0, 0],
                alpha: None,
                crop: None,
                rotate_deg: 0.0,
                oval: false,
            },
            Transform::identity(),
        );
        assert_eq!(p.data(), before);
    }

    #[test]
    fn page_allocation_limit_rejects_before_large_raster_allocation() {
        let fonts = Fonts::new();
        let page = Page::new(20_000.0, 20_000.0);
        assert!(paint_page(&fonts, &page, 72.0).is_none());
    }

    #[test]
    fn empty_shape_payloads_preserve_pixels_and_negative_rectangle_extents_are_normalized() {
        let fonts = Fonts::new();
        let mut faces = HashMap::new();
        let mut p = white();
        let before = p.data().to_vec();
        let ops = [
            Op::FillPoly {
                points: vec![],
                color: [0.0; 3],
            },
            Op::StrokePoly {
                points: vec![],
                width: 1.0,
                color: [0.0; 3],
            },
            Op::FillPath {
                contours: vec![vec![]],
                color: [0.0; 3],
                even_odd: true,
            },
            Op::StrokePath {
                subpaths: vec![(vec![], true)],
                width: 1.0,
                color: [0.0; 3],
            },
            Op::FillRect {
                x: 10.0,
                y: 10.0,
                w: 0.0,
                h: 20.0,
                color: [0.0; 3],
            },
        ];
        for (index, op) in ops.into_iter().enumerate() {
            paint_op(&mut p, &fonts, &mut faces, &op, Transform::identity());
            assert!(p.data() == before, "empty shape {index} changed pixels");
        }
        // A zero-width stroked rectangle is a vertical line; its fill is empty.
        let line = Op::StrokeRect {
            x: 10.0,
            y: 10.0,
            w: 0.0,
            h: 20.0,
            width: 2.0,
            color: [0.0; 3],
        };
        paint_op(&mut p, &fonts, &mut faces, &line, Transform::identity());
        assert_eq!(sample(&p, 9, 20), [0, 0, 0, 255]);
        assert_eq!(sample(&p, 10, 20), [0, 0, 0, 255]);
        assert_eq!(sample(&p, 8, 20), [255; 4]);
        assert_eq!(sample(&p, 11, 20), [255; 4]);
        fill_rect(
            &mut p,
            30.0,
            30.0,
            -20.0,
            -20.0,
            [1.0, 0.0, 0.0],
            Transform::identity(),
        );
        assert_eq!(sample(&p, 20, 20), [255, 0, 0, 255]);
        assert_eq!(sample(&p, 2, 2), [255; 4]);
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod residual_display_contract_tests {
    use super::*;
    use crate::convert::font::FaceId;

    fn bundled_fonts() -> Fonts<'static> {
        let mut fonts = Fonts::new();
        fonts.insert_embedded(
            "ResidualBundledCarlito",
            false,
            false,
            FaceId::CarlitoRegular.bytes(),
        );
        fonts
    }

    fn white() -> Pixmap {
        let mut p = Pixmap::new(40, 40).unwrap();
        p.fill(Color::WHITE);
        p
    }

    fn pixel(p: &Pixmap, x: u32, y: u32) -> [u8; 4] {
        let offset = ((y * p.width() + x) * 4) as usize;
        p.data()[offset..offset + 4].try_into().unwrap()
    }

    #[test]
    fn empty_glyphs_and_nonpositive_display_sizes_never_load_or_paint_a_face() {
        let fonts = bundled_fonts();
        let face = FaceRef::Embedded(0);
        let owned = fonts.get(face).glyphs("A");
        for (size, glyphs) in [
            (12.0, Vec::new()),
            (0.0, owned.clone()),
            (-1.0, owned.clone()),
        ] {
            // Nonpositive display sizes are explicit rejected display input;
            // empty glyph payload also occurs for nonpainting source runs.
            let mut p = white();
            let before = p.data().to_vec();
            let mut faces = HashMap::new();
            paint_text(
                &mut p,
                &fonts,
                &mut faces,
                &TextRun {
                    face,
                    size,
                    x: 10.0,
                    y: 10.0,
                    glyphs: &glyphs,
                    color: [0.0; 3],
                    hscale: 1.0,
                    rotate_deg: 0.0,
                },
                Transform::identity(),
            );
            assert_eq!(p.data(), before);
            assert!(faces.is_empty());
            assert_eq!(
                glyphs,
                if size > 0.0 {
                    Vec::new()
                } else {
                    owned.clone()
                }
            );
        }
        // A space has a real advance and no outline: no ink is fabricated.
        let space = fonts.get(face).glyphs(" ");
        let mut p = white();
        let before = p.data().to_vec();
        let mut faces = HashMap::new();
        paint_text(
            &mut p,
            &fonts,
            &mut faces,
            &TextRun {
                face,
                size: 12.0,
                x: 10.0,
                y: 10.0,
                glyphs: &space,
                color: [0.0; 3],
                hscale: 1.0,
                rotate_deg: 0.0,
            },
            Transform::identity(),
        );
        assert_eq!(p.data(), before);
        assert_eq!(faces.len(), 1);
        assert!(fonts.get(face).width_pt(" ", 12.0) > 0.0);
    }

    #[test]
    fn source_text_extraction_keeps_owned_spaces_watermarks_and_scaled_gap_thresholds() {
        let fonts = bundled_fonts();
        let face = FaceRef::Embedded(0);
        let f = fonts.get(face);
        let text = |x, y, s: &str, hscale| Op::Text {
            face,
            size: 10.0,
            x,
            y,
            glyphs: f.glyphs(s),
            color: [0.0; 3],
            text: s.into(),
            hscale,
        };
        for (left, right, expected) in [
            ("A ", "B", "A B\n"),
            ("A", " B", "A B\n"),
            ("A", "B", "A B\n"),
        ] {
            let scaled_advance = f.width_pt(left, 10.0) * 0.5;
            let mut page = Page::new(40.0, 40.0);
            // Paint order is intentionally reversed; source pieces share a
            // baseline and are reconstructed in x order with one separator.
            page.ops = vec![
                text(5.0 + scaled_advance + 3.0, 20.0, right, 1.0),
                text(5.0, 20.0, left, 0.5),
                text(0.0, 30.0, "", 1.0),
                Op::Pin(true),
                Op::Pin(false),
            ];
            assert_eq!(page_text(&fonts, &page), expected);
            assert_eq!(page.ops.len(), 5);
        }
        let mut page = Page::new(40.0, 40.0);
        page.ops = vec![
            Op::Watermark {
                face,
                size: 10.0,
                x: 2.0,
                y: 30.0,
                glyphs: f.glyphs("Draft"),
                color: [0.5; 3],
                text: "Draft".into(),
                rotate_deg: 30.0,
            },
            text(2.0, 10.0, "Body ", 1.0),
        ];
        assert_eq!(page_text(&fonts, &page), "Draft\nBody\n");
    }

    #[test]
    fn collapsed_vector_subpaths_and_rejected_encoded_picture_payloads_preserve_all_pixels() {
        let fonts = bundled_fonts();
        let mut faces = HashMap::new();
        let mut p = white();
        let before = p.data().to_vec();
        let ops = [
            Op::FillPoly {
                points: vec![(12.0, 12.0)],
                color: [1.0, 0.0, 0.0],
            },
            Op::StrokePoly {
                points: vec![(12.0, 12.0)],
                width: 1.0,
                color: [1.0, 0.0, 0.0],
            },
            Op::FillPath {
                contours: vec![vec![(12.0, 12.0)]],
                color: [1.0, 0.0, 0.0],
                even_odd: false,
            },
            Op::StrokePath {
                subpaths: vec![(vec![(12.0, 12.0)], false)],
                width: 1.0,
                color: [1.0, 0.0, 0.0],
            },
            Op::Cubic {
                start: (12.0, 12.0),
                segments: Vec::new(),
                width: 1.0,
                color: [1.0, 0.0, 0.0],
            },
            Op::Pin(true),
            Op::Pin(false),
        ];
        for (index, op) in ops.iter().enumerate() {
            paint_op(&mut p, &fonts, &mut faces, op, Transform::identity());
            assert_eq!(p.data(), before, "collapsed display path {index}");
        }
        for bytes in [
            Vec::new(),
            b"not an encoded image".to_vec(),
            vec![0x89, b'P', b'N', b'G'],
        ] {
            // Explicit malformed encoded-image diagnostic, not fake RGBA.
            let op = Op::Jpeg {
                x: 10.0,
                y: 10.0,
                dw: 20.0,
                dh: 20.0,
                width: 1,
                height: 1,
                bytes: bytes.clone(),
                components: 3,
                crop: None,
                rotate_deg: 0.0,
                oval: false,
            };
            paint_op(&mut p, &fonts, &mut faces, &op, Transform::identity());
            assert_eq!(p.data(), before);
            if let Op::Jpeg { bytes: after, .. } = op {
                assert_eq!(after, bytes);
            }
        }
    }

    #[test]
    fn compound_fill_rules_and_open_or_closed_strokes_have_independent_literal_geometry() {
        let fonts = bundled_fonts();
        let outer = vec![(5.0, 5.0), (35.0, 5.0), (35.0, 35.0), (5.0, 35.0)];
        let inner = vec![(15.0, 15.0), (25.0, 15.0), (25.0, 25.0), (15.0, 25.0)];
        for even_odd in [false, true] {
            let mut p = white();
            let mut faces = HashMap::new();
            paint_op(
                &mut p,
                &fonts,
                &mut faces,
                &Op::FillPath {
                    contours: vec![outer.clone(), inner.clone()],
                    color: [1.0, 0.0, 0.0],
                    even_odd,
                },
                Transform::identity(),
            );
            assert_eq!(pixel(&p, 10, 20), [255, 0, 0, 255]);
            assert_eq!(
                pixel(&p, 20, 20),
                if even_odd { [255; 4] } else { [255, 0, 0, 255] }
            );
            assert_eq!(pixel(&p, 2, 2), [255; 4]);
        }
        for close in [false, true] {
            let mut p = white();
            let mut faces = HashMap::new();
            paint_op(
                &mut p,
                &fonts,
                &mut faces,
                &Op::StrokePath {
                    subpaths: vec![(vec![(10.0, 10.0), (30.0, 10.0), (30.0, 30.0)], close)],
                    width: 2.0,
                    color: [0.0; 3],
                },
                Transform::identity(),
            );
            assert_eq!(pixel(&p, 20, 10), [0, 0, 0, 255]);
            assert_eq!(pixel(&p, 30, 20), [0, 0, 0, 255]);
            assert_eq!(
                pixel(&p, 20, 20),
                if close { [0, 0, 0, 255] } else { [255; 4] }
            );
            assert_eq!(pixel(&p, 2, 2), [255; 4]);
        }
    }

    #[test]
    fn memory_png_roundtrip_preserves_exact_unrotated_page_size_and_pixels() {
        let fonts = bundled_fonts();
        let mut page = Page::new(40.0, 20.0);
        page.ops = vec![Op::FillRect {
            x: 5.0,
            y: 5.0,
            w: 10.0,
            h: 5.0,
            color: [1.0, 0.0, 0.0],
        }];
        let painted = paint_page(&fonts, &page, 72.0).unwrap();
        assert_eq!((painted.width(), painted.height()), (40, 20));
        assert_eq!(pixel(&painted, 8, 12), [255, 0, 0, 255]);
        assert_eq!(pixel(&painted, 8, 2), [255; 4]);
        let png = encode_png(&painted);
        let before = png.clone();
        assert_eq!(&png[..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
        let decoded = image::load_from_memory(&png).unwrap().to_rgba8();
        assert_eq!((decoded.width(), decoded.height()), (40, 20));
        assert_eq!(decoded.as_raw(), painted.data());
        assert_eq!(png, before);
        assert_eq!(page.ops.len(), 1);
    }
}

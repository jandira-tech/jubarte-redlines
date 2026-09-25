// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! PDF 1.4 writer: embedded TTF (Identity-H), stroked rules, JPEG/RGB images.

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::io::Write;

use flate2::Compression;
use flate2::write::ZlibEncoder;

use super::PdfOptions;
use super::font::{FaceId, FaceRef, Fonts, word_device_paint, word_device_track};

/// One drawing command on a page (PDF user space, origin bottom-left).
pub(crate) enum Op {
    Text {
        face: FaceRef,
        size: f32,
        x: f32,
        y: f32,
        glyphs: Vec<u16>,
        color: [f32; 3],
        /// Source characters. When every char is WinAnsi, the writer emits a
        /// simple TrueType font like Word Quartz (hinted by MuPDF). Empty or
        /// non-WinAnsi text stays on Identity-H CID.
        text: String,
        /// `w:w` horizontal scale of the glyphs (1.0 = none).
        hscale: f32,
    },
    Line {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        width: f32,
        color: [f32; 3],
    },
    FillRect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: [f32; 3],
    },
    /// Closed rectangle stroke (Word SmartArt connector bars `re S`).
    StrokeRect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        width: f32,
        color: [f32; 3],
    },
    FillPoly {
        points: Vec<(f32, f32)>,
        color: [f32; 3],
    },
    /// Closed polygon stroke (Strict01 rightArrow lnRef shade outline).
    StrokePoly {
        points: Vec<(f32, f32)>,
        width: f32,
        color: [f32; 3],
    },
    /// Filled compound path (nonzero): every contour is one closed subpath,
    /// so a donut's inner ellipse is a hole, not a slit (preset paths).
    FillPath {
        contours: Vec<Vec<(f32, f32)>>,
        color: [f32; 3],
        /// Even-odd (`f*`): Office fills `a:custGeom` paths alternately, so
        /// a traced signature's crossing strokes stay thin outlines.
        even_odd: bool,
    },
    /// Stroked subpaths; `true` closes one (`h`). Open ones keep the
    /// preset's open outline (brackets, braces) without a closing chord.
    StrokePath {
        subpaths: Vec<(Vec<(f32, f32)>, bool)>,
        width: f32,
        color: [f32; 3],
    },
    /// Cubic Bézier stroke (DrawingML curvedConnector). `segments` are
    /// (ctrl1, ctrl2, end) triples after `start`.
    Cubic {
        start: (f32, f32),
        segments: Vec<[(f32, f32); 3]>,
        width: f32,
        color: [f32; 3],
    },
    Jpeg {
        x: f32,
        y: f32,
        dw: f32,
        dh: f32,
        width: u32,
        height: u32,
        bytes: Vec<u8>,
        components: u8,
        crop: Option<[f32; 4]>,
        rotate_deg: f32,
        /// `prstGeom prst="ellipse"`: the picture shows through an oval.
        oval: bool,
    },
    Rgb {
        x: f32,
        y: f32,
        dw: f32,
        dh: f32,
        width: u32,
        height: u32,
        bytes: Vec<u8>,
        alpha: Option<Vec<u8>>,
        crop: Option<[f32; 4]>,
        rotate_deg: f32,
        oval: bool,
    },
    /// Behind-doc Word watermark (header SDT gallery=Watermarks).
    Watermark {
        face: FaceRef,
        size: f32,
        x: f32,
        y: f32,
        glyphs: Vec<u16>,
        color: [f32; 3],
        text: String,
        rotate_deg: f32,
    },
}

/// Sticky-note PDF annotation (not painted into the content stream).
pub(crate) struct PdfComment {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub contents: String,
    pub author: String,
}

/// One finished page, with the section `pgSz` it was laid out against.
pub(crate) struct Page {
    pub ops: Vec<Op>,
    pub width: f32,
    pub height: f32,
    pub comments: Vec<PdfComment>,
    /// Word All-Markup pasteboard: scale content, paint gray balloon column.
    pub markup_pane: bool,
    /// The section's right margin, which sets the pasteboard's scale.
    pub margin_r: f32,
    /// Laid out turned a quarter for vertical text (`tbRl`): the writer
    /// turns it back and stands CJK glyphs upright.
    pub vertical: bool,
}

impl Page {
    pub(crate) fn new(width: f32, height: f32) -> Self {
        Self {
            ops: Vec::new(),
            width,
            height,
            comments: Vec::new(),
            markup_pane: false,
            margin_r: 0.0,
            vertical: false,
        }
    }
}

/// Word Save-as-PDF All Markup: the page is scaled by `k` from x = 0.96
/// and a 0.949 gray pane 257.3pt wide (page units) overlaps its right
/// margin from 9.15pt past the text edge. `k` is a whole 1/300 fitting
/// page and pane into the paper width less 8pt (file_27 mr 54: 219/300;
/// docxide case63/64 mr 90: 229/300; fixtures_500 00b0c1ee A4: 228/300;
/// landscape mr 36: 230/300).
#[derive(Clone, Copy)]
struct MarkupChrome {
    gx: f32,
    gy: f32,
    gw: f32,
    gh: f32,
    k: f32,
    tx: f32,
    ty: f32,
}

const MARKUP_PANE_W: f32 = 257.3;
const MARKUP_PANE_GAP: f32 = 9.15;

fn markup_chrome(width: f32, height: f32, margin_r: f32) -> Option<MarkupChrome> {
    let span = width - margin_r + MARKUP_PANE_GAP + MARKUP_PANE_W;
    if span <= 0.0 {
        return None;
    }
    let k = ((width - 8.0) / span * 300.0).floor() / 300.0;
    let tx = 0.96;
    let gh = height * k;
    let ty = ((height - gh) / 2.0 / 0.24).round() * 0.24;
    Some(MarkupChrome {
        gx: tx + (width - margin_r + MARKUP_PANE_GAP) * k,
        gy: ty,
        gw: MARKUP_PANE_W * k,
        gh,
        k,
        tx,
        ty,
    })
}

impl Op {
    /// Move the op `dy` points up the page (negative: down).
    pub(crate) fn shift_y(&mut self, dy: f32) {
        match self {
            Op::Text { y, .. }
            | Op::FillRect { y, .. }
            | Op::StrokeRect { y, .. }
            | Op::Jpeg { y, .. }
            | Op::Rgb { y, .. }
            | Op::Watermark { y, .. } => *y += dy,
            Op::Line { y1, y2, .. } => {
                *y1 += dy;
                *y2 += dy;
            }
            Op::FillPoly { points, .. } | Op::StrokePoly { points, .. } => {
                points.iter_mut().for_each(|p| p.1 += dy);
            }
            Op::FillPath { contours, .. } => contours.iter_mut().flatten().for_each(|p| p.1 += dy),
            Op::StrokePath { subpaths, .. } => subpaths
                .iter_mut()
                .flat_map(|(pts, _)| pts.iter_mut())
                .for_each(|p| p.1 += dy),
            Op::Cubic {
                start, segments, ..
            } => {
                start.1 += dy;
                segments.iter_mut().flatten().for_each(|p| p.1 += dy);
            }
        }
    }

    pub(crate) fn text(
        face: impl Into<FaceRef>,
        size: f32,
        x: f32,
        y: f32,
        glyphs: Vec<u16>,
        color: [f32; 3],
        text: impl Into<String>,
    ) -> Self {
        Self::Text {
            face: face.into(),
            size,
            x,
            y,
            glyphs,
            color,
            text: text.into(),
            hscale: 1.0,
        }
    }

    /// The same text drawn `scale` times as wide (`w:w`).
    pub(crate) fn scaled(mut self, scale: f32) -> Self {
        if let Self::Text { hscale, .. } = &mut self {
            *hscale = scale;
        }
        self
    }
}

pub(crate) fn emit(fonts: &Fonts, pages: &[Page], options: PdfOptions) -> Vec<u8> {
    let used: Vec<FaceRef> = {
        let mut seen = Vec::new();
        for page in pages {
            for op in &page.ops {
                if let Op::Text { face, .. } | Op::Watermark { face, .. } = op
                    && !seen.contains(face)
                {
                    seen.push(*face);
                }
            }
        }
        if seen.is_empty() {
            seen.push(FaceRef::Catalogue(FaceId::CarlitoRegular));
        }
        seen
    };

    let mut objs: Vec<Vec<u8>> = vec![Vec::new(), Vec::new(), Vec::new()];
    // 1 catalog, 2 pages, 3 info
    let mut simple_need = Vec::new();
    let mut cid_need = Vec::new();
    // `winansi_bytes` scans the text and allocates, and every text op needs the
    // answer twice more below (resource name + string literal). Encode once
    // here, indexed by `[page][op]`, and read it back in the emit loop.
    let mut encodings: Vec<Vec<Option<Vec<u8>>>> = Vec::with_capacity(pages.len());
    for page in pages {
        let mut page_enc: Vec<Option<Vec<u8>>> = Vec::with_capacity(page.ops.len());
        for op in &page.ops {
            let mut enc = None;
            if let Op::Text {
                face, glyphs, text, ..
            }
            | Op::Watermark {
                face, glyphs, text, ..
            } = op
            {
                // A glyph shaped from several characters is painted by its
                // id; WinAnsi would paint each character instead.
                if text.chars().count() == glyphs.len() {
                    enc = winansi_bytes(text);
                }
                if enc.is_some() {
                    if !simple_need.contains(face) {
                        simple_need.push(*face);
                    }
                } else if !cid_need.contains(face) {
                    cid_need.push(*face);
                }
            }
            page_enc.push(enc);
        }
        encodings.push(page_enc);
    }
    if simple_need.is_empty() && cid_need.is_empty() {
        cid_need.push(FaceRef::Catalogue(FaceId::CarlitoRegular));
    }

    let mut simple_obj = FaceObjIds::new();
    let mut cid_obj = FaceObjIds::new();
    // Resource names stay readable (`/Calibri-Bold`), but every name is run
    // through `uniquify`: `sanitize_pdf_name` maps every non-alphanumeric byte
    // to `-`, so two override faces whose PostScript names differ only in
    // punctuation (`Foo_Bar` / `Foo.Bar`) would otherwise collapse to one key
    // and a reader would bind one of them to the wrong glyph mapping.
    let mut taken: Vec<String> = Vec::new();
    for face_id in &used {
        let face = fonts.get(*face_id);
        let want_simple = simple_need.contains(face_id);
        let want_cid = cid_need.contains(face_id);
        if !want_simple && !want_cid {
            continue;
        }
        let file_id = objs.len() + 1;
        let used_gids = face_used_glyphs(face, *face_id, pages);
        let program = subset_keep_gids(face.bytes(), &used_gids);
        // Font programs and image samples are binary: nothing greps them,
        // so they always deflate (000f5278 was a 25 MB PDF with them raw).
        // Content streams follow `options.compress`.
        objs.push(font_file_obj(
            program.as_deref().unwrap_or(face.bytes()),
            true,
        ));
        let desc_id = objs.len() + 1;
        objs.push(font_descriptor_obj(face, file_id));
        if want_simple {
            let id = objs.len() + 1;
            objs.push(simple_ttf_obj(face, desc_id));
            let name = uniquify(face.pdf_name(), &mut taken);
            simple_obj.insert(*face_id, id, name);
        }
        if want_cid {
            let cid_id = objs.len() + 1;
            objs.push(cid_font_obj(face, desc_id));
            let cmap_id = objs.len() + 1;
            objs.push(to_unicode_obj(
                &face_unicode_map(face, *face_id, pages),
                options.compress,
            ));
            let type0_id = objs.len() + 1;
            objs.push(type0_font_obj(face, cid_id, cmap_id));
            // `…CID` keeps the Type0 entry distinct from this face's simple
            // entry, exactly as before.
            let base = if want_simple {
                format!("{}CID", face.pdf_name())
            } else {
                face.pdf_name().to_string()
            };
            let name = uniquify(&base, &mut taken);
            cid_obj.insert(*face_id, type0_id, name);
        }
    }

    let mut page_ids = Vec::new();
    for (page_idx, page) in pages.iter().enumerate() {
        let page_enc = &encodings[page_idx];
        let mut xobjects = String::new();
        let mut img_n = 0usize;
        let mut has_watermark = false;
        for op in &page.ops {
            match op {
                Op::Jpeg {
                    width,
                    height,
                    bytes,
                    components,
                    ..
                } => {
                    img_n += 1;
                    let id = objs.len() + 1;
                    objs.push(jpeg_xobject(*width, *height, bytes, *components));
                    let _ = write!(xobjects, "/Im{img_n} {id} 0 R ");
                }
                Op::Rgb {
                    width,
                    height,
                    bytes,
                    alpha,
                    ..
                } => {
                    img_n += 1;
                    let smask = alpha.as_ref().map(|plane| {
                        let sid = objs.len() + 1;
                        objs.push(gray_xobject(*width, *height, plane, true));
                        sid
                    });
                    let id = objs.len() + 1;
                    objs.push(rgb_xobject(*width, *height, bytes, true, smask));
                    let _ = write!(xobjects, "/Im{img_n} {id} 0 R ");
                }
                Op::Watermark { .. } => has_watermark = true,
                _ => {}
            }
        }
        let mut stream = String::new();
        let markup = page
            .markup_pane
            .then(|| markup_chrome(page.width, page.height, page.margin_r))
            .flatten();
        if page.vertical {
            // Laid-out x runs down the page, laid-out y leftward from the
            // right edge: X = y, Y = width - x.
            let _ = writeln!(stream, "q 0 -1 1 0 0 {:.2} cm", page.width);
        }
        if let Some(m) = markup {
            let _ = writeln!(
                stream,
                "0.949 0.949 0.949 rg {x:.2} {y:.2} {w:.2} {h:.2} re f\n\
                 q {k:.4} 0 0 {k:.4} {tx:.2} {ty:.2} cm",
                x = m.gx,
                y = m.gy,
                w = m.gw,
                h = m.gh,
                k = m.k,
                tx = m.tx,
                ty = m.ty,
            );
        }
        // Only the faces this page actually paints, not every face in the
        // document.
        let res_for = |face: FaceRef, winansi: bool| -> Option<(usize, &str)> {
            if winansi {
                simple_obj.get(face).or_else(|| cid_obj.get(face))
            } else {
                cid_obj.get(face).or_else(|| simple_obj.get(face))
            }
        };
        let mut page_faces: Vec<(usize, &str)> = Vec::new();
        for (op_idx, op) in page.ops.iter().enumerate() {
            let (Op::Text { face, glyphs, .. } | Op::Watermark { face, glyphs, .. }) = op else {
                continue;
            };
            if glyphs.is_empty() {
                continue;
            }
            if let Some(entry) = res_for(*face, page_enc[op_idx].is_some())
                && !page_faces.iter().any(|(_, name)| *name == entry.1)
            {
                page_faces.push(entry);
            }
        }
        let mut font_res = String::new();
        for (obj_id, name) in &page_faces {
            let _ = write!(font_res, "/{name} {obj_id} 0 R ");
        }
        let mut img_counter = 0usize;
        for (op_idx, op) in page.ops.iter().enumerate() {
            match op {
                Op::Text {
                    face,
                    size,
                    x,
                    y,
                    glyphs,
                    color,
                    text,
                    hscale,
                } => {
                    if page.vertical
                        && text.chars().any(stands_upright)
                        && text.chars().count() == glyphs.len()
                        && let Some((_, name)) = res_for(*face, false)
                    {
                        let f = fonts.get(*face);
                        let (r, g, b) = (color[0], color[1], color[2]);
                        let mut gx = *x;
                        for (ch, gid) in text.chars().zip(glyphs.iter()) {
                            let adv = f.advance_pt(ch, *size) * *hscale;
                            if stands_upright(ch) {
                                // Stand the glyph up about its em box's
                                // centre; small marks sit in the cell's upper
                                // right in vertical setting.
                                let (cx, cy) = (gx + adv / 2.0, *y + 0.38 * size);
                                let lift = if matches!(ch, '、' | '。' | '，' | '．') {
                                    0.55 * size
                                } else {
                                    0.0
                                };
                                let _ = writeln!(
                                    stream,
                                    "q 1 0 0 1 {cx:.2} {cy:.2} cm 0 1 -1 0 0 0 cm BT /{name} {size:.2} Tf \
                                     {r:.3} {g:.3} {b:.3} rg {ox:.2} {oy:.2} Td <{gid:04X}> Tj ET Q",
                                    ox = lift - adv / 2.0,
                                    oy = lift - 0.38 * size,
                                );
                            } else {
                                let _ = writeln!(
                                    stream,
                                    "BT /{name} {size:.2} Tf {r:.3} {g:.3} {b:.3} rg {gx:.2} {y:.2} Td <{gid:04X}> Tj ET",
                                );
                            }
                            gx += adv;
                        }
                        continue;
                    }
                    if glyphs.is_empty() {
                        continue;
                    }
                    let encoded = page_enc[op_idx].as_deref();
                    let Some((_, name)) = res_for(*face, encoded.is_some()) else {
                        continue;
                    };
                    let lit = if let Some(bytes) = encoded {
                        pdf_literal(bytes)
                    } else {
                        let hex: String = glyphs.iter().map(|g| format!("{g:04X}")).collect();
                        format!("<{hex}>")
                    };
                    let (r, g, b) = (color[0], color[1], color[2]);
                    // A `w:w` scale squeezes the glyphs themselves about
                    // their origin; advances were scaled at layout.
                    let sx = *hscale;
                    if let Some((ppem, tc)) = word_device_paint(*size) {
                        // Word writes baselines in whole device units from
                        // the page top (0.24pt grid).
                        let down = page.height - *y;
                        let y = &(page.height - ((down / 0.24) + 0.5).floor() * 0.24);
                        let a = if (sx - 1.0).abs() > 0.001 {
                            format!("{:.4}", 0.24 * sx)
                        } else {
                            "0.24".into()
                        };
                        let _ = writeln!(
                            stream,
                            "q {a} 0 0 0.24 {x:.2} {y:.2} cm BT /{name} {ppem:.0} Tf {r:.3} {g:.3} {b:.3} rg {tc:.4} Tc 0 0 Td {lit} Tj ET Q",
                        );
                    } else if (sx - 1.0).abs() > 0.001 {
                        let _ = writeln!(
                            stream,
                            "q {sx:.4} 0 0 1 {x:.2} {y:.2} cm BT /{name} {size:.2} Tf {r:.3} {g:.3} {b:.3} rg 0 0 Td {lit} Tj ET Q",
                        );
                    } else {
                        let tc = word_device_track(*size);
                        let tc_op = if tc.abs() > 0.00005 {
                            format!("{tc:.5} Tc ")
                        } else {
                            String::new()
                        };
                        let _ = writeln!(
                            stream,
                            "BT /{name} {size:.2} Tf {r:.3} {g:.3} {b:.3} rg {tc_op}{x:.2} {y:.2} Td {lit} Tj ET",
                        );
                    }
                }
                Op::Watermark {
                    face,
                    size,
                    x,
                    y,
                    glyphs,
                    color,
                    text,
                    rotate_deg,
                } => {
                    if glyphs.is_empty() {
                        continue;
                    }
                    let rad = rotate_deg.to_radians();
                    let (sin, cos) = (rad.sin(), rad.cos());
                    let width = fonts.get(*face).width_pt(text, *size);
                    let dx = -width / 2.0;
                    let dy = -size * 0.35;
                    let encoded = page_enc[op_idx].as_deref();
                    let lit = if let Some(bytes) = encoded {
                        pdf_literal(bytes)
                    } else {
                        let hex: String = glyphs.iter().map(|g| format!("{g:04X}")).collect();
                        format!("<{hex}>")
                    };
                    let Some((_, name)) = res_for(*face, encoded.is_some()) else {
                        continue;
                    };
                    let _ = writeln!(
                        stream,
                        "q /WmGs gs 1 0 0 1 {x:.2} {y:.2} cm {cos:.4} {sin:.4} {nsin:.4} {cos:.4} 0 0 cm \
                         BT /{name} {size:.2} Tf {r:.3} {g:.3} {b:.3} rg {dx:.2} {dy:.2} Td {lit} Tj ET Q",
                        nsin = -sin,
                        r = color[0],
                        g = color[1],
                        b = color[2],
                    );
                }
                Op::Line {
                    x1,
                    y1,
                    x2,
                    y2,
                    width,
                    color,
                } => {
                    let _ = writeln!(
                        stream,
                        "{w:.2} w {r:.3} {g:.3} {b:.3} RG {x1:.2} {y1:.2} m {x2:.2} {y2:.2} l S",
                        w = width,
                        r = color[0],
                        g = color[1],
                        b = color[2],
                    );
                }
                Op::FillRect { x, y, w, h, color } => {
                    let _ = writeln!(
                        stream,
                        "{r:.3} {g:.3} {b:.3} rg {x:.2} {y:.2} {w:.2} {h:.2} re f",
                        r = color[0],
                        g = color[1],
                        b = color[2],
                    );
                }
                Op::StrokeRect {
                    x,
                    y,
                    w,
                    h,
                    width,
                    color,
                } => {
                    let _ = writeln!(
                        stream,
                        "{lw:.2} w {r:.3} {g:.3} {b:.3} RG {x:.2} {y:.2} {w:.2} {h:.2} re S",
                        lw = width,
                        r = color[0],
                        g = color[1],
                        b = color[2],
                    );
                }
                Op::FillPoly { points, color } => {
                    if let Some((x0, y0)) = points.first() {
                        let _ = write!(
                            stream,
                            "{r:.3} {g:.3} {b:.3} rg {x0:.2} {y0:.2} m",
                            r = color[0],
                            g = color[1],
                            b = color[2],
                        );
                        for (x, y) in points.iter().skip(1) {
                            let _ = write!(stream, " {x:.2} {y:.2} l");
                        }
                        stream.push_str(" h f\n");
                    }
                }
                Op::FillPath {
                    contours,
                    color,
                    even_odd,
                } => {
                    let mut body = String::new();
                    for c in contours.iter().filter(|c| c.len() >= 2) {
                        for (i, (x, y)) in c.iter().enumerate() {
                            let _ =
                                write!(body, " {x:.2} {y:.2} {}", if i == 0 { 'm' } else { 'l' });
                        }
                        body.push_str(" h");
                    }
                    if !body.is_empty() {
                        let _ = writeln!(
                            stream,
                            "{r:.3} {g:.3} {b:.3} rg{body} {op}",
                            r = color[0],
                            g = color[1],
                            b = color[2],
                            op = if *even_odd { "f*" } else { "f" },
                        );
                    }
                }
                Op::StrokePath {
                    subpaths,
                    width,
                    color,
                } => {
                    let mut body = String::new();
                    for (pts, closed) in subpaths.iter().filter(|(p, _)| p.len() >= 2) {
                        for (i, (x, y)) in pts.iter().enumerate() {
                            let _ =
                                write!(body, " {x:.2} {y:.2} {}", if i == 0 { 'm' } else { 'l' });
                        }
                        if *closed {
                            body.push_str(" h");
                        }
                    }
                    if !body.is_empty() {
                        let _ = writeln!(
                            stream,
                            "{w:.2} w {r:.3} {g:.3} {b:.3} RG{body} S",
                            w = width,
                            r = color[0],
                            g = color[1],
                            b = color[2],
                        );
                    }
                }
                Op::StrokePoly {
                    points,
                    width,
                    color,
                } => {
                    if let Some((x0, y0)) = points.first() {
                        let _ = write!(
                            stream,
                            "{w:.2} w {r:.3} {g:.3} {b:.3} RG {x0:.2} {y0:.2} m",
                            w = width,
                            r = color[0],
                            g = color[1],
                            b = color[2],
                        );
                        for (x, y) in points.iter().skip(1) {
                            let _ = write!(stream, " {x:.2} {y:.2} l");
                        }
                        stream.push_str(" h S\n");
                    }
                }
                Op::Cubic {
                    start,
                    segments,
                    width,
                    color,
                } => {
                    let _ = write!(
                        stream,
                        "{w:.2} w {r:.3} {g:.3} {b:.3} RG {x:.2} {y:.2} m",
                        w = width,
                        r = color[0],
                        g = color[1],
                        b = color[2],
                        x = start.0,
                        y = start.1,
                    );
                    for [(c1x, c1y), (c2x, c2y), (ex, ey)] in segments {
                        let _ = write!(
                            stream,
                            " {c1x:.2} {c1y:.2} {c2x:.2} {c2y:.2} {ex:.2} {ey:.2} c"
                        );
                    }
                    stream.push_str(" S\n");
                }
                Op::Jpeg {
                    x,
                    y,
                    dw,
                    dh,
                    crop,
                    rotate_deg,
                    oval,
                    ..
                }
                | Op::Rgb {
                    x,
                    y,
                    dw,
                    dh,
                    crop,
                    rotate_deg,
                    oval,
                    ..
                } => {
                    img_counter += 1;
                    let drawn = paint_image(*x, *y, *dw, *dh, *crop, img_counter, *rotate_deg);
                    if *oval {
                        let _ =
                            writeln!(stream, "q {} W n {drawn}Q", ellipse_path(*x, *y, *dw, *dh));
                    } else {
                        stream.push_str(&drawn);
                    }
                }
            }
        }
        if markup.is_some() {
            stream.push_str("Q\n");
        }
        if page.vertical {
            stream.push_str("Q\n");
        }
        let content_id = objs.len() + 1;
        objs.push(stream_object(&stream, options.compress));
        let mut annot_refs = String::new();
        for note in &page.comments {
            let id = objs.len() + 1;
            let scaled = markup.map(|m| PdfComment {
                x: m.k * note.x + m.tx,
                y: m.k * note.y + m.ty,
                w: note.w * m.k,
                h: note.h * m.k,
                contents: note.contents.clone(),
                author: note.author.clone(),
            });
            objs.push(text_annot_obj(scaled.as_ref().unwrap_or(note)));
            let _ = write!(annot_refs, "{id} 0 R ");
        }
        let annots = if annot_refs.is_empty() {
            String::new()
        } else {
            format!(" /Annots [{annot_refs}]")
        };
        let page_id = objs.len() + 1;
        page_ids.push(page_id);
        let ext_gstate = if has_watermark {
            " /ExtGState << /WmGs << /Type /ExtGState /ca 0.5 >> >>"
        } else {
            ""
        };
        objs.push(
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {w:.2} {h:.2}] \
                   /Contents {content_id} 0 R \
                   /Resources << /Font << {font_res} >> /XObject << {xobjects} >>{ext_gstate} >>{annots} >>",
                // A vertical page was laid out turned: its physical width
                // is the laid-out height.
                w = if page.vertical { page.height } else { page.width },
                h = if page.vertical { page.width } else { page.height },
            )
            .into_bytes(),
        );
    }

    objs[0] = b"<< /Type /Catalog /Pages 2 0 R >>".to_vec();
    let kids = page_ids
        .iter()
        .map(|id| format!("{id} 0 R"))
        .collect::<Vec<_>>()
        .join(" ");
    objs[1] = format!(
        "<< /Type /Pages /Kids [{kids}] /Count {n} >>",
        n = page_ids.len()
    )
    .into_bytes();
    objs[2] = b"<< /Producer (jubarte) /Creator (jubarte) >>".to_vec();
    finalize_pdf(&objs)
}

/// A PDF resource name that no other font entry in this document holds.
///
/// `sanitize_pdf_name` is lossy (every non-alphanumeric byte becomes `-`), so
/// distinct faces can want the same name. The first claimant keeps the plain
/// name — which is what makes a content stream readable, and what the
/// conversion tests assert on — and later collisions get `-2`, `-3`, … so the
/// page resource dictionary can never hold a duplicate key.
fn uniquify(base: &str, taken: &mut Vec<String>) -> String {
    let mut name = base.to_string();
    let mut n = 1u32;
    while taken.contains(&name) {
        n += 1;
        name = format!("{base}-{n}");
    }
    taken.push(name.clone());
    name
}

/// Face → (font object id, PDF resource name). Vec-backed: a
/// document uses a handful of faces, so a linear scan beats hashing.
struct FaceObjIds {
    items: Vec<(FaceRef, usize, String)>,
}

impl FaceObjIds {
    fn new() -> Self {
        Self { items: Vec::new() }
    }

    fn insert(&mut self, k: FaceRef, obj_id: usize, res_name: String) {
        self.items.push((k, obj_id, res_name));
    }

    /// `(object id, resource name)` for `k`, if this map holds it.
    fn get(&self, k: FaceRef) -> Option<(usize, &str)> {
        self.items
            .iter()
            .find(|(id, _, _)| *id == k)
            .map(|(_, obj_id, name)| (*obj_id, name.as_str()))
    }
}

/// Glyph ids a face paints: the shaped ids (Identity-H) and, for the
/// WinAnsi path, the ids the reader finds through the face's own cmap.
fn face_used_glyphs(face: &super::font::Face, id: FaceRef, pages: &[Page]) -> BTreeSet<u16> {
    let parsed = ttf_parser::Face::parse(face.bytes(), 0).ok();
    let mut used = BTreeSet::from([0u16]);
    for page in pages {
        for op in &page.ops {
            if let Op::Text {
                face, glyphs, text, ..
            }
            | Op::Watermark {
                face, glyphs, text, ..
            } = op
                && *face == id
            {
                used.extend(glyphs.iter().copied());
                if let Some(parsed) = &parsed {
                    used.extend(
                        text.chars()
                            .filter_map(|c| parsed.glyph_index(c))
                            .map(|g| g.0),
                    );
                }
            }
        }
    }
    used
}

/// Tables a PDF reader needs from an embedded TrueType program.
const SUBSET_TABLES: &[&[u8; 4]] = &[
    b"OS/2", b"cmap", b"cvt ", b"fpgm", b"glyf", b"head", b"hhea", b"hmtx", b"loca", b"maxp",
    b"name", b"post", b"prep",
];

/// A TrueType program that keeps every glyph id but only the outlines of
/// `used` (plus their composite components): unused glyphs become empty.
/// Ids never move, so `/W` and the Identity CIDToGIDMap stay valid, and
/// bitmap/layout tables a reader ignores are dropped. Faces were embedded
/// whole: a one-line document was 1.3 MB, a CJK one 10 MB. `None` (embed
/// whole) for a face without `glyf` (CFF) or one that does not parse.
fn subset_keep_gids(ttf: &[u8], used: &BTreeSet<u16>) -> Option<Vec<u8>> {
    let u16_at = |b: &[u8], at: usize| -> Option<u16> {
        b.get(at..at + 2).map(|x| u16::from_be_bytes([x[0], x[1]]))
    };
    let u32_at = |b: &[u8], at: usize| -> Option<u32> {
        b.get(at..at + 4)
            .map(|x| u32::from_be_bytes([x[0], x[1], x[2], x[3]]))
    };
    let num_tables = usize::from(u16_at(ttf, 4)?);
    let mut tables: Vec<([u8; 4], &[u8])> = Vec::with_capacity(num_tables);
    for t in 0..num_tables {
        let rec = 12 + 16 * t;
        let tag: [u8; 4] = ttf.get(rec..rec + 4)?.try_into().ok()?;
        let offset = usize::try_from(u32_at(ttf, rec + 8)?).ok()?;
        let length = usize::try_from(u32_at(ttf, rec + 12)?).ok()?;
        tables.push((tag, ttf.get(offset..offset + length)?));
    }
    let table = |tag: &[u8; 4]| tables.iter().find(|(t, _)| t == tag).map(|(_, d)| *d);
    let head = table(b"head")?;
    let glyf = table(b"glyf")?;
    let loca = table(b"loca")?;
    let num_glyphs = usize::from(u16_at(table(b"maxp")?, 4)?);
    let long = u16_at(head, 50)? != 0;
    let offset_of = |g: usize| -> Option<usize> {
        if long {
            usize::try_from(u32_at(loca, 4 * g)?).ok()
        } else {
            Some(usize::from(u16_at(loca, 2 * g)?) * 2)
        }
    };
    let glyph = |g: usize| -> Option<&[u8]> { glyf.get(offset_of(g)?..offset_of(g + 1)?) };
    // Close over composite components.
    let mut keep: BTreeSet<usize> = BTreeSet::new();
    let mut stack: Vec<usize> = used.iter().map(|g| usize::from(*g)).collect();
    while let Some(g) = stack.pop() {
        if g >= num_glyphs || !keep.insert(g) {
            continue;
        }
        let data = glyph(g)?;
        if data.len() < 10 || i16::from_be_bytes([data[0], data[1]]) >= 0 {
            continue;
        }
        let mut at = 10;
        loop {
            let flags = u16_at(data, at)?;
            stack.push(usize::from(u16_at(data, at + 2)?));
            at += 4 + if flags & 0x0001 != 0 { 4 } else { 2 };
            at += if flags & 0x0008 != 0 {
                2
            } else if flags & 0x0040 != 0 {
                4
            } else if flags & 0x0080 != 0 {
                8
            } else {
                0
            };
            if flags & 0x0020 == 0 {
                break;
            }
        }
    }
    let mut new_glyf: Vec<u8> = Vec::new();
    let mut new_loca: Vec<u8> = Vec::with_capacity(4 * (num_glyphs + 1));
    for g in 0..num_glyphs {
        new_loca.extend_from_slice(&u32::try_from(new_glyf.len()).ok()?.to_be_bytes());
        if keep.contains(&g) {
            new_glyf.extend_from_slice(glyph(g)?);
            while !new_glyf.len().is_multiple_of(4) {
                new_glyf.push(0);
            }
        }
    }
    new_loca.extend_from_slice(&u32::try_from(new_glyf.len()).ok()?.to_be_bytes());
    let mut new_head = head.to_vec();
    new_head.get_mut(8..12)?.copy_from_slice(&[0; 4]);
    new_head
        .get_mut(50..52)?
        .copy_from_slice(&1u16.to_be_bytes());
    let mut out_tables: Vec<([u8; 4], Cow<'_, [u8]>)> = Vec::new();
    for (tag, data) in &tables {
        if !SUBSET_TABLES.contains(&tag) {
            continue;
        }
        let data: Cow<'_, [u8]> = match tag {
            b"glyf" => Cow::Owned(std::mem::take(&mut new_glyf)),
            b"loca" => Cow::Owned(std::mem::take(&mut new_loca)),
            b"head" => Cow::Owned(std::mem::take(&mut new_head)),
            _ => Cow::Borrowed(*data),
        };
        out_tables.push((*tag, data));
    }
    out_tables.sort_by_key(|a| a.0);
    Some(write_sfnt(u32_at(ttf, 0)?, &out_tables))
}

/// An sfnt from `tables` (sorted by tag), with checksums and 4-byte padding.
fn write_sfnt(version: u32, tables: &[([u8; 4], Cow<'_, [u8]>)]) -> Vec<u8> {
    let checksum = |data: &[u8]| -> u32 {
        data.chunks(4).fold(0u32, |sum, c| {
            let mut w = [0u8; 4];
            w[..c.len()].copy_from_slice(c);
            sum.wrapping_add(u32::from_be_bytes(w))
        })
    };
    let n = u16::try_from(tables.len()).unwrap_or(u16::MAX);
    let mut pow = 1u16;
    let mut log = 0u16;
    while pow * 2 <= n {
        pow *= 2;
        log += 1;
    }
    let mut out = Vec::new();
    out.extend_from_slice(&version.to_be_bytes());
    out.extend_from_slice(&n.to_be_bytes());
    out.extend_from_slice(&(pow * 16).to_be_bytes());
    out.extend_from_slice(&log.to_be_bytes());
    out.extend_from_slice(&(n * 16 - pow * 16).to_be_bytes());
    let mut offset = 12 + 16 * tables.len();
    let mut body: Vec<u8> = Vec::new();
    for (tag, data) in tables {
        out.extend_from_slice(tag);
        out.extend_from_slice(&checksum(data).to_be_bytes());
        out.extend_from_slice(&u32::try_from(offset).unwrap_or(0).to_be_bytes());
        out.extend_from_slice(&u32::try_from(data.len()).unwrap_or(0).to_be_bytes());
        body.extend_from_slice(data);
        while !body.len().is_multiple_of(4) {
            body.push(0);
        }
        offset = 12 + 16 * tables.len() + body.len();
    }
    out.extend_from_slice(&body);
    out
}

fn font_file_obj(ttf: &[u8], compress: bool) -> Vec<u8> {
    // `/Length1` stays the *uncompressed* face length (PDF 32000-1 9.9), so a
    // reader knows how many bytes to expect after inflating.
    let raw_len = ttf.len();
    let (bytes, filter) = deflate(ttf, compress);
    let mut out = format!(
        "<< /Length {} /Length1 {raw_len}{filter} >>\nstream\n",
        bytes.len()
    )
    .into_bytes();
    out.extend_from_slice(&bytes);
    out.extend_from_slice(b"\nendstream");
    out
}

fn font_descriptor_obj(face: &super::font::Face, file_id: usize) -> Vec<u8> {
    let name = face.pdf_name();
    let [a, b, c, d] = face.pdf_bbox_1000();
    let ascent = face.pdf_ascent_1000();
    let descent = face.pdf_descent_1000();
    format!(
        "<< /Type /FontDescriptor /FontName /{name} /Flags 32 \
           /FontBBox [{a} {b} {c} {d}] /ItalicAngle 0 \
           /Ascent {ascent} /Descent {descent} /CapHeight {ascent} /StemV 80 \
           /FontFile2 {file_id} 0 R >>"
    )
    .into_bytes()
}

fn simple_ttf_obj(face: &super::font::Face, desc_id: usize) -> Vec<u8> {
    let name = face.pdf_name();
    let widths: Vec<String> = (32u8..=255)
        .map(|b| face.width_1000(winansi_char(b)).to_string())
        .collect();
    format!(
        "<< /Type /Font /Subtype /TrueType /BaseFont /{name} \
           /FirstChar 32 /LastChar 255 /Widths [{}] \
           /Encoding /WinAnsiEncoding /FontDescriptor {desc_id} 0 R >>",
        widths.join(" ")
    )
    .into_bytes()
}

/// Faces are embedded whole, and `/W` carries a width for every glyph in the
/// face rather than only the ids the page ops reference.
///
/// The tradeoff is deliberate for now: subsetting means rebuilding `loca` /
/// `glyf` / `cmap` and remapping every emitted glyph id, and a wrong subset is
/// a silently missing glyph in an oracle diff. It costs size — on a 217-page
/// redline the five embedded faces are 5.5 MB of a 48.8 MB file, and `/W`
/// lists thousands of unused widths. `PdfOptions::compress` recovers most of
/// that (5.5 MB → 3.0 MB) without touching glyph data; narrowing `/W` to the
/// referenced ids is the cheaper next step if it is not enough.
fn cid_font_obj(face: &super::font::Face, desc_id: usize) -> Vec<u8> {
    let name = face.pdf_name();
    let widths = face.pdf_widths_1000();
    let w_list = widths
        .iter()
        .map(i32::to_string)
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        "<< /Type /Font /Subtype /CIDFontType2 /BaseFont /{name} \
           /CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> \
           /FontDescriptor {desc_id} 0 R /DW 500 /W [0 [{w_list}]] /CIDToGIDMap /Identity >>"
    )
    .into_bytes()
}

fn type0_font_obj(face: &super::font::Face, cid_id: usize, cmap_id: usize) -> Vec<u8> {
    let name = face.pdf_name();
    format!(
        "<< /Type /Font /Subtype /Type0 /BaseFont /{name} /Encoding /Identity-H \
           /DescendantFonts [{cid_id} 0 R] /ToUnicode {cmap_id} 0 R >>"
    )
    .into_bytes()
}

/// The text behind each glyph id a face paints, for `/ToUnicode`.
/// The face's own cmap answers first (exact, order-free); a glyph it cannot
/// reach (a shaped form) takes the character at its index when the run has
/// one glyph per character, or its whole text when it is painted alone (a
/// glyph shaped from several characters).
fn face_unicode_map(
    face: &super::font::Face,
    id: FaceRef,
    pages: &[Page],
) -> BTreeMap<u16, String> {
    let parsed = ttf_parser::Face::parse(face.bytes(), 0).ok();
    let mut map = BTreeMap::new();
    let mut zipped = BTreeMap::new();
    let mut painted = BTreeSet::new();
    for page in pages {
        for op in &page.ops {
            if let Op::Text {
                face, glyphs, text, ..
            }
            | Op::Watermark {
                face, glyphs, text, ..
            } = op
                && *face == id
            {
                painted.extend(glyphs.iter().copied());
                for c in text.chars() {
                    if let Some(g) = parsed.as_ref().and_then(|p| p.glyph_index(c)) {
                        map.entry(g.0).or_insert_with(|| c.to_string());
                    }
                }
                if glyphs.len() == text.chars().count() {
                    for (&g, c) in glyphs.iter().zip(text.chars()) {
                        zipped.entry(g).or_insert_with(|| c.to_string());
                    }
                } else if let [g] = glyphs[..]
                    && !text.is_empty()
                {
                    zipped.entry(g).or_insert_with(|| text.clone());
                }
            }
        }
    }
    for (g, c) in zipped {
        map.entry(g).or_insert(c);
    }
    map.retain(|g, _| *g != 0 && painted.contains(g));
    map
}

/// A `/ToUnicode` CMap stream (PDF 32000-1 9.10.3) mapping 2-byte CIDs
/// (= glyph ids under Identity-H) to UTF-16BE.
fn to_unicode_obj(map: &BTreeMap<u16, String>, compress: bool) -> Vec<u8> {
    let mut cmap = String::from(
        "/CIDInit /ProcSet findresource begin\n12 dict begin\nbegincmap\n\
         /CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def\n\
         /CMapName /Adobe-Identity-UCS def\n/CMapType 2 def\n\
         1 begincodespacerange\n<0000> <FFFF>\nendcodespacerange\n",
    );
    let entries: Vec<(&u16, &String)> = map.iter().collect();
    for chunk in entries.chunks(100) {
        let _ = writeln!(cmap, "{} beginbfchar", chunk.len());
        for (g, text) in chunk {
            let _ = write!(cmap, "<{g:04X}> <");
            for u in text.encode_utf16() {
                let _ = write!(cmap, "{u:04X}");
            }
            cmap.push_str(">\n");
        }
        cmap.push_str("endbfchar\n");
    }
    cmap.push_str("endcmap\nCMapName currentdict /CMap defineresource pop\nend\nend");
    let (bytes, filter) = deflate(cmap.as_bytes(), compress);
    let mut out = format!("<< /Length {}{filter} >>\nstream\n", bytes.len()).into_bytes();
    out.extend_from_slice(&bytes);
    out.extend_from_slice(b"\nendstream");
    out
}

fn jpeg_xobject(width: u32, height: u32, bytes: &[u8], components: u8) -> Vec<u8> {
    let (colorspace, decode) = match components {
        1 => ("/DeviceGray", ""),
        3 => ("/DeviceRGB", ""),
        4 => ("/DeviceCMYK", " /Decode [1 0 1 0 1 0 1 0]"),
        _ => ("/DeviceRGB", ""),
    };
    let mut out = format!(
        "<< /Type /XObject /Subtype /Image /Width {width} /Height {height} \
           /ColorSpace {colorspace} /BitsPerComponent 8 /Filter /DCTDecode{decode} \
           /Length {} >>\nstream\n",
        bytes.len()
    )
    .into_bytes();
    out.extend_from_slice(bytes);
    out.extend_from_slice(b"\nendstream");
    out
}

fn rgb_xobject(
    width: u32,
    height: u32,
    bytes: &[u8],
    compress: bool,
    smask: Option<usize>,
) -> Vec<u8> {
    let (bytes, filter) = deflate(bytes, compress);
    let smask_e = smask
        .map(|id| format!(" /SMask {id} 0 R"))
        .unwrap_or_default();
    let mut out = format!(
        "<< /Type /XObject /Subtype /Image /Width {width} /Height {height} \
           /ColorSpace /DeviceRGB /BitsPerComponent 8{filter}{smask_e} \
           /Length {} >>\nstream\n",
        bytes.len()
    )
    .into_bytes();
    out.extend_from_slice(&bytes);
    out.extend_from_slice(b"\nendstream");
    out
}

fn gray_xobject(width: u32, height: u32, bytes: &[u8], compress: bool) -> Vec<u8> {
    let (bytes, filter) = deflate(bytes, compress);
    let mut out = format!(
        "<< /Type /XObject /Subtype /Image /Width {width} /Height {height} \
           /ColorSpace /DeviceGray /BitsPerComponent 8{filter} \
           /Length {} >>\nstream\n",
        bytes.len()
    )
    .into_bytes();
    out.extend_from_slice(&bytes);
    out.extend_from_slice(b"\nendstream");
    out
}

/// The ellipse inscribed in the box as four cubic arcs (`m … c … h`).
fn ellipse_path(x: f32, y: f32, w: f32, h: f32) -> String {
    const K: f32 = 0.552_284_8;
    let (rx, ry) = (w * 0.5, h * 0.5);
    let (cx, cy) = (x + rx, y + ry);
    let (kx, ky) = (rx * K, ry * K);
    format!(
        "{:.2} {cy:.2} m {:.2} {:.2} {:.2} {:.2} {cx:.2} {:.2} c \
         {:.2} {:.2} {:.2} {:.2} {:.2} {cy:.2} c \
         {:.2} {:.2} {:.2} {:.2} {cx:.2} {:.2} c \
         {:.2} {:.2} {:.2} {:.2} {:.2} {cy:.2} c h",
        cx + rx,
        cx + rx,
        cy + ky,
        cx + kx,
        cy + ry,
        cy + ry,
        cx - kx,
        cy + ry,
        cx - rx,
        cy + ky,
        cx - rx,
        cx - rx,
        cy - ky,
        cx - kx,
        cy - ry,
        cy - ry,
        cx + kx,
        cy - ry,
        cx + rx,
        cy - ky,
        cx + rx,
    )
}

/// `a:srcRect` l/t/r/b as 0..1. Scale the full image so the uncropped
/// window fills `dw×dh`, then clip to the extent. `a:xfrm/@rot` is applied
/// about the extent centre (Word).
fn paint_image(
    x: f32,
    y: f32,
    dw: f32,
    dh: f32,
    crop: Option<[f32; 4]>,
    n: usize,
    rotate_deg: f32,
) -> String {
    let inner = match crop {
        Some([l, t, r, b]) if l.abs() + r.abs() + t.abs() + b.abs() > 0.001 => {
            let fw = (1.0 - l - r).max(0.001);
            let fh = (1.0 - t - b).max(0.001);
            let sx = dw / fw;
            let sy = dh / fh;
            let x0 = x - sx * l;
            let y0 = y - sy * b;
            format!(
                "q {x:.2} {y:.2} {dw:.2} {dh:.2} re W n {sx:.2} 0 0 {sy:.2} {x0:.2} {y0:.2} cm /Im{n} Do Q\n"
            )
        }
        _ => format!("q {dw:.2} 0 0 {dh:.2} {x:.2} {y:.2} cm /Im{n} Do Q\n"),
    };
    if rotate_deg.abs() < 0.05 {
        return inner;
    }
    let cx = x + dw * 0.5;
    let cy = y + dh * 0.5;
    let rad = rotate_deg.to_radians();
    let mut cos = rad.cos();
    let mut sin = rad.sin();
    if cos.abs() < 1e-4 {
        cos = 0.0;
    }
    if sin.abs() < 1e-4 {
        sin = 0.0;
    }
    format!(
        "q 1 0 0 1 {cx:.2} {cy:.2} cm {cos:.4} {sin:.4} {nsin:.4} {cos:.4} 0 0 cm \
         1 0 0 1 {ncx:.2} {ncy:.2} cm {inner}Q\n",
        nsin = -sin,
        ncx = -cx,
        ncy = -cy,
    )
}

fn text_annot_obj(note: &PdfComment) -> Vec<u8> {
    let x2 = note.x + note.w.max(12.0);
    let y2 = note.y + note.h.max(12.0);
    let contents = pdf_text_string(&note.contents);
    let author = pdf_text_string(&note.author);
    // /F 0: not Printed, so raster oracles (comment-stripped Word PDFs)
    // do not pick up balloon chrome.
    format!(
        "<< /Type /Annot /Subtype /Text /Rect [{x:.2} {y:.2} {x2:.2} {y2:.2}] \
           /Contents {contents} /T {author} /Name /Comment /F 0 /C [1 0.92 0.4] >>",
        x = note.x,
        y = note.y,
    )
    .into_bytes()
}

/// Stream payload plus the `/Filter` entry that describes it.
///
/// `/FlateDecode` is zlib-wrapped deflate (PDF 32000-1 7.4.4), which is what
/// `ZlibEncoder` writes. A deflate failure is not worth failing a conversion
/// over: fall back to the raw bytes and no filter.
fn deflate(raw: &[u8], compress: bool) -> (Cow<'_, [u8]>, &'static str) {
    if !compress {
        return (Cow::Borrowed(raw), "");
    }
    let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
    match enc.write_all(raw).and_then(|()| enc.finish()) {
        Ok(packed) => (Cow::Owned(packed), " /Filter /FlateDecode"),
        Err(_) => (Cow::Borrowed(raw), ""),
    }
}

fn stream_object(ops: &str, compress: bool) -> Vec<u8> {
    let (bytes, filter) = deflate(ops.as_bytes(), compress);
    let mut out = format!("<< /Length {}{filter} >>\nstream\n", bytes.len()).into_bytes();
    out.extend_from_slice(&bytes);
    out.extend_from_slice(b"\nendstream");
    out
}

fn winansi_byte(ch: char) -> Option<u8> {
    match ch as u32 {
        0x20..=0x7E | 0xA0..=0xFF => Some(ch as u8),
        0x0152 => Some(0x8C),
        0x0153 => Some(0x9C),
        0x0160 => Some(0x8A),
        0x0161 => Some(0x9A),
        0x0178 => Some(0x9F),
        0x017D => Some(0x8E),
        0x017E => Some(0x9E),
        0x0192 => Some(0x83),
        0x02C6 => Some(0x88),
        0x02DC => Some(0x98),
        0x2013 => Some(0x96),
        0x2014 => Some(0x97),
        0x2018 => Some(0x91),
        0x2019 => Some(0x92),
        0x201A => Some(0x82),
        0x201C => Some(0x93),
        0x201D => Some(0x94),
        0x201E => Some(0x84),
        0x2020 => Some(0x86),
        0x2021 => Some(0x87),
        0x2022 => Some(0x95),
        0x2026 => Some(0x85),
        0x2030 => Some(0x89),
        0x2039 => Some(0x8B),
        0x203A => Some(0x9B),
        0x20AC => Some(0x80),
        0x2122 => Some(0x99),
        _ => None,
    }
}

fn winansi_char(byte: u8) -> char {
    match byte {
        0x80 => '\u{20AC}',
        0x82 => '\u{201A}',
        0x83 => '\u{0192}',
        0x84 => '\u{201E}',
        0x85 => '\u{2026}',
        0x86 => '\u{2020}',
        0x87 => '\u{2021}',
        0x88 => '\u{02C6}',
        0x89 => '\u{2030}',
        0x8A => '\u{0160}',
        0x8B => '\u{2039}',
        0x8C => '\u{0152}',
        0x8E => '\u{017D}',
        0x91 => '\u{2018}',
        0x92 => '\u{2019}',
        0x93 => '\u{201C}',
        0x94 => '\u{201D}',
        0x95 => '\u{2022}',
        0x96 => '\u{2013}',
        0x97 => '\u{2014}',
        0x98 => '\u{02DC}',
        0x99 => '\u{2122}',
        0x9A => '\u{0161}',
        0x9B => '\u{203A}',
        0x9C => '\u{0153}',
        0x9E => '\u{017E}',
        0x9F => '\u{0178}',
        0x20..=0x7E | 0xA0..=0xFF => char::from(byte),
        _ => ' ',
    }
}

fn winansi_bytes(text: &str) -> Option<Vec<u8>> {
    if text.is_empty() {
        return None;
    }
    text.chars().map(winansi_byte).collect()
}

/// A PDF *text string* (PDF 32000-1 7.9.2.2), for annotation fields.
///
/// A literal string is read as PDFDocEncoding, so raw UTF-8 shows up as
/// mojibake — `José` becomes `JosÃ©`, and anything outside Latin-1 is worse.
/// Non-ASCII text therefore goes out as a UTF-16BE hex string with a `FEFF`
/// BOM. ASCII stays a literal: it is identical under both encodings and keeps
/// the annotation readable in the raw file.
fn pdf_text_string(text: &str) -> String {
    if text.is_ascii() {
        return pdf_literal(text.as_bytes());
    }
    let mut out = String::from("<FEFF");
    for unit in text.encode_utf16() {
        let _ = write!(out, "{unit:04X}");
    }
    out.push('>');
    out
}

fn pdf_literal(bytes: &[u8]) -> String {
    let mut out = String::from("(");
    for &b in bytes {
        match b {
            b'(' | b')' | b'\\' => {
                out.push('\\');
                out.push(char::from(b));
            }
            32..=126 => out.push(char::from(b)),
            _ => {
                let _ = write!(out, "\\{b:03o}");
            }
        }
    }
    out.push(')');
    out
}

fn finalize_pdf(objects: &[Vec<u8>]) -> Vec<u8> {
    let mut out = Vec::from(b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n");
    let mut offsets = Vec::with_capacity(objects.len());
    for (idx, obj) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n", idx + 1).as_bytes());
        out.extend_from_slice(obj);
        if !obj.ends_with(b"\n") {
            out.push(b'\n');
        }
        out.extend_from_slice(b"endobj\n");
    }
    let xref_at = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
    out.extend_from_slice(b"0000000000 65535 f \n");
    for offset in offsets {
        out.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {size} /Root 1 0 R /Info 3 0 R >>\nstartxref\n{xref_at}\n%%EOF\n",
            size = objects.len() + 1,
        )
        .as_bytes(),
    );
    out
}

/// A character that stands upright in vertical text: ideographs, kana and
/// full-width forms. Brackets, dashes and the long-vowel mark turn with the
/// line, as do Latin letters and digits.
fn stands_upright(c: char) -> bool {
    let cjk = matches!(
        c,
        '\u{3000}'..='\u{9FFF}' | '\u{F900}'..='\u{FAFF}' | '\u{FF00}'..='\u{FFEF}' | '\u{20000}'..='\u{2FA1F}'
    );
    cjk && !matches!(
        c,
        '〈' | '〉'
            | '《'
            | '》'
            | '「'
            | '」'
            | '『'
            | '』'
            | '【'
            | '】'
            | '〔'
            | '〕'
            | '〖'
            | '〗'
            | '〘'
            | '〙'
            | '〚'
            | '〛'
            | '（'
            | '）'
            | '［'
            | '］'
            | '｛'
            | '｝'
            | 'ー'
            | '〜'
            | '～'
            | '－'
            | '＝'
            | '\u{3000}'
    )
}

#[cfg(test)]
mod tests {
    use super::uniquify;

    /// Faces were embedded whole (a one-line PDF was 1.3 MB). The subset
    /// keeps every glyph id, the outlines of the used ones and the
    /// components of a used composite, and empties the rest.
    #[test]
    fn subset_keeps_ids_and_used_outlines_only() {
        let bytes = super::FaceId::CarlitoRegular.bytes();
        let full = ttf_parser::Face::parse(bytes, 0).expect("Carlito");
        let a = full.glyph_index('A').expect("A").0;
        let b = full.glyph_index('B').expect("B").0;
        // "É" is a composite of E and an accent in Carlito: E's outline
        // must survive although E itself is unused.
        let e_acute = full.glyph_index('É').expect("Eacute").0;
        let e = full.glyph_index('E').expect("E").0;
        let used = std::collections::BTreeSet::from([0u16, a, e_acute]);
        let program = super::subset_keep_gids(bytes, &used).expect("glyf face subsets");
        assert!(
            program.len() < bytes.len() / 3,
            "{} vs {}",
            program.len(),
            bytes.len()
        );
        let sub = ttf_parser::Face::parse(&program, 0).expect("subset parses");
        assert_eq!(
            sub.number_of_glyphs(),
            full.number_of_glyphs(),
            "ids never move"
        );
        assert_eq!(sub.glyph_index('A').map(|g| g.0), Some(a), "cmap kept");
        let bbox = |f: &ttf_parser::Face<'_>, g: u16| f.glyph_bounding_box(ttf_parser::GlyphId(g));
        assert_eq!(bbox(&sub, a), bbox(&full, a), "used outline kept");
        assert!(bbox(&sub, b).is_none(), "unused outline emptied");
        assert_eq!(bbox(&sub, e_acute), bbox(&full, e_acute));
        assert_eq!(
            bbox(&sub, e),
            bbox(&full, e),
            "a composite keeps its components"
        );
    }

    /// CodeRabbit PR#4: two override faces whose PostScript names differ only
    /// in punctuation both sanitize to `Foo-Bar`, so the page resource
    /// dictionary held a duplicate key and a reader bound one of the faces to
    /// the wrong glyph mapping.
    #[test]
    fn colliding_font_names_get_distinct_resource_names() {
        let mut taken = Vec::new();
        let first = uniquify("Foo-Bar", &mut taken);
        let second = uniquify("Foo-Bar", &mut taken);
        let third = uniquify("Foo-Bar", &mut taken);
        assert_eq!(first, "Foo-Bar", "first claimant keeps the readable name");
        assert_ne!(first, second);
        assert_ne!(second, third);
        assert_ne!(first, third);
        assert_eq!(second, "Foo-Bar-2");
        assert_eq!(third, "Foo-Bar-3");
    }

    /// CodeRabbit PR#4: annotation text went out as raw UTF-8 in a literal
    /// string, which readers decode as PDFDocEncoding.
    #[test]
    fn non_ascii_annotation_text_is_utf16be_with_a_bom() {
        // "José" — U+004A U+006F U+0073 U+00E9
        assert_eq!(super::pdf_text_string("José"), "<FEFF004A006F007300E9>");
        // Outside Latin-1 entirely.
        assert_eq!(super::pdf_text_string("東京"), "<FEFF67714EAC>");
        // Astral plane must survive as a surrogate pair.
        assert_eq!(super::pdf_text_string("\u{1F600}"), "<FEFFD83DDE00>");
    }

    /// ASCII keeps the readable literal form: identical under PDFDocEncoding
    /// and UTF-16BE, and the conversion suite parses `(...)` literals.
    #[test]
    fn ascii_annotation_text_stays_a_literal_string() {
        assert_eq!(super::pdf_text_string("Reviewer"), "(Reviewer)");
        assert_eq!(super::pdf_text_string("a (b) c"), "(a \\(b\\) c)");
    }

    /// A disambiguated name must not collide with a face that genuinely
    /// carries the disambiguated spelling.
    #[test]
    fn uniquify_skips_a_name_already_claimed_verbatim() {
        let mut taken = Vec::new();
        assert_eq!(uniquify("Sans", &mut taken), "Sans");
        assert_eq!(uniquify("Sans-2", &mut taken), "Sans-2");
        assert_eq!(uniquify("Sans", &mut taken), "Sans-3");
    }
}

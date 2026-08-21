// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! PDF 1.4 writer: embedded TTF (Identity-H), stroked rules, JPEG/RGB images.

use super::font::{FaceId, Fonts, word_device_paint, word_device_track};

/// One drawing command on a page (PDF user space, origin bottom-left).
pub(crate) enum Op {
    Text {
        face: FaceId,
        size: f32,
        x: f32,
        y: f32,
        glyphs: Vec<u16>,
        color: [f32; 3],
        /// Source characters. When every char is WinAnsi, the writer emits a
        /// simple TrueType font like Word Quartz (hinted by MuPDF). Empty or
        /// non-WinAnsi text stays on Identity-H CID.
        text: String,
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
    FillPoly {
        points: Vec<(f32, f32)>,
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
    },
    Rgb {
        x: f32,
        y: f32,
        dw: f32,
        dh: f32,
        width: u32,
        height: u32,
        bytes: Vec<u8>,
    },
    /// Behind-doc Word watermark (header SDT gallery=Watermarks).
    Watermark {
        face: FaceId,
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
}

impl Page {
    pub(crate) fn new(width: f32, height: f32) -> Self {
        Self {
            ops: Vec::new(),
            width,
            height,
            comments: Vec::new(),
            markup_pane: false,
        }
    }
}

/// Word Save-as-PDF All Markup (file_27): letter content is scaled into the
/// left ~415pt and a 0.949 gray balloon sits on the right. Landscape uses
/// the matching Word path (cm 0.184 vs portrait 0.1752).
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

fn markup_chrome(width: f32, height: f32) -> Option<MarkupChrome> {
    if (width - 612.0).abs() < 2.0 && (height - 792.0).abs() < 2.0 {
        Some(MarkupChrome {
            gx: 414.9576,
            gy: 107.52,
            gw: 187.8144,
            gh: 578.16,
            k: 0.73,
            tx: 0.96,
            ty: 107.52,
        })
    } else if (width - 792.0).abs() < 2.0 && (height - 612.0).abs() < 2.0 {
        Some(MarkupChrome {
            gx: 587.552,
            gy: 71.04,
            gw: 197.248,
            gh: 469.2,
            k: 0.184 / 0.24,
            tx: 0.96,
            ty: 71.04,
        })
    } else {
        None
    }
}

impl Op {
    pub(crate) fn text(
        face: FaceId,
        size: f32,
        x: f32,
        y: f32,
        glyphs: Vec<u16>,
        color: [f32; 3],
        text: impl Into<String>,
    ) -> Self {
        Self::Text {
            face,
            size,
            x,
            y,
            glyphs,
            color,
            text: text.into(),
        }
    }
}

pub(crate) fn emit(fonts: &Fonts, pages: &[Page]) -> Vec<u8> {
    let used: Vec<FaceId> = {
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
            seen.push(FaceId::CarlitoRegular);
        }
        seen
    };

    let mut objs: Vec<Vec<u8>> = vec![Vec::new(), Vec::new(), Vec::new()];
    // 1 catalog, 2 pages, 3 info
    let mut simple_need = Vec::new();
    let mut cid_need = Vec::new();
    for page in pages {
        for op in &page.ops {
            if let Op::Text { face, text, .. } | Op::Watermark { face, text, .. } = op {
                if winansi_bytes(text).is_some() {
                    if !simple_need.contains(face) {
                        simple_need.push(*face);
                    }
                } else if !cid_need.contains(face) {
                    cid_need.push(*face);
                }
            }
        }
    }
    if simple_need.is_empty() && cid_need.is_empty() {
        cid_need.push(FaceId::CarlitoRegular);
    }

    let mut simple_obj = HashMapLite::new();
    let mut cid_obj = HashMapLite::new();
    for face_id in &used {
        let face = fonts.get(*face_id);
        let want_simple = simple_need.contains(face_id);
        let want_cid = cid_need.contains(face_id);
        if !want_simple && !want_cid {
            continue;
        }
        let file_id = objs.len() + 1;
        objs.push(font_file_obj(face.bytes()));
        let desc_id = objs.len() + 1;
        objs.push(font_descriptor_obj(face, file_id));
        if want_simple {
            let id = objs.len() + 1;
            objs.push(simple_ttf_obj(face, desc_id));
            simple_obj.insert(*face_id, id);
        }
        if want_cid {
            let cid_id = objs.len() + 1;
            objs.push(cid_font_obj(face, desc_id));
            let type0_id = objs.len() + 1;
            objs.push(type0_font_obj(face, cid_id));
            cid_obj.insert(*face_id, type0_id);
        }
    }

    let mut page_ids = Vec::new();
    for page in pages {
        let mut xobjects = String::new();
        let mut extra_ops = String::new();
        let mut img_n = 0usize;
        for op in &page.ops {
            match op {
                Op::Jpeg {
                    width,
                    height,
                    bytes,
                    x,
                    y,
                    dw,
                    dh,
                    ..
                } => {
                    img_n += 1;
                    let id = objs.len() + 1;
                    objs.push(jpeg_xobject(*width, *height, bytes));
                    xobjects.push_str(&format!("/Im{img_n} {id} 0 R "));
                    extra_ops.push_str(&format!(
                        "q {dw:.2} 0 0 {dh:.2} {x:.2} {y:.2} cm /Im{img_n} Do Q\n"
                    ));
                }
                Op::Rgb {
                    width,
                    height,
                    bytes,
                    x,
                    y,
                    dw,
                    dh,
                    ..
                } => {
                    img_n += 1;
                    let id = objs.len() + 1;
                    objs.push(rgb_xobject(*width, *height, bytes));
                    xobjects.push_str(&format!("/Im{img_n} {id} 0 R "));
                    extra_ops.push_str(&format!(
                        "q {dw:.2} 0 0 {dh:.2} {x:.2} {y:.2} cm /Im{img_n} Do Q\n"
                    ));
                }
                _ => {}
            }
        }
        let mut stream = String::new();
        let markup = page
            .markup_pane
            .then(|| markup_chrome(page.width, page.height))
            .flatten();
        if let Some(m) = markup {
            stream.push_str(&format!(
                "0.949 0.949 0.949 rg {x:.2} {y:.2} {w:.2} {h:.2} re f\n\
                 q {k:.4} 0 0 {k:.4} {tx:.2} {ty:.2} cm\n",
                x = m.gx,
                y = m.gy,
                w = m.gw,
                h = m.gh,
                k = m.k,
                tx = m.tx,
                ty = m.ty,
            ));
        }
        let mut font_res = String::new();
        for (face_id, obj_id) in &simple_obj {
            font_res.push_str(&format!(
                "/{name} {obj_id} 0 R ",
                name = fonts.get(*face_id).pdf_name()
            ));
        }
        for (face_id, obj_id) in &cid_obj {
            let base = fonts.get(*face_id).pdf_name();
            let name = if simple_obj.contains(*face_id) {
                format!("{base}CID")
            } else {
                base.to_string()
            };
            font_res.push_str(&format!("/{name} {obj_id} 0 R "));
        }
        for op in &page.ops {
            match op {
                Op::Text {
                    face,
                    size,
                    x,
                    y,
                    glyphs,
                    color,
                    text,
                } => {
                    if glyphs.is_empty() {
                        continue;
                    }
                    let base = fonts.get(*face).pdf_name();
                    let name = if winansi_bytes(text).is_none() && simple_obj.contains(*face) {
                        format!("{base}CID")
                    } else {
                        base.to_string()
                    };
                    let lit = if let Some(bytes) = winansi_bytes(text) {
                        pdf_literal(&bytes)
                    } else {
                        let hex: String = glyphs.iter().map(|g| format!("{g:04X}")).collect();
                        format!("<{hex}>")
                    };
                    let (r, g, b) = (color[0], color[1], color[2]);
                    if let Some((ppem, tc)) = word_device_paint(*size) {
                        stream.push_str(&format!(
                            "q 0.24 0 0 0.24 {x:.2} {y:.2} cm BT /{name} {ppem:.0} Tf {r:.3} {g:.3} {b:.3} rg {tc:.4} Tc 0 0 Td {lit} Tj ET Q\n",
                        ));
                    } else {
                        let tc = word_device_track(*size);
                        let tc_op = if tc.abs() > 0.00005 {
                            format!("{tc:.5} Tc ")
                        } else {
                            String::new()
                        };
                        stream.push_str(&format!(
                            "BT /{name} {size:.2} Tf {r:.3} {g:.3} {b:.3} rg {tc_op}{x:.2} {y:.2} Td {lit} Tj ET\n",
                        ));
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
                    let base = fonts.get(*face).pdf_name();
                    let rad = rotate_deg.to_radians();
                    let (sin, cos) = (rad.sin(), rad.cos());
                    let width = fonts.get(*face).width_pt(text, *size);
                    let dx = -width / 2.0;
                    let dy = -size * 0.35;
                    let lit = if let Some(bytes) = winansi_bytes(text) {
                        pdf_literal(&bytes)
                    } else {
                        let hex: String = glyphs.iter().map(|g| format!("{g:04X}")).collect();
                        format!("<{hex}>")
                    };
                    let name = if winansi_bytes(text).is_none() && simple_obj.contains(*face) {
                        format!("{base}CID")
                    } else {
                        base.to_string()
                    };
                    stream.push_str(&format!(
                        "q 1 0 0 1 {x:.2} {y:.2} cm {cos:.4} {sin:.4} {nsin:.4} {cos:.4} 0 0 cm \
                         BT /{name} {size:.2} Tf {r:.3} {g:.3} {b:.3} rg {dx:.2} {dy:.2} Td {lit} Tj ET Q\n",
                        nsin = -sin,
                        r = color[0],
                        g = color[1],
                        b = color[2],
                    ));
                }
                Op::Line {
                    x1,
                    y1,
                    x2,
                    y2,
                    width,
                    color,
                } => {
                    stream.push_str(&format!(
                        "{w:.2} w {r:.3} {g:.3} {b:.3} RG {x1:.2} {y1:.2} m {x2:.2} {y2:.2} l S\n",
                        w = width,
                        r = color[0],
                        g = color[1],
                        b = color[2],
                    ));
                }
                Op::FillRect { x, y, w, h, color } => {
                    stream.push_str(&format!(
                        "{r:.3} {g:.3} {b:.3} rg {x:.2} {y:.2} {w:.2} {h:.2} re f\n",
                        r = color[0],
                        g = color[1],
                        b = color[2],
                    ));
                }
                Op::FillPoly { points, color } => {
                    if let Some((x0, y0)) = points.first() {
                        stream.push_str(&format!(
                            "{r:.3} {g:.3} {b:.3} rg {x0:.2} {y0:.2} m",
                            r = color[0],
                            g = color[1],
                            b = color[2],
                        ));
                        for (x, y) in points.iter().skip(1) {
                            stream.push_str(&format!(" {x:.2} {y:.2} l"));
                        }
                        stream.push_str(" h f\n");
                    }
                }
                Op::Cubic {
                    start,
                    segments,
                    width,
                    color,
                } => {
                    stream.push_str(&format!(
                        "{w:.2} w {r:.3} {g:.3} {b:.3} RG {x:.2} {y:.2} m",
                        w = width,
                        r = color[0],
                        g = color[1],
                        b = color[2],
                        x = start.0,
                        y = start.1,
                    ));
                    for [(c1x, c1y), (c2x, c2y), (ex, ey)] in segments {
                        stream.push_str(&format!(
                            " {c1x:.2} {c1y:.2} {c2x:.2} {c2y:.2} {ex:.2} {ey:.2} c"
                        ));
                    }
                    stream.push_str(" S\n");
                }
                Op::Jpeg { .. } | Op::Rgb { .. } => {}
            }
        }
        stream.push_str(&extra_ops);
        if markup.is_some() {
            stream.push_str("Q\n");
        }
        let content_id = objs.len() + 1;
        objs.push(stream_object(&stream));
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
            annot_refs.push_str(&format!("{id} 0 R "));
        }
        let annots = if annot_refs.is_empty() {
            String::new()
        } else {
            format!(" /Annots [{annot_refs}]")
        };
        let page_id = objs.len() + 1;
        page_ids.push(page_id);
        objs.push(
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {w:.2} {h:.2}] \
                   /Contents {content_id} 0 R \
                   /Resources << /Font << {font_res} >> /XObject << {xobjects} >> >>{annots} >>",
                w = page.width,
                h = page.height,
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

struct HashMapLite {
    items: Vec<(FaceId, usize)>,
}

impl HashMapLite {
    fn new() -> Self {
        Self { items: Vec::new() }
    }
    fn insert(&mut self, k: FaceId, v: usize) {
        self.items.push((k, v));
    }

    fn contains(&self, k: FaceId) -> bool {
        self.items.iter().any(|(id, _)| *id == k)
    }
}

impl<'a> IntoIterator for &'a HashMapLite {
    type Item = &'a (FaceId, usize);
    type IntoIter = std::slice::Iter<'a, (FaceId, usize)>;
    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

fn font_file_obj(ttf: &[u8]) -> Vec<u8> {
    let mut out = format!(
        "<< /Length {} /Length1 {} >>\nstream\n",
        ttf.len(),
        ttf.len()
    )
    .into_bytes();
    out.extend_from_slice(ttf);
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

fn type0_font_obj(face: &super::font::Face, cid_id: usize) -> Vec<u8> {
    let name = face.pdf_name();
    format!(
        "<< /Type /Font /Subtype /Type0 /BaseFont /{name} /Encoding /Identity-H \
           /DescendantFonts [{cid_id} 0 R] >>"
    )
    .into_bytes()
}

fn jpeg_xobject(width: u32, height: u32, bytes: &[u8]) -> Vec<u8> {
    let mut out = format!(
        "<< /Type /XObject /Subtype /Image /Width {width} /Height {height} \
           /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /DCTDecode \
           /Length {} >>\nstream\n",
        bytes.len()
    )
    .into_bytes();
    out.extend_from_slice(bytes);
    out.extend_from_slice(b"\nendstream");
    out
}

fn rgb_xobject(width: u32, height: u32, bytes: &[u8]) -> Vec<u8> {
    let mut out = format!(
        "<< /Type /XObject /Subtype /Image /Width {width} /Height {height} \
           /ColorSpace /DeviceRGB /BitsPerComponent 8 \
           /Length {} >>\nstream\n",
        bytes.len()
    )
    .into_bytes();
    out.extend_from_slice(bytes);
    out.extend_from_slice(b"\nendstream");
    out
}

fn text_annot_obj(note: &PdfComment) -> Vec<u8> {
    let x2 = note.x + note.w.max(12.0);
    let y2 = note.y + note.h.max(12.0);
    let contents = pdf_literal(note.contents.as_bytes());
    let author = pdf_literal(note.author.as_bytes());
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

fn stream_object(ops: &str) -> Vec<u8> {
    let bytes = ops.as_bytes();
    let mut out = format!("<< /Length {} >>\nstream\n", bytes.len()).into_bytes();
    out.extend_from_slice(bytes);
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

fn pdf_literal(bytes: &[u8]) -> String {
    let mut out = String::from("(");
    for &b in bytes {
        match b {
            b'(' | b')' | b'\\' => {
                out.push('\\');
                out.push(char::from(b));
            }
            32..=126 => out.push(char::from(b)),
            _ => out.push_str(&format!("\\{b:03o}")),
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

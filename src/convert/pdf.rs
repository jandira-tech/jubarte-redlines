// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! PDF 1.4 writer: embedded TTF (Identity-H), stroked rules, JPEG/RGB images.

use super::font::{FaceId, Fonts};

/// One drawing command on a page (PDF user space, origin bottom-left).
pub(crate) enum Op {
    Text {
        face: FaceId,
        size: f32,
        x: f32,
        y: f32,
        glyphs: Vec<u16>,
        color: [f32; 3],
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
}

/// One finished page, with the section `pgSz` it was laid out against.
pub(crate) struct Page {
    pub ops: Vec<Op>,
    pub width: f32,
    pub height: f32,
}

impl Page {
    pub(crate) fn new(width: f32, height: f32) -> Self {
        Self {
            ops: Vec::new(),
            width,
            height,
        }
    }
}

pub(crate) fn emit(fonts: &Fonts, pages: &[Page]) -> Vec<u8> {
    let used: Vec<FaceId> = {
        let mut seen = Vec::new();
        for page in pages {
            for op in &page.ops {
                if let Op::Text { face, .. } = op
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
    let mut font_obj = HashMapLite::new();
    for face_id in &used {
        let face = fonts.get(*face_id);
        let file_id = objs.len() + 1;
        objs.push(font_file_obj(face.id.bytes()));
        let desc_id = objs.len() + 1;
        objs.push(font_descriptor_obj(face, file_id));
        let cid_id = objs.len() + 1;
        objs.push(cid_font_obj(face, desc_id));
        let type0_id = objs.len() + 1;
        objs.push(type0_font_obj(face, cid_id));
        font_obj.insert(*face_id, type0_id);
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
        let mut font_res = String::new();
        for (face_id, obj_id) in &font_obj {
            font_res.push_str(&format!(
                "/{name} {obj_id} 0 R ",
                name = face_id.postscript()
            ));
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
                } => {
                    if glyphs.is_empty() {
                        continue;
                    }
                    let hex: String = glyphs.iter().map(|g| format!("{g:04X}")).collect();
                    stream.push_str(&format!(
                        "BT /{name} {size:.2} Tf {r:.3} {g:.3} {b:.3} rg {x:.2} {y:.2} Td <{hex}> Tj ET\n",
                        name = face.postscript(),
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
                Op::Jpeg { .. } | Op::Rgb { .. } => {}
            }
        }
        stream.push_str(&extra_ops);
        let content_id = objs.len() + 1;
        objs.push(stream_object(&stream));
        let page_id = objs.len() + 1;
        page_ids.push(page_id);
        objs.push(
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {w:.2} {h:.2}] \
                   /Contents {content_id} 0 R \
                   /Resources << /Font << {font_res} >> /XObject << {xobjects} >> >> >>",
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
    let name = face.id.postscript();
    format!(
        "<< /Type /FontDescriptor /FontName /{name} /Flags 32 \
           /FontBBox [{a} {b} {c} {d}] /ItalicAngle 0 \
           /Ascent {ascent} /Descent {descent} /CapHeight {ascent} /StemV 80 \
           /FontFile2 {file_id} 0 R >>",
        a = face.bbox[0],
        b = face.bbox[1],
        c = face.bbox[2],
        d = face.bbox[3],
        ascent = face.ascent as i32,
        descent = face.descent as i32,
    )
    .into_bytes()
}

fn cid_font_obj(face: &super::font::Face, desc_id: usize) -> Vec<u8> {
    let name = face.id.postscript();
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
    let name = face.id.postscript();
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

fn stream_object(ops: &str) -> Vec<u8> {
    let bytes = ops.as_bytes();
    let mut out = format!("<< /Length {} >>\nstream\n", bytes.len()).into_bytes();
    out.extend_from_slice(bytes);
    out.extend_from_slice(b"\nendstream");
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

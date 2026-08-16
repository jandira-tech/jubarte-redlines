// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Bundled metric-compatible faces (Carlito = Calibri, Liberation = Arial/Times)
//! plus glyph advances for wrap and PDF embedding.

use std::collections::HashMap;

/// A bundled face used by the DOCX→PDF writer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum FaceId {
    CarlitoRegular,
    CarlitoBold,
    CarlitoItalic,
    CarlitoBoldItalic,
    SansRegular,
    SansBold,
    SansItalic,
    SansBoldItalic,
    SerifRegular,
    SerifBold,
    SerifItalic,
    SerifBoldItalic,
}

impl FaceId {
    pub(crate) fn all() -> [Self; 12] {
        [
            Self::CarlitoRegular,
            Self::CarlitoBold,
            Self::CarlitoItalic,
            Self::CarlitoBoldItalic,
            Self::SansRegular,
            Self::SansBold,
            Self::SansItalic,
            Self::SansBoldItalic,
            Self::SerifRegular,
            Self::SerifBold,
            Self::SerifItalic,
            Self::SerifBoldItalic,
        ]
    }

    pub(crate) fn bytes(self) -> &'static [u8] {
        match self {
            Self::CarlitoRegular => include_bytes!("../../assets/fonts/Carlito-Regular.ttf"),
            Self::CarlitoBold => include_bytes!("../../assets/fonts/Carlito-Bold.ttf"),
            Self::CarlitoItalic => include_bytes!("../../assets/fonts/Carlito-Italic.ttf"),
            Self::CarlitoBoldItalic => include_bytes!("../../assets/fonts/Carlito-BoldItalic.ttf"),
            Self::SansRegular => include_bytes!("../../assets/fonts/LiberationSans-Regular.ttf"),
            Self::SansBold => include_bytes!("../../assets/fonts/LiberationSans-Bold.ttf"),
            Self::SansItalic => include_bytes!("../../assets/fonts/LiberationSans-Italic.ttf"),
            Self::SansBoldItalic => {
                include_bytes!("../../assets/fonts/LiberationSans-BoldItalic.ttf")
            }
            Self::SerifRegular => include_bytes!("../../assets/fonts/LiberationSerif-Regular.ttf"),
            Self::SerifBold => include_bytes!("../../assets/fonts/LiberationSerif-Bold.ttf"),
            Self::SerifItalic => include_bytes!("../../assets/fonts/LiberationSerif-Italic.ttf"),
            Self::SerifBoldItalic => {
                include_bytes!("../../assets/fonts/LiberationSerif-BoldItalic.ttf")
            }
        }
    }

    pub(crate) fn postscript(self) -> &'static str {
        match self {
            Self::CarlitoRegular => "Carlito",
            Self::CarlitoBold => "Carlito-Bold",
            Self::CarlitoItalic => "Carlito-Italic",
            Self::CarlitoBoldItalic => "Carlito-BoldItalic",
            Self::SansRegular => "LiberationSans",
            Self::SansBold => "LiberationSans-Bold",
            Self::SansItalic => "LiberationSans-Italic",
            Self::SansBoldItalic => "LiberationSans-BoldItalic",
            Self::SerifRegular => "LiberationSerif",
            Self::SerifBold => "LiberationSerif-Bold",
            Self::SerifItalic => "LiberationSerif-Italic",
            Self::SerifBoldItalic => "LiberationSerif-BoldItalic",
        }
    }
}

/// Parsed metrics + cmap for one bundled face.
pub(crate) struct Face {
    pub id: FaceId,
    pub upem: f32,
    pub ascent: f32,
    pub descent: f32,
    pub line_gap: f32,
    pub bbox: [i16; 4],
    pub widths: Vec<u16>,
    cmap: HashMap<u32, u16>,
}

impl Face {
    fn load(id: FaceId) -> Self {
        let bytes = id.bytes();
        let face = ttf_parser::Face::parse(bytes, 0).expect("bundled TTF is valid");
        let upem = f32::from(face.units_per_em());
        let ascent = f32::from(
            face.typographic_ascender()
                .unwrap_or_else(|| face.ascender()),
        );
        let descent = f32::from(
            face.typographic_descender()
                .unwrap_or_else(|| face.descender()),
        );
        let line_gap = f32::from(
            face.typographic_line_gap()
                .unwrap_or_else(|| face.line_gap()),
        );
        let glyph_count = face.number_of_glyphs();
        let mut widths = vec![0u16; glyph_count as usize];
        for (gid, slot) in widths.iter_mut().enumerate() {
            let glyph = ttf_parser::GlyphId(gid as u16);
            *slot = face.glyph_hor_advance(glyph).unwrap_or(0);
        }
        let mut cmap = HashMap::new();
        if let Some(table) = face.tables().cmap {
            for sub in table.subtables {
                if !sub.is_unicode() {
                    continue;
                }
                sub.codepoints(|cp| {
                    if let Some(gid) = sub.glyph_index(cp) {
                        cmap.entry(cp).or_insert(gid.0);
                    }
                });
            }
        }
        let bbox = face.global_bounding_box();
        Self {
            id,
            upem,
            ascent,
            descent,
            line_gap,
            bbox: [bbox.x_min, bbox.y_min, bbox.x_max, bbox.y_max],
            widths,
            cmap,
        }
    }

    pub(crate) fn glyph(&self, ch: char) -> u16 {
        self.cmap.get(&(ch as u32)).copied().unwrap_or(0)
    }

    pub(crate) fn advance_pt(&self, ch: char, size: f32) -> f32 {
        let gid = self.glyph(ch) as usize;
        let adv = self.widths.get(gid).copied().unwrap_or(0);
        f32::from(adv) * size / self.upem
    }

    pub(crate) fn width_pt(&self, text: &str, size: f32) -> f32 {
        self.shape(text, size).into_iter().map(|(_, a)| a).sum()
    }

    pub(crate) fn ascent_pt(&self, size: f32) -> f32 {
        self.ascent * size / self.upem
    }

    pub(crate) fn single_line_pt(&self, size: f32) -> f32 {
        (self.ascent + self.descent.abs() + self.line_gap) * size / self.upem
    }

    pub(crate) fn glyphs(&self, text: &str) -> Vec<u16> {
        self.shape(text, 11.0).into_iter().map(|(g, _)| g).collect()
    }

    /// HarfBuzz-compatible glyph ids + advances in points.
    pub(crate) fn shape(&self, text: &str, size: f32) -> Vec<(u16, f32)> {
        let Some(face) = rustybuzz::Face::from_slice(self.id.bytes(), 0) else {
            return text
                .chars()
                .map(|ch| (self.glyph(ch), self.advance_pt(ch, size)))
                .collect();
        };
        let mut buf = rustybuzz::UnicodeBuffer::new();
        buf.push_str(text);
        let out = rustybuzz::shape(&face, &[], buf);
        let infos = out.glyph_infos();
        let pos = out.glyph_positions();
        infos
            .iter()
            .zip(pos.iter())
            .map(|(info, p)| {
                let adv = p.x_advance as f32 / self.upem * size;
                (info.glyph_id as u16, adv)
            })
            .collect()
    }

    pub(crate) fn pdf_widths_1000(&self) -> Vec<i32> {
        self.widths
            .iter()
            .map(|&w| ((i32::from(w) * 1000) / self.upem as i32).max(0))
            .collect()
    }
}

/// Catalogue of the 12 bundled faces.
pub(crate) struct Fonts {
    faces: HashMap<FaceId, Face>,
}

impl Fonts {
    pub(crate) fn new() -> Self {
        let mut faces = HashMap::new();
        for id in FaceId::all() {
            faces.insert(id, Face::load(id));
        }
        Self { faces }
    }

    pub(crate) fn get(&self, id: FaceId) -> &Face {
        &self.faces[&id]
    }

    pub(crate) fn resolve(&self, family: &str, bold: bool, italic: bool) -> FaceId {
        let key = family
            .to_ascii_lowercase()
            .replace([' ', '-'], "")
            .replace("mt", "");
        let sans = key.contains("arial")
            || key.contains("helvetica")
            || key.contains("liberationsans")
            || key.contains("sansserif");
        let serif = key.contains("times")
            || key.contains("georgia")
            || key.contains("cambria")
            || key.contains("caladea")
            || key.contains("liberationserif")
            || key.contains("serif") && !key.contains("sans");
        match (sans, serif, bold, italic) {
            (true, _, false, false) => FaceId::SansRegular,
            (true, _, true, false) => FaceId::SansBold,
            (true, _, false, true) => FaceId::SansItalic,
            (true, _, true, true) => FaceId::SansBoldItalic,
            (_, true, false, false) => FaceId::SerifRegular,
            (_, true, true, false) => FaceId::SerifBold,
            (_, true, false, true) => FaceId::SerifItalic,
            (_, true, true, true) => FaceId::SerifBoldItalic,
            (_, _, false, false) => FaceId::CarlitoRegular,
            (_, _, true, false) => FaceId::CarlitoBold,
            (_, _, false, true) => FaceId::CarlitoItalic,
            (_, _, true, true) => FaceId::CarlitoBoldItalic,
        }
    }
}

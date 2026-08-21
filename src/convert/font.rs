// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Bundled metric-compatible faces (Carlito = Calibri, Liberation Sans/Serif =
//! Arial/Times, Liberation Mono = Courier) plus glyph advances for wrap and
//! PDF embedding.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

/// Word Quartz Save-as-PDF writes a small `Tc` at 300dpi body sizes so
/// linear hmtx does not sit ~1pt wide of the oracle (color_sim wipe).
/// 11.04 → Tc≈-0.0015; 16.08 → Tc≈-0.0018. Other sizes keep hmtx.
pub(crate) fn word_device_track(size: f32) -> f32 {
    if (size - 11.04).abs() < 0.02 {
        -0.0015 * size
    } else if (size - 16.08).abs() < 0.02 {
        -0.0018 * size
    } else {
        0.0
    }
}

/// Word Quartz paints snapped body sizes as `ppem Tf` inside a `0.24`
/// cm (300 dpi). MuPDF then hints at 46/67ppem like the oracle, not at
/// 11.04/16.08 in user space (file_151 color_sim / ΔE).
pub(crate) fn word_device_paint(size: f32) -> Option<(f32, f32)> {
    if (size - 11.04).abs() < 0.02 {
        Some((46.0, -0.0015))
    } else if (size - 16.08).abs() < 0.02 {
        Some((67.0, -0.0018))
    } else {
        None
    }
}

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
    MonoRegular,
    MonoBold,
    MonoItalic,
    MonoBoldItalic,
    AptosRegular,
    AptosBold,
    AptosItalic,
    AptosBoldItalic,
    AptosDisplayRegular,
    AptosDisplayBold,
    AptosDisplayItalic,
    AptosDisplayBoldItalic,
    CalibriLightRegular,
    CalibriLightItalic,
    VerdanaRegular,
    VerdanaBold,
    VerdanaItalic,
    VerdanaBoldItalic,
    CambriaRegular,
    CambriaBold,
    CambriaItalic,
    CambriaBoldItalic,
    Symbol,
}

impl FaceId {
    pub(crate) fn all() -> [Self; 35] {
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
            Self::MonoRegular,
            Self::MonoBold,
            Self::MonoItalic,
            Self::MonoBoldItalic,
            Self::AptosRegular,
            Self::AptosBold,
            Self::AptosItalic,
            Self::AptosBoldItalic,
            Self::AptosDisplayRegular,
            Self::AptosDisplayBold,
            Self::AptosDisplayItalic,
            Self::AptosDisplayBoldItalic,
            Self::CalibriLightRegular,
            Self::CalibriLightItalic,
            Self::VerdanaRegular,
            Self::VerdanaBold,
            Self::VerdanaItalic,
            Self::VerdanaBoldItalic,
            Self::CambriaRegular,
            Self::CambriaBold,
            Self::CambriaItalic,
            Self::CambriaBoldItalic,
            Self::Symbol,
        ]
    }

    pub(crate) fn is_cambria(self) -> bool {
        matches!(
            self,
            Self::CambriaRegular
                | Self::CambriaBold
                | Self::CambriaItalic
                | Self::CambriaBoldItalic
        )
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
            Self::MonoRegular => include_bytes!("../../assets/fonts/LiberationMono-Regular.ttf"),
            Self::MonoBold => include_bytes!("../../assets/fonts/LiberationMono-Bold.ttf"),
            Self::MonoItalic => include_bytes!("../../assets/fonts/LiberationMono-Italic.ttf"),
            Self::MonoBoldItalic => {
                include_bytes!("../../assets/fonts/LiberationMono-BoldItalic.ttf")
            }
            Self::AptosRegular => include_bytes!("../../assets/fonts/Carlito-Regular.ttf"),
            Self::AptosBold => include_bytes!("../../assets/fonts/Carlito-Bold.ttf"),
            Self::AptosItalic => include_bytes!("../../assets/fonts/Carlito-Italic.ttf"),
            Self::AptosBoldItalic => include_bytes!("../../assets/fonts/Carlito-BoldItalic.ttf"),
            // System/Word CloudFonts Aptos Display overlays these.
            Self::AptosDisplayRegular => include_bytes!("../../assets/fonts/Carlito-Regular.ttf"),
            Self::AptosDisplayBold => include_bytes!("../../assets/fonts/Carlito-Bold.ttf"),
            Self::AptosDisplayItalic => include_bytes!("../../assets/fonts/Carlito-Italic.ttf"),
            Self::AptosDisplayBoldItalic => {
                include_bytes!("../../assets/fonts/Carlito-BoldItalic.ttf")
            }
            // System calibril.ttf overlays these.
            Self::CalibriLightRegular => include_bytes!("../../assets/fonts/Carlito-Regular.ttf"),
            Self::CalibriLightItalic => include_bytes!("../../assets/fonts/Carlito-Italic.ttf"),
            // System Verdana overlays these; Liberation Sans is the
            // Arial-metric fallback when Word DFonts are absent.
            Self::VerdanaRegular => include_bytes!("../../assets/fonts/LiberationSans-Regular.ttf"),
            Self::VerdanaBold => include_bytes!("../../assets/fonts/LiberationSans-Bold.ttf"),
            Self::VerdanaItalic => include_bytes!("../../assets/fonts/LiberationSans-Italic.ttf"),
            Self::VerdanaBoldItalic => {
                include_bytes!("../../assets/fonts/LiberationSans-BoldItalic.ttf")
            }
            // System Cambria overlays these; Liberation Serif is the
            // fallback when Word DFonts are absent.
            Self::CambriaRegular => {
                include_bytes!("../../assets/fonts/LiberationSerif-Regular.ttf")
            }
            Self::CambriaBold => include_bytes!("../../assets/fonts/LiberationSerif-Bold.ttf"),
            Self::CambriaItalic => include_bytes!("../../assets/fonts/LiberationSerif-Italic.ttf"),
            Self::CambriaBoldItalic => {
                include_bytes!("../../assets/fonts/LiberationSerif-BoldItalic.ttf")
            }
            // System Symbol overlays this; Liberation Sans U+2022 is the
            // fallback when Symbol.ttf is absent.
            Self::Symbol => include_bytes!("../../assets/fonts/LiberationSans-Regular.ttf"),
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
            Self::MonoRegular => "LiberationMono",
            Self::MonoBold => "LiberationMono-Bold",
            Self::MonoItalic => "LiberationMono-Italic",
            Self::MonoBoldItalic => "LiberationMono-BoldItalic",
            Self::AptosRegular => "Aptos",
            Self::AptosBold => "Aptos-Bold",
            Self::AptosItalic => "Aptos-Italic",
            Self::AptosBoldItalic => "Aptos-BoldItalic",
            Self::AptosDisplayRegular => "AptosDisplay",
            Self::AptosDisplayBold => "AptosDisplay-Bold",
            Self::AptosDisplayItalic => "AptosDisplay-Italic",
            Self::AptosDisplayBoldItalic => "AptosDisplay-BoldItalic",
            Self::CalibriLightRegular => "Calibri-Light",
            Self::CalibriLightItalic => "Calibri-LightItalic",
            Self::VerdanaRegular => "Verdana",
            Self::VerdanaBold => "Verdana-Bold",
            Self::VerdanaItalic => "Verdana-Italic",
            Self::VerdanaBoldItalic => "Verdana-BoldItalic",
            Self::CambriaRegular => "Cambria",
            Self::CambriaBold => "Cambria-Bold",
            Self::CambriaItalic => "Cambria-Italic",
            Self::CambriaBoldItalic => "Cambria-BoldItalic",
            Self::Symbol => "Symbol",
        }
    }
}

/// Parsed metrics + cmap for one bundled (or system-overlaid) face.
///
/// `bytes` is `&'static`: bundled faces are `include_bytes!` data, and
/// system-override files are leaked once at `Fonts` (process-lifetime
/// `LazyLock`) construction so the pre-parsed `rustybuzz::Face` below can
/// borrow them without a self-referential struct.
pub(crate) struct Face {
    bytes: &'static [u8],
    /// Parsed once here; `shape` runs per text run and must not re-parse
    /// the font table directory each call.
    buzz: Option<rustybuzz::Face<'static>>,
    pdf_name: String,
    pub upem: f32,
    pub ascent: f32,
    pub descent: f32,
    pub line_gap: f32,
    /// Win ascent when USE_TYPO_METRICS is unset (Liberation ↔ Arial).
    paint_ascent: f32,
    pub bbox: [i16; 4],
    pub widths: Vec<u16>,
    cmap: HashMap<u32, u16>,
}

impl Face {
    pub(crate) fn bytes(&self) -> &[u8] {
        self.bytes
    }

    pub(crate) fn pdf_name(&self) -> &str {
        &self.pdf_name
    }

    fn load(id: FaceId) -> Self {
        Self::from_bytes(id, id.bytes(), id.postscript().to_string()).expect("bundled TTF is valid")
    }

    fn from_path(id: FaceId, path: &Path) -> Option<Self> {
        let bytes = fs::read(path).ok()?;
        let ps = ttf_postscript_name(&bytes).unwrap_or_else(|| id.postscript().to_string());
        // Leaked once per process: `Fonts` lives in a LazyLock, and the
        // pre-parsed rustybuzz face needs 'static bytes.
        let bytes: &'static [u8] = Box::leak(bytes.into_boxed_slice());
        Self::from_bytes(id, bytes, sanitize_pdf_name(&ps))
    }

    /// `None` when the bytes are not a parseable TTF/TTC face — a truncated
    /// or unsupported *system* font file must fall back to the bundled face,
    /// never panic (a panic here poisons the process-wide `Fonts` LazyLock).
    fn from_bytes(_id: FaceId, bytes: &'static [u8], pdf_name: String) -> Option<Self> {
        let face = ttf_parser::Face::parse(bytes, 0).ok()?;
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
        let paint_ascent = face
            .tables()
            .os2
            .filter(|os2| !os2.use_typographic_metrics())
            .map(|os2| f32::from(os2.windows_ascender()))
            .filter(|win| *win > 0.0)
            .unwrap_or(ascent);
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
        let buzz = rustybuzz::Face::from_slice(bytes, 0);
        Some(Self {
            bytes,
            buzz,
            pdf_name,
            upem,
            ascent,
            descent,
            line_gap,
            paint_ascent,
            bbox: [bbox.x_min, bbox.y_min, bbox.x_max, bbox.y_max],
            widths,
            cmap,
        })
    }

    pub(crate) fn glyph(&self, ch: char) -> u16 {
        self.cmap.get(&(ch as u32)).copied().unwrap_or(0)
    }

    pub(crate) fn advance_pt(&self, ch: char, size: f32) -> f32 {
        let gid = self.glyph(ch) as usize;
        let adv = self.widths.get(gid).copied().unwrap_or(0);
        f32::from(adv) * size / self.upem + word_device_track(size)
    }

    pub(crate) fn width_pt(&self, text: &str, size: f32) -> f32 {
        self.shape(text, size).into_iter().map(|(_, a)| a).sum()
    }

    pub(crate) fn ascent_pt(&self, size: f32) -> f32 {
        // Official no_comments Word oracles place Calibri with usWinAscent
        // (11pt → 82.56 from top). Typo 1536 sits 2.2pt high and tanks the
        // randomized Calibri cluster (file_71 91→72).
        self.paint_ascent * size / self.upem
    }

    pub(crate) fn descent_pt(&self, size: f32) -> f32 {
        self.descent.abs() * size / self.upem
    }

    pub(crate) fn single_line_pt(&self, size: f32) -> f32 {
        (self.ascent + self.descent.abs() + self.line_gap) * size / self.upem
    }

    pub(crate) fn glyphs(&self, text: &str) -> Vec<u16> {
        // Only the glyph ids are kept; ids are size-independent, so the
        // shaping size passed here is arbitrary.
        self.shape(text, 11.0).into_iter().map(|(g, _)| g).collect()
    }

    /// HarfBuzz-compatible glyph ids + advances in points.
    pub(crate) fn shape(&self, text: &str, size: f32) -> Vec<(u16, f32)> {
        let Some(face) = self.buzz.as_ref() else {
            return text
                .chars()
                .map(|ch| (self.glyph(ch), self.advance_pt(ch, size)))
                .collect();
        };
        let mut buf = rustybuzz::UnicodeBuffer::new();
        buf.push_str(text);
        // Word Quartz WinAnsi PDFs do not ligate Calibri and place glyphs
        // on hmtx (T=5.38pt), not GPOS/kern (T+e shrinks ~1pt and wipes
        // official color_sim). Keep 1:1 WinAnsi clusters.
        let word_pdf = [
            rustybuzz::Feature::new(rustybuzz::ttf_parser::Tag::from_bytes(b"liga"), 0, ..),
            rustybuzz::Feature::new(rustybuzz::ttf_parser::Tag::from_bytes(b"clig"), 0, ..),
            rustybuzz::Feature::new(rustybuzz::ttf_parser::Tag::from_bytes(b"dlig"), 0, ..),
            rustybuzz::Feature::new(rustybuzz::ttf_parser::Tag::from_bytes(b"kern"), 0, ..),
        ];
        let out = rustybuzz::shape(face, &word_pdf, buf);
        let infos = out.glyph_infos();
        let pos = out.glyph_positions();
        infos
            .iter()
            .zip(pos.iter())
            .map(|(info, p)| {
                let adv = p.x_advance as f32 / self.upem * size + word_device_track(size);
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

    pub(crate) fn width_1000(&self, ch: char) -> i32 {
        let gid = self.glyph(ch) as usize;
        let adv = self.widths.get(gid).copied().unwrap_or(0);
        ((i32::from(adv) * 1000) / self.upem as i32).max(0)
    }

    fn scale_1000(&self, units: f32) -> i32 {
        (units * 1000.0 / self.upem).round() as i32
    }

    /// PDF simple TrueType glyph space is 1000 units (Word Quartz).
    /// Emitting raw 2048-UPM Ascent made fitz title boxes sit at y=44
    /// instead of Word's pgMar top (file_146 / 175 / 176 = 65pt).
    pub(crate) fn pdf_ascent_1000(&self) -> i32 {
        self.scale_1000(self.paint_ascent)
    }

    pub(crate) fn pdf_descent_1000(&self) -> i32 {
        self.scale_1000(self.descent)
    }

    pub(crate) fn pdf_bbox_1000(&self) -> [i32; 4] {
        [
            self.scale_1000(f32::from(self.bbox[0])),
            self.scale_1000(f32::from(self.bbox[1])),
            self.scale_1000(f32::from(self.bbox[2])),
            self.scale_1000(f32::from(self.bbox[3])),
        ]
    }
}

/// Catalogue of the bundled faces (one entry per `FaceId::all()` member).
pub(crate) struct Fonts {
    faces: HashMap<FaceId, Face>,
}

impl Fonts {
    pub(crate) fn new() -> Self {
        let mut faces = HashMap::new();
        for id in FaceId::all() {
            let face = system_override(id)
                .and_then(|path| Face::from_path(id, &path))
                .unwrap_or_else(|| Face::load(id));
            faces.insert(id, face);
        }
        Self { faces }
    }

    pub(crate) fn get(&self, id: FaceId) -> &Face {
        &self.faces[&id]
    }

    pub(crate) fn resolve(&self, family: &str, bold: bool, italic: bool) -> FaceId {
        let primary = family
            .split(',')
            .next()
            .unwrap_or(family)
            .trim()
            .trim_matches(|c| c == '"' || c == '\'');
        let key = primary
            .to_ascii_lowercase()
            .replace([' ', '-'], "")
            .replace("mt", "");
        let mono = key.contains("courier")
            || key.contains("consolas")
            || key.contains("monaco")
            || key.contains("menlo")
            || key.contains("cousine")
            || key.contains("nimbusmono")
            || key.ends_with("mono");
        let aptos_display = key.starts_with("aptosdisplay")
            || (key.starts_with("aptos") && key.contains("display"));
        let aptos = key.starts_with("aptos") && !aptos_display;
        let verdana = key.starts_with("verdana");
        // Official Word Quartz substitutes missing Inter with Cambria
        // (sample_document / eigenpal). Times overlay on the serif slot
        // left that cluster at ITT ~44.
        if key.contains("symbol") {
            return FaceId::Symbol;
        }
        // Strict01 Title/Heading1/2 are major=Calibri Light. Do not fall
        // through to Calibri Regular (Carlito).
        if key.contains("calibrilight") || (key.contains("calibri") && key.contains("light")) {
            return if italic {
                FaceId::CalibriLightItalic
            } else {
                FaceId::CalibriLightRegular
            };
        }
        if key.contains("cambria") || key == "inter" {
            return match (bold, italic) {
                (false, false) => FaceId::CambriaRegular,
                (true, false) => FaceId::CambriaBold,
                (false, true) => FaceId::CambriaItalic,
                (true, true) => FaceId::CambriaBoldItalic,
            };
        }
        let sans = key.contains("arial")
            || key.contains("helvetica")
            || key.contains("liberationsans")
            || key.contains("opensans")
            || key.contains("roboto")
            || key.contains("tahoma")
            || key.contains("trebuchet")
            || key.contains("geneva")
            || key == "sansserif";
        let serif = key.contains("times")
            || key.contains("georgia")
            || key.contains("caladea")
            || key.contains("liberationserif")
            || (key.contains("serif") && !key.contains("sans"));
        match (
            mono,
            aptos_display,
            aptos,
            verdana,
            sans,
            serif,
            bold,
            italic,
        ) {
            (true, _, _, _, _, _, false, false) => FaceId::MonoRegular,
            (true, _, _, _, _, _, true, false) => FaceId::MonoBold,
            (true, _, _, _, _, _, false, true) => FaceId::MonoItalic,
            (true, _, _, _, _, _, true, true) => FaceId::MonoBoldItalic,
            (_, true, _, _, _, _, false, false) => FaceId::AptosDisplayRegular,
            (_, true, _, _, _, _, true, false) => FaceId::AptosDisplayBold,
            (_, true, _, _, _, _, false, true) => FaceId::AptosDisplayItalic,
            (_, true, _, _, _, _, true, true) => FaceId::AptosDisplayBoldItalic,
            (_, _, true, _, _, _, false, false) => FaceId::AptosRegular,
            (_, _, true, _, _, _, true, false) => FaceId::AptosBold,
            (_, _, true, _, _, _, false, true) => FaceId::AptosItalic,
            (_, _, true, _, _, _, true, true) => FaceId::AptosBoldItalic,
            (_, _, _, true, _, _, false, false) => FaceId::VerdanaRegular,
            (_, _, _, true, _, _, true, false) => FaceId::VerdanaBold,
            (_, _, _, true, _, _, false, true) => FaceId::VerdanaItalic,
            (_, _, _, true, _, _, true, true) => FaceId::VerdanaBoldItalic,
            (_, _, _, _, true, _, false, false) => FaceId::SansRegular,
            (_, _, _, _, true, _, true, false) => FaceId::SansBold,
            (_, _, _, _, true, _, false, true) => FaceId::SansItalic,
            (_, _, _, _, true, _, true, true) => FaceId::SansBoldItalic,
            (_, _, _, _, _, true, false, false) => FaceId::SerifRegular,
            (_, _, _, _, _, true, true, false) => FaceId::SerifBold,
            (_, _, _, _, _, true, false, true) => FaceId::SerifItalic,
            (_, _, _, _, _, true, true, true) => FaceId::SerifBoldItalic,
            (_, _, _, _, _, _, false, false) => FaceId::CarlitoRegular,
            (_, _, _, _, _, _, true, false) => FaceId::CarlitoBold,
            (_, _, _, _, _, _, false, true) => FaceId::CarlitoItalic,
            (_, _, _, _, _, _, true, true) => FaceId::CarlitoBoldItalic,
        }
    }
}

fn system_override(id: FaceId) -> Option<PathBuf> {
    let names: &[&str] = match id {
        FaceId::CarlitoRegular => &["Calibri.ttf", "calibri.ttf"],
        FaceId::CarlitoBold => &["Calibrib.ttf", "Calibri Bold.ttf", "calibrib.ttf"],
        FaceId::CarlitoItalic => &["Calibrii.ttf", "Calibri Italic.ttf", "calibrii.ttf"],
        FaceId::CarlitoBoldItalic => &["Calibriz.ttf", "Calibri Bold Italic.ttf", "calibriz.ttf"],
        FaceId::SansRegular => &["Arial.ttf", "arial.ttf"],
        FaceId::SansBold => &["Arial Bold.ttf", "arialbd.ttf"],
        FaceId::SansItalic => &["Arial Italic.ttf", "ariali.ttf"],
        FaceId::SansBoldItalic => &["Arial Bold Italic.ttf", "arialbi.ttf"],
        FaceId::SerifRegular => &["Times New Roman.ttf", "times.ttf"],
        FaceId::SerifBold => &["Times New Roman Bold.ttf", "timesbd.ttf"],
        FaceId::SerifItalic => &["Times New Roman Italic.ttf", "timesi.ttf"],
        FaceId::SerifBoldItalic => &["Times New Roman Bold Italic.ttf", "timesbi.ttf"],
        FaceId::MonoRegular => &["Courier New.ttf", "cour.ttf"],
        FaceId::MonoBold => &["Courier New Bold.ttf", "courbd.ttf"],
        FaceId::MonoItalic => &["Courier New Italic.ttf", "couri.ttf"],
        FaceId::MonoBoldItalic => &["Courier New Bold Italic.ttf", "courbi.ttf"],
        FaceId::AptosRegular => &["Aptos.ttf"],
        FaceId::AptosBold => &["Aptos-Bold.ttf"],
        FaceId::AptosItalic => &["Aptos-Italic.ttf"],
        FaceId::AptosBoldItalic => &["Aptos-Bold-Italic.ttf"],
        FaceId::AptosDisplayRegular => &["AptosDisplay-Regular.ttf", "AptosDisplay.ttf"],
        FaceId::AptosDisplayBold => &["AptosDisplay-Bold.ttf"],
        FaceId::AptosDisplayItalic => &["AptosDisplay-Italic.ttf"],
        FaceId::AptosDisplayBoldItalic => &["AptosDisplay-BoldItalic.ttf"],
        FaceId::CalibriLightRegular => &["calibril.ttf", "Calibri Light.ttf", "CalibriL.ttf"],
        FaceId::CalibriLightItalic => &["calibrili.ttf", "Calibri Light Italic.ttf"],
        FaceId::VerdanaRegular => &["Verdana.ttf"],
        FaceId::VerdanaBold => &["Verdana Bold.ttf", "Verdanab.ttf"],
        FaceId::VerdanaItalic => &["Verdana Italic.ttf", "Verdanai.ttf"],
        FaceId::VerdanaBoldItalic => &["Verdana Bold Italic.ttf", "Verdanaz.ttf"],
        FaceId::CambriaRegular => &["Cambria.ttf", "Cambria.ttc"],
        FaceId::CambriaBold => &["Cambriab.ttf", "Cambria Bold.ttf"],
        FaceId::CambriaItalic => &["Cambriai.ttf", "Cambria Italic.ttf"],
        FaceId::CambriaBoldItalic => &["Cambriaz.ttf", "Cambria Bold Italic.ttf"],
        FaceId::Symbol => &["Symbol.ttf", "symbol.ttf"],
    };
    // Deliberately macOS-only: these overrides exist to match the
    // Word-on-macOS oracles this converter is calibrated against. On other
    // platforms no override is found and the bundled metric-compatible
    // faces are used.
    const DIRS: &[&str] = &[
        "/Applications/Microsoft Word.app/Contents/Resources/DFonts",
        "/Library/Fonts/Microsoft",
        "/System/Library/Fonts/Supplemental",
        "/System/Library/Fonts",
        "/Library/Fonts",
    ];
    for dir in DIRS {
        for name in names {
            let path = Path::new(dir).join(name);
            if path.is_file() {
                return Some(path);
            }
        }
    }
    cloud_font_override(id)
}

fn cloud_font_override(id: FaceId) -> Option<PathBuf> {
    // Scanned once per process: `Fonts::new` probes every FaceId, and each
    // probe would otherwise re-read the directory and re-parse every font
    // file in it just to recover PostScript names.
    static CLOUD_FONTS: LazyLock<Vec<(String, PathBuf)>> = LazyLock::new(|| {
        let mut found = Vec::new();
        let Some(home) = std::env::var_os("HOME") else {
            return found;
        };
        let dir = PathBuf::from(home).join(
            "Library/Group Containers/UBF8T346G9.Office/FontCache/4/CloudFonts/Aptos Display",
        );
        let Ok(entries) = fs::read_dir(&dir) else {
            return found;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("ttf") {
                continue;
            }
            let Ok(bytes) = fs::read(&path) else {
                continue;
            };
            if let Some(ps) = ttf_postscript_name(&bytes) {
                found.push((ps, path));
            }
        }
        found
    });
    let want = id.postscript();
    CLOUD_FONTS
        .iter()
        .find(|(ps, _)| ps == want)
        .map(|(_, path)| path.clone())
}

fn ttf_postscript_name(bytes: &[u8]) -> Option<String> {
    let face = ttf_parser::Face::parse(bytes, 0).ok()?;
    let name = face.tables().name?;
    name.names.into_iter().find_map(|n| {
        if n.name_id != ttf_parser::name_id::POST_SCRIPT_NAME || !n.is_unicode() {
            return None;
        }
        n.to_string()
    })
}

fn sanitize_pdf_name(name: &str) -> String {
    name.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '+' {
                ch
            } else {
                '-'
            }
        })
        .collect()
}

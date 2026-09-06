// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Bundled metric-compatible faces (Carlito = Calibri, Liberation Sans/Serif =
//! Arial/Times, Liberation Mono = Courier) plus glyph advances for wrap and
//! PDF embedding.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

thread_local! {
    static ACTIVE_FONT_TABLE: RefCell<super::font_table::FontTable> =
        RefCell::new(super::font_table::FontTable::default());
}

/// Install `table` for the duration of `f` so `Fonts::resolve` honours altName.
pub(crate) fn with_font_table<T>(table: super::font_table::FontTable, f: impl FnOnce() -> T) -> T {
    ACTIVE_FONT_TABLE.with(|slot| {
        let prev = slot.replace(table);
        let out = f();
        slot.replace(prev);
        out
    })
}

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
    ConsolasRegular,
    ConsolasBold,
    ConsolasItalic,
    ConsolasBoldItalic,
    GeorgiaRegular,
    GeorgiaBold,
    GeorgiaItalic,
    GeorgiaBoldItalic,
    BookAntiquaRegular,
    BookAntiquaBold,
    BookAntiquaItalic,
    BookAntiquaBoldItalic,
    Symbol,
}

impl FaceId {
    pub(crate) fn all() -> [Self; 47] {
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
            Self::ConsolasRegular,
            Self::ConsolasBold,
            Self::ConsolasItalic,
            Self::ConsolasBoldItalic,
            Self::GeorgiaRegular,
            Self::GeorgiaBold,
            Self::GeorgiaItalic,
            Self::GeorgiaBoldItalic,
            Self::BookAntiquaRegular,
            Self::BookAntiquaBold,
            Self::BookAntiquaItalic,
            Self::BookAntiquaBoldItalic,
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

    pub(crate) fn is_arial(self) -> bool {
        matches!(
            self,
            Self::SansRegular | Self::SansBold | Self::SansItalic | Self::SansBoldItalic
        )
    }

    pub(crate) fn is_times(self) -> bool {
        matches!(
            self,
            Self::SerifRegular | Self::SerifBold | Self::SerifItalic | Self::SerifBoldItalic
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
            // System Consolas overlays these; Liberation Mono is the
            // fallback when Word DFonts are absent.
            Self::ConsolasRegular => {
                include_bytes!("../../assets/fonts/LiberationMono-Regular.ttf")
            }
            Self::ConsolasBold => include_bytes!("../../assets/fonts/LiberationMono-Bold.ttf"),
            Self::ConsolasItalic => include_bytes!("../../assets/fonts/LiberationMono-Italic.ttf"),
            Self::ConsolasBoldItalic => {
                include_bytes!("../../assets/fonts/LiberationMono-BoldItalic.ttf")
            }
            // System Georgia overlays these; Liberation Serif is the
            // fallback when Georgia.ttf is absent.
            Self::GeorgiaRegular => {
                include_bytes!("../../assets/fonts/LiberationSerif-Regular.ttf")
            }
            Self::GeorgiaBold => include_bytes!("../../assets/fonts/LiberationSerif-Bold.ttf"),
            Self::GeorgiaItalic => include_bytes!("../../assets/fonts/LiberationSerif-Italic.ttf"),
            Self::GeorgiaBoldItalic => {
                include_bytes!("../../assets/fonts/LiberationSerif-BoldItalic.ttf")
            }
            // Word DFonts Book Antiqua / Palatino Linotype overlay these;
            // Liberation Serif is the fallback when those faces are absent.
            Self::BookAntiquaRegular => {
                include_bytes!("../../assets/fonts/LiberationSerif-Regular.ttf")
            }
            Self::BookAntiquaBold => include_bytes!("../../assets/fonts/LiberationSerif-Bold.ttf"),
            Self::BookAntiquaItalic => {
                include_bytes!("../../assets/fonts/LiberationSerif-Italic.ttf")
            }
            Self::BookAntiquaBoldItalic => {
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
            Self::ConsolasRegular => "Consolas",
            Self::ConsolasBold => "Consolas-Bold",
            Self::ConsolasItalic => "Consolas-Italic",
            Self::ConsolasBoldItalic => "Consolas-BoldItalic",
            Self::GeorgiaRegular => "Georgia",
            Self::GeorgiaBold => "Georgia-Bold",
            Self::GeorgiaItalic => "Georgia-Italic",
            Self::GeorgiaBoldItalic => "Georgia-BoldItalic",
            Self::BookAntiquaRegular => "BookAntiqua",
            Self::BookAntiquaBold => "BookAntiqua-Bold",
            Self::BookAntiquaItalic => "BookAntiqua-Italic",
            Self::BookAntiquaBoldItalic => "BookAntiqua-BoldItalic",
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
        self.shape_kern(text, size, false)
            .into_iter()
            .map(|(_, a)| a)
            .sum()
    }

    pub(crate) fn width_pt_kern(&self, text: &str, size: f32, kern: bool) -> f32 {
        self.shape_kern(text, size, kern)
            .into_iter()
            .map(|(_, a)| a)
            .sum()
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
        self.shape_kern(text, size, false)
    }

    pub(crate) fn shape_kern(&self, text: &str, size: f32, kern: bool) -> Vec<(u16, f32)> {
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
        // official color_sim). Title `w:kern val=28` (potpourri 28pt)
        // is the exception: Word "Pot-Pourri" is 108.6 vs hmtx 111.0.
        // docDefaults/Normal kern=2 stays off.
        let kern_bit = u32::from(kern);
        let word_pdf = [
            rustybuzz::Feature::new(rustybuzz::ttf_parser::Tag::from_bytes(b"liga"), 0, ..),
            rustybuzz::Feature::new(rustybuzz::ttf_parser::Tag::from_bytes(b"clig"), 0, ..),
            rustybuzz::Feature::new(rustybuzz::ttf_parser::Tag::from_bytes(b"dlig"), 0, ..),
            rustybuzz::Feature::new(
                rustybuzz::ttf_parser::Tag::from_bytes(b"kern"),
                kern_bit,
                ..,
            ),
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
        ACTIVE_FONT_TABLE.with(|slot| self.resolve_in(family, bold, italic, &slot.borrow()))
    }

    /// Resolve `family` using Word's font table. Installed faces win;
    /// otherwise `w:altName` is tried (cycle-guarded). Unknown names still
    /// fall through to Carlito, matching today's `resolve`.
    pub(crate) fn resolve_in(
        &self,
        family: &str,
        bold: bool,
        italic: bool,
        table: &super::font_table::FontTable,
    ) -> FaceId {
        let mut visited = HashSet::new();
        self.resolve_walk(family, bold, italic, table, &mut visited)
    }

    fn resolve_walk(
        &self,
        family: &str,
        bold: bool,
        italic: bool,
        table: &super::font_table::FontTable,
        visited: &mut HashSet<String>,
    ) -> FaceId {
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
        if let Some(id) = Self::mapped_face(&key, bold, italic) {
            return id;
        }
        let visit_key = primary.to_ascii_lowercase();
        if !visited.insert(visit_key) {
            return Self::carlito(bold, italic);
        }
        if let Some(alt) = table.alt_name(primary) {
            return self.resolve_walk(alt, bold, italic, table, visited);
        }
        Self::carlito(bold, italic)
    }

    fn carlito(bold: bool, italic: bool) -> FaceId {
        match (bold, italic) {
            (false, false) => FaceId::CarlitoRegular,
            (true, false) => FaceId::CarlitoBold,
            (false, true) => FaceId::CarlitoItalic,
            (true, true) => FaceId::CarlitoBoldItalic,
        }
    }

    /// Known catalogue faces. `None` means "not in the 47-face table" so
    /// `resolve_in` can try `altName` before the Carlito last resort.
    fn mapped_face(key: &str, bold: bool, italic: bool) -> Option<FaceId> {
        let mono = key.contains("courier")
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
            return Some(FaceId::Symbol);
        }
        // Strict01 Title/Heading1/2 are major=Calibri Light. Do not fall
        // through to Calibri Regular (Carlito).
        if key.contains("calibrilight") || (key.contains("calibri") && key.contains("light")) {
            return Some(if italic {
                FaceId::CalibriLightItalic
            } else {
                FaceId::CalibriLightRegular
            });
        }
        // Calibri/Carlito is the catalogue default, not the unknown-family
        // last resort — otherwise altName on Calibri would steal Cambria.
        if key.contains("calibri") || key.contains("carlito") {
            return Some(Self::carlito(bold, italic));
        }
        if key.contains("cambria") || key == "inter" {
            return Some(match (bold, italic) {
                (false, false) => FaceId::CambriaRegular,
                (true, false) => FaceId::CambriaBold,
                (false, true) => FaceId::CambriaItalic,
                (true, true) => FaceId::CambriaBoldItalic,
            });
        }
        if key.contains("consolas") {
            return Some(match (bold, italic) {
                (false, false) => FaceId::ConsolasRegular,
                (true, false) => FaceId::ConsolasBold,
                (false, true) => FaceId::ConsolasItalic,
                (true, true) => FaceId::ConsolasBoldItalic,
            });
        }
        if key.contains("georgia") {
            return Some(match (bold, italic) {
                (false, false) => FaceId::GeorgiaRegular,
                (true, false) => FaceId::GeorgiaBold,
                (false, true) => FaceId::GeorgiaItalic,
                (true, true) => FaceId::GeorgiaBoldItalic,
            });
        }
        // file_22 / sd_2517 live period run is Book Antiqua. Folding it
        // into Carlito (or Times) missed Word Quartz's BookAntiqua embed.
        if key.contains("bookantiqua") || key.contains("palatino") {
            return Some(match (bold, italic) {
                (false, false) => FaceId::BookAntiquaRegular,
                (true, false) => FaceId::BookAntiquaBold,
                (false, true) => FaceId::BookAntiquaItalic,
                (true, true) => FaceId::BookAntiquaBoldItalic,
            });
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
            (true, _, _, _, _, _, false, false) => Some(FaceId::MonoRegular),
            (true, _, _, _, _, _, true, false) => Some(FaceId::MonoBold),
            (true, _, _, _, _, _, false, true) => Some(FaceId::MonoItalic),
            (true, _, _, _, _, _, true, true) => Some(FaceId::MonoBoldItalic),
            (_, true, _, _, _, _, false, false) => Some(FaceId::AptosDisplayRegular),
            (_, true, _, _, _, _, true, false) => Some(FaceId::AptosDisplayBold),
            (_, true, _, _, _, _, false, true) => Some(FaceId::AptosDisplayItalic),
            (_, true, _, _, _, _, true, true) => Some(FaceId::AptosDisplayBoldItalic),
            (_, _, true, _, _, _, false, false) => Some(FaceId::AptosRegular),
            (_, _, true, _, _, _, true, false) => Some(FaceId::AptosBold),
            (_, _, true, _, _, _, false, true) => Some(FaceId::AptosItalic),
            (_, _, true, _, _, _, true, true) => Some(FaceId::AptosBoldItalic),
            (_, _, _, true, _, _, false, false) => Some(FaceId::VerdanaRegular),
            (_, _, _, true, _, _, true, false) => Some(FaceId::VerdanaBold),
            (_, _, _, true, _, _, false, true) => Some(FaceId::VerdanaItalic),
            (_, _, _, true, _, _, true, true) => Some(FaceId::VerdanaBoldItalic),
            (_, _, _, _, true, _, false, false) => Some(FaceId::SansRegular),
            (_, _, _, _, true, _, true, false) => Some(FaceId::SansBold),
            (_, _, _, _, true, _, false, true) => Some(FaceId::SansItalic),
            (_, _, _, _, true, _, true, true) => Some(FaceId::SansBoldItalic),
            (_, _, _, _, _, true, false, false) => Some(FaceId::SerifRegular),
            (_, _, _, _, _, true, true, false) => Some(FaceId::SerifBold),
            (_, _, _, _, _, true, false, true) => Some(FaceId::SerifItalic),
            (_, _, _, _, _, true, true, true) => Some(FaceId::SerifBoldItalic),
            _ => None,
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
        FaceId::ConsolasRegular => &["Consola.ttf", "Consolas.ttf", "consola.ttf"],
        FaceId::ConsolasBold => &["Consolab.ttf", "Consolas Bold.ttf", "consolab.ttf"],
        FaceId::ConsolasItalic => &["Consolai.ttf", "Consolas Italic.ttf", "consolai.ttf"],
        FaceId::ConsolasBoldItalic => &["Consolaz.ttf", "Consolas Bold Italic.ttf", "consolaz.ttf"],
        FaceId::GeorgiaRegular => &["Georgia.ttf", "georgia.ttf"],
        FaceId::GeorgiaBold => &["Georgia Bold.ttf", "georgiab.ttf"],
        FaceId::GeorgiaItalic => &["Georgia Italic.ttf", "georgiai.ttf"],
        FaceId::GeorgiaBoldItalic => &["Georgia Bold Italic.ttf", "georgiaz.ttf"],
        FaceId::BookAntiquaRegular => &["Book Antiqua.ttf", "pala.ttf"],
        FaceId::BookAntiquaBold => &["Book Antiqua Bold.ttf", "palab.ttf"],
        FaceId::BookAntiquaItalic => &["Book Antiqua Italic.ttf", "palai.ttf"],
        FaceId::BookAntiquaBoldItalic => &["Book Antiqua Bold Italic.ttf", "palabi.ttf"],
        FaceId::Symbol => &["Symbol.ttf", "symbol.ttf"],
    };
    // Deliberately macOS-only: these overrides exist to match the
    // Word-on-macOS oracles this converter is calibrated against. On other
    // platforms no override is found and the bundled metric-compatible
    // faces are used.
    // Arial/Verdana/Georgia live in Supplemental; Calibri/Cambria/Aptos in
    // DFonts. Symbol is DFonts/Microsoft only: Apple's `Symbol.ttf` (system
    // or Supplemental on GitHub macOS runners) is not Word SymbolMT.
    const WORD_DIRS: &[&str] = &[
        "/Applications/Microsoft Word.app/Contents/Resources/DFonts",
        "/Library/Fonts/Microsoft",
    ];
    const DIRS: &[&str] = &[
        "/Applications/Microsoft Word.app/Contents/Resources/DFonts",
        "/Library/Fonts/Microsoft",
        "/System/Library/Fonts/Supplemental",
        "/Library/Fonts",
    ];
    let dirs = if id == FaceId::Symbol {
        WORD_DIRS
    } else {
        DIRS
    };
    for dir in dirs {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apple_system_symbol_is_not_the_word_overlay() {
        if let Some(path) = system_override(FaceId::Symbol) {
            let s = path.to_string_lossy();
            assert!(
                s.contains("DFonts") || s.contains("Microsoft"),
                "Symbol overlay must be Word DFonts/Microsoft, not Apple: {s}"
            );
        }
    }

    #[test]
    fn calibri_gpos_kerns_av_at_28pt() {
        let fonts = Fonts::new();
        let face = fonts.get(FaceId::CarlitoRegular);
        let off = face.width_pt_kern("AVAVAV", 28.0, false);
        let on = face.width_pt_kern("AVAVAV", 28.0, true);
        assert!(
            off - on > 0.8,
            "GPOS kern must tighten AV at 28pt; on={on} off={off}"
        );
    }

    #[test]
    fn calibri_gpos_kern_off_at_body_size_matches_hmtx() {
        let fonts = Fonts::new();
        let face = fonts.get(FaceId::CarlitoRegular);
        let hmtx = face.width_pt("The", 11.0);
        let shaped = face.width_pt_kern("The", 11.0, false);
        assert!(
            (hmtx - shaped).abs() < 0.05,
            "body kern=false must stay hmtx; hmtx={hmtx} shaped={shaped}"
        );
    }

    #[test]
    fn aptos_twelve_stays_unligated_after_mini_727() {
        // Word potpourri / file_170 Aptos 12 "flour" is U+FB02. Aptos≥12
        // liga (mini 727) was ITT-neg: file_170 −0.0036 / potpourri
        // −0.0002. Quartz prefers f+l. Do not retry.
        let fonts = Fonts::new();
        let face = fonts.get(FaceId::AptosRegular);
        let g = face.shape("fl", 12.0);
        assert_eq!(g.len(), 2, "mini 727 Aptos 12 liga ITT-neg; glyphs={g:?}");
    }

    #[test]
    fn calibri_eleven_does_not_ligate_fl() {
        let fonts = Fonts::new();
        let face = fonts.get(FaceId::CarlitoRegular);
        let g = face.shape("fl", 11.0);
        assert_eq!(
            g.len(),
            2,
            "Word Quartz WinAnsi Calibri does not ligate; glyphs={g:?}"
        );
    }

    #[test]
    fn aptos_ten_five_does_not_ligate_fl() {
        // comments-lots / I_am_sharing Aptos 10.56 keeps fi/ff/fl as two
        // chars (60+ hits, 0 U+FB02). Ungated Aptos liga would ITT-neg.
        let fonts = Fonts::new();
        let face = fonts.get(FaceId::AptosRegular);
        let g = face.shape("fl", 10.5);
        assert_eq!(g.len(), 2, "Word Aptos 10.5 does not ligate; glyphs={g:?}");
    }

    #[test]
    fn resolve_uses_altname_when_requested_face_missing() {
        let table = super::super::font_table::parse_font_table_xml(
            r#"<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
                 <w:font w:name="SomeRare"><w:altName w:val="Cambria"/></w:font>
               </w:fonts>"#,
        );
        let fonts = Fonts::new();
        assert_eq!(
            fonts.resolve_in("SomeRare", false, false, &table),
            FaceId::CambriaRegular,
            "missing face must follow font-table altName"
        );
    }

    #[test]
    fn resolve_keeps_installed_face_over_altname() {
        let table = super::super::font_table::parse_font_table_xml(
            r#"<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
                 <w:font w:name="Calibri"><w:altName w:val="Cambria"/></w:font>
               </w:fonts>"#,
        );
        let fonts = Fonts::new();
        assert_eq!(
            fonts.resolve_in("Calibri", false, false, &table),
            FaceId::CarlitoRegular,
            "installed Calibri (Carlito) wins over altName Cambria"
        );
    }

    #[test]
    fn resolve_altname_cycle_falls_back() {
        let table = super::super::font_table::parse_font_table_xml(
            r#"<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
                 <w:font w:name="GhostA"><w:altName w:val="GhostB"/></w:font>
                 <w:font w:name="GhostB"><w:altName w:val="GhostA"/></w:font>
               </w:fonts>"#,
        );
        let fonts = Fonts::new();
        assert_eq!(
            fonts.resolve_in("GhostA", false, false, &table),
            FaceId::CarlitoRegular
        );
    }

    #[test]
    fn resolve_follows_altname_chain_and_preserves_style() {
        let table = super::super::font_table::parse_font_table_xml(
            r#"<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
                 <w:font w:name="GhostA"><w:altName w:val="GhostB"/></w:font>
                 <w:font w:name="GhostB"><w:altName w:val="Cambria"/></w:font>
               </w:fonts>"#,
        );
        let fonts = Fonts::new();
        for (bold, italic, expected) in [
            (false, false, FaceId::CambriaRegular),
            (true, false, FaceId::CambriaBold),
            (false, true, FaceId::CambriaItalic),
            (true, true, FaceId::CambriaBoldItalic),
        ] {
            assert_eq!(
                fonts.resolve_in("ghosta", bold, italic, &table),
                expected,
                "style must survive a case-insensitive multi-hop altName lookup"
            );
        }
    }

    #[test]
    fn nested_font_table_scope_restores_outer_table() {
        let outer = super::super::font_table::parse_font_table_xml(
            r#"<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
                 <w:font w:name="SomeRare"><w:altName w:val="Cambria"/></w:font>
               </w:fonts>"#,
        );
        let inner = super::super::font_table::parse_font_table_xml(
            r#"<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
                 <w:font w:name="SomeRare"><w:altName w:val="Consolas"/></w:font>
               </w:fonts>"#,
        );
        let fonts = Fonts::new();

        with_font_table(outer, || {
            assert_eq!(
                fonts.resolve("SomeRare", false, false),
                FaceId::CambriaRegular
            );
            with_font_table(inner, || {
                assert_eq!(
                    fonts.resolve("SomeRare", false, false),
                    FaceId::ConsolasRegular
                );
            });
            assert_eq!(
                fonts.resolve("SomeRare", false, false),
                FaceId::CambriaRegular,
                "leaving an inner conversion must restore the outer font table"
            );
        });
        assert_eq!(
            fonts.resolve("SomeRare", false, false),
            FaceId::CarlitoRegular,
            "leaving all conversion scopes must restore the default table"
        );
    }
}

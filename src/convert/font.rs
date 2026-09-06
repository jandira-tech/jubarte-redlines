// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Bundled metric-compatible faces (Carlito = Calibri, Liberation Sans/Serif =
//! Arial/Times, Liberation Mono = Courier) plus glyph advances for wrap and
//! PDF embedding.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt::{self, Write as _};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex, OnceLock};

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

thread_local! {
    static FONT_REPORT: RefCell<Option<Vec<FontReportEntry>>> = const { RefCell::new(None) };
}

/// Collect distinct [`FontReportEntry`] rows produced by `Fonts::resolve`
/// inside `f` (plan Step 2f).
pub(crate) fn with_font_report<T>(f: impl FnOnce() -> T) -> (T, Vec<FontReportEntry>) {
    FONT_REPORT.with(|slot| {
        let prev = slot.replace(Some(Vec::new()));
        let out = f();
        let report = slot.replace(prev).unwrap_or_default();
        (out, report)
    })
}

fn record_font_resolution(entry: FontReportEntry) {
    FONT_REPORT.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(report) = slot.as_mut() else {
            return;
        };
        if report.iter().any(|existing| {
            existing.requested == entry.requested
                && existing.bold == entry.bold
                && existing.italic == entry.italic
        }) {
            return;
        }
        report.push(entry);
    });
}

/// Which resolve step produced the physical face (plan Step 2f).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FontStep {
    /// Document-embedded `.odttf` for this family + style.
    Embedded,
    /// Requested family is installed (or in the catalogue overlay).
    Explicit,
    /// `w:altName` from `fontTable.xml`.
    AltName,
    /// Theme slot with no explicit `w:ascii` (reserved; apply_rfonts today
    /// writes the slot name before resolve).
    Theme,
    /// Word-substitution evidence table (plan Step 2d).
    WordSubstitution,
    /// Bundled metric-compatible face; the requested family is not on disk.
    OpenFallback,
    /// `w:family` / `w:pitch` generic (roman / swiss / modern / fixed).
    Generic,
    /// Unknown family; evidence-table last resort (Cambria).
    Unknown,
}

impl FontStep {
    /// Stable JSON / CLI token for this step.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Embedded => "embedded",
            Self::Explicit => "explicit",
            Self::AltName => "altName",
            Self::Theme => "theme",
            Self::WordSubstitution => "word_substitution",
            Self::OpenFallback => "open_fallback",
            Self::Generic => "generic",
            Self::Unknown => "unknown",
        }
    }
}

impl fmt::Display for FontStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One distinct requested family + style from a conversion (plan Step 2f).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontReportEntry {
    /// Family string the run asked for (opaque; quotes/commas preserved).
    pub requested: String,
    /// Which resolve step selected the physical face.
    pub step: FontStep,
    /// Physical face actually used (PDF PostScript name).
    pub physical: String,
    /// Requested bold.
    pub bold: bool,
    /// Requested italic.
    pub italic: bool,
    /// True when the physical face does not provide the requested style.
    pub synthetic: bool,
}

impl FontReportEntry {
    /// One JSON object matching `{requested, step, physical, bold, italic, synthetic}`.
    #[must_use]
    pub fn to_json(&self) -> String {
        format!(
            "{{\"requested\":{},\"step\":{},\"physical\":{},\"bold\":{},\"italic\":{},\"synthetic\":{}}}",
            json_string(&self.requested),
            json_string(self.step.as_str()),
            json_string(&self.physical),
            json_bool(self.bold),
            json_bool(self.italic),
            json_bool(self.synthetic),
        )
    }
}

/// JSON array of [`FontReportEntry::to_json`] objects.
#[must_use]
pub fn font_report_json(entries: &[FontReportEntry]) -> String {
    let mut out = String::from("[");
    for (i, entry) in entries.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&entry.to_json());
    }
    out.push(']');
    out
}

fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn json_bool(v: bool) -> &'static str {
    if v { "true" } else { "false" }
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

/// Physical family + style. Catalogue `FaceId` values map here so call sites
/// can migrate off the closed enum (plan Step 2e) without a behaviour change.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct FaceKey {
    pub family: String,
    pub bold: bool,
    pub italic: bool,
}

/// Catalogue slot or a per-document embedded face (plan xml 3.1 ckpt 4).
///
/// Copy-sized so `Op::Text` can keep a font identity without a `String`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum FaceRef {
    Catalogue(FaceId),
    Embedded(u16),
}

impl From<FaceId> for FaceRef {
    fn from(id: FaceId) -> Self {
        Self::Catalogue(id)
    }
}

impl PartialEq<FaceId> for FaceRef {
    fn eq(&self, other: &FaceId) -> bool {
        matches!(self, Self::Catalogue(id) if id == other)
    }
}

impl FaceRef {
    pub(crate) fn is_arial(self) -> bool {
        matches!(self, Self::Catalogue(id) if id.is_arial())
    }

    pub(crate) fn is_cambria(self) -> bool {
        matches!(self, Self::Catalogue(id) if id.is_cambria())
    }

    pub(crate) fn is_times(self) -> bool {
        matches!(self, Self::Catalogue(id) if id.is_times())
    }
}

impl FaceId {
    fn index(self) -> usize {
        self as usize
    }

    pub(crate) fn key(self) -> FaceKey {
        FaceKey {
            family: self.logical_family().to_string(),
            bold: self.is_bold_style(),
            italic: self.is_italic_style(),
        }
    }

    fn logical_family(self) -> &'static str {
        match self {
            Self::CarlitoRegular
            | Self::CarlitoBold
            | Self::CarlitoItalic
            | Self::CarlitoBoldItalic => "Calibri",
            Self::SansRegular | Self::SansBold | Self::SansItalic | Self::SansBoldItalic => "Arial",
            Self::SerifRegular | Self::SerifBold | Self::SerifItalic | Self::SerifBoldItalic => {
                "Times New Roman"
            }
            Self::MonoRegular | Self::MonoBold | Self::MonoItalic | Self::MonoBoldItalic => {
                "Courier New"
            }
            Self::AptosRegular | Self::AptosBold | Self::AptosItalic | Self::AptosBoldItalic => {
                "Aptos"
            }
            Self::AptosDisplayRegular
            | Self::AptosDisplayBold
            | Self::AptosDisplayItalic
            | Self::AptosDisplayBoldItalic => "Aptos Display",
            Self::CalibriLightRegular | Self::CalibriLightItalic => "Calibri Light",
            Self::VerdanaRegular
            | Self::VerdanaBold
            | Self::VerdanaItalic
            | Self::VerdanaBoldItalic => "Verdana",
            Self::CambriaRegular
            | Self::CambriaBold
            | Self::CambriaItalic
            | Self::CambriaBoldItalic => "Cambria",
            Self::ConsolasRegular
            | Self::ConsolasBold
            | Self::ConsolasItalic
            | Self::ConsolasBoldItalic => "Consolas",
            Self::GeorgiaRegular
            | Self::GeorgiaBold
            | Self::GeorgiaItalic
            | Self::GeorgiaBoldItalic => "Georgia",
            Self::BookAntiquaRegular
            | Self::BookAntiquaBold
            | Self::BookAntiquaItalic
            | Self::BookAntiquaBoldItalic => "Book Antiqua",
            Self::Symbol => "Symbol",
        }
    }

    fn is_bold_style(self) -> bool {
        matches!(
            self,
            Self::CarlitoBold
                | Self::CarlitoBoldItalic
                | Self::SansBold
                | Self::SansBoldItalic
                | Self::SerifBold
                | Self::SerifBoldItalic
                | Self::MonoBold
                | Self::MonoBoldItalic
                | Self::AptosBold
                | Self::AptosBoldItalic
                | Self::AptosDisplayBold
                | Self::AptosDisplayBoldItalic
                | Self::VerdanaBold
                | Self::VerdanaBoldItalic
                | Self::CambriaBold
                | Self::CambriaBoldItalic
                | Self::ConsolasBold
                | Self::ConsolasBoldItalic
                | Self::GeorgiaBold
                | Self::GeorgiaBoldItalic
                | Self::BookAntiquaBold
                | Self::BookAntiquaBoldItalic
        )
    }

    fn is_italic_style(self) -> bool {
        matches!(
            self,
            Self::CarlitoItalic
                | Self::CarlitoBoldItalic
                | Self::SansItalic
                | Self::SansBoldItalic
                | Self::SerifItalic
                | Self::SerifBoldItalic
                | Self::MonoItalic
                | Self::MonoBoldItalic
                | Self::AptosItalic
                | Self::AptosBoldItalic
                | Self::AptosDisplayItalic
                | Self::AptosDisplayBoldItalic
                | Self::CalibriLightItalic
                | Self::VerdanaItalic
                | Self::VerdanaBoldItalic
                | Self::CambriaItalic
                | Self::CambriaBoldItalic
                | Self::ConsolasItalic
                | Self::ConsolasBoldItalic
                | Self::GeorgiaItalic
                | Self::GeorgiaBoldItalic
                | Self::BookAntiquaItalic
                | Self::BookAntiquaBoldItalic
        )
    }
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

/// Process-lifetime catalogue (one slot per `FaceId::all()` member).
/// Faces load on first `get` so a conversion that uses three families
/// does not parse the other forty-four (plan Step 2e).
///
/// `OnceLock<Box<Face>>` so the array is pointer-sized. `[OnceLock<Face>; 47]`
/// overflowed the Windows CLI stack (`jubarte convert` in convert_docx_to_pdf).
struct Catalogue {
    faces: [OnceLock<Box<Face>>; 47],
}

fn catalogue() -> &'static Catalogue {
    static CATALOGUE: LazyLock<Catalogue> = LazyLock::new(|| {
        debug_assert_eq!(FaceId::all().len(), 47);
        Catalogue {
            faces: std::array::from_fn(|_| OnceLock::new()),
        }
    });
    &CATALOGUE
}

impl Catalogue {
    fn get(&self, id: FaceId) -> &Face {
        self.faces[id.index()]
            .get_or_init(|| {
                Box::new(
                    system_override(id)
                        .and_then(|path| Face::from_path(id, &path))
                        .unwrap_or_else(|| Face::load(id)),
                )
            })
            .as_ref()
    }
}

/// Bundled catalogue plus per-document embedded faces (`.odttf`).
pub(crate) struct Fonts {
    extra: Vec<Face>,
    extra_index: HashMap<FaceKey, u16>,
}

impl Fonts {
    pub(crate) fn new() -> Self {
        Self {
            extra: Vec::new(),
            extra_index: HashMap::new(),
        }
    }

    pub(crate) fn for_document(
        pkg: &crate::opc::PartFs,
        table: &super::font_table::FontTable,
    ) -> Self {
        let mut fonts = Self::new();
        for ((family, bold, italic), bytes) in super::font_table::load_embedded_fonts(pkg, table) {
            fonts.insert_embedded(&family, bold, italic, bytes);
        }
        fonts
    }

    pub(crate) fn insert_embedded(
        &mut self,
        family: &str,
        bold: bool,
        italic: bool,
        bytes: Vec<u8>,
    ) {
        let leaked = intern_font_bytes(bytes);
        let ps = ttf_postscript_name(leaked).unwrap_or_else(|| family.to_string());
        let Some(face) = Face::from_bytes(FaceId::CarlitoRegular, leaked, sanitize_pdf_name(&ps))
        else {
            return;
        };
        let Ok(idx) = u16::try_from(self.extra.len()) else {
            return;
        };
        self.extra.push(face);
        self.extra_index.insert(
            FaceKey {
                family: family.to_ascii_lowercase(),
                bold,
                italic,
            },
            idx,
        );
    }

    fn embedded_index(&self, family: &str, bold: bool, italic: bool) -> Option<u16> {
        let exact = FaceKey {
            family: family.to_ascii_lowercase(),
            bold,
            italic,
        };
        if let Some(&idx) = self.extra_index.get(&exact) {
            return Some(idx);
        }
        if bold || italic {
            self.extra_index
                .get(&FaceKey {
                    family: exact.family,
                    bold: false,
                    italic: false,
                })
                .copied()
        } else {
            None
        }
    }

    pub(crate) fn get(&self, id: impl Into<FaceRef>) -> &Face {
        match id.into() {
            FaceRef::Catalogue(id) => self.get_key(&id.key()),
            FaceRef::Embedded(i) => self
                .extra
                .get(usize::from(i))
                .unwrap_or_else(|| catalogue().get(FaceId::CarlitoRegular)),
        }
    }

    pub(crate) fn get_key(&self, key: &FaceKey) -> &Face {
        if let Some(idx) = self.embedded_index(&key.family, key.bold, key.italic) {
            return self
                .extra
                .get(usize::from(idx))
                .unwrap_or_else(|| catalogue().get(FaceId::CarlitoRegular));
        }
        catalogue().get(Self::id_from_key(key))
    }

    fn id_from_key(key: &FaceKey) -> FaceId {
        Self::mapped_face(
            &key.family
                .to_ascii_lowercase()
                .replace([' ', '-'], "")
                .replace("mt", ""),
            key.bold,
            key.italic,
        )
        .unwrap_or(FaceId::CambriaRegular)
    }

    pub(crate) fn resolve(&self, family: &str, bold: bool, italic: bool) -> FaceRef {
        let (face, entry) =
            ACTIVE_FONT_TABLE.with(|slot| self.classify_in(family, bold, italic, &slot.borrow()));
        record_font_resolution(entry);
        face
    }

    /// Resolve `family` and classify the step without recording a report row.
    pub(crate) fn classify_in(
        &self,
        family: &str,
        bold: bool,
        italic: bool,
        table: &super::font_table::FontTable,
    ) -> (FaceRef, FontReportEntry) {
        let primary = family_token(family);
        if let Some(idx) = self.embedded_index(primary, bold, italic) {
            let face = FaceRef::Embedded(idx);
            let exact = self.extra_index.contains_key(&FaceKey {
                family: primary.to_ascii_lowercase(),
                bold,
                italic,
            });
            return (
                face,
                FontReportEntry {
                    requested: family.to_string(),
                    step: FontStep::Embedded,
                    physical: self.get(face).pdf_name().to_string(),
                    bold,
                    italic,
                    synthetic: (bold || italic) && !exact,
                },
            );
        }
        let (id, step) = self.resolve_in_step(family, bold, italic, table);
        let face = FaceRef::Catalogue(id);
        (
            face,
            FontReportEntry {
                requested: family.to_string(),
                step,
                physical: self.get(face).pdf_name().to_string(),
                bold,
                italic,
                synthetic: (bold && !id.is_bold_style()) || (italic && !id.is_italic_style()),
            },
        )
    }

    /// Resolve `family` using Word's font table. Installed faces win;
    /// otherwise `w:altName`, then the Word-substitution evidence table,
    /// then `w:family`/`w:pitch` generics. Unknown names use the evidence
    /// table's Cambria row (plan Step 2d).
    #[cfg(test)]
    pub(crate) fn resolve_in(
        &self,
        family: &str,
        bold: bool,
        italic: bool,
        table: &super::font_table::FontTable,
    ) -> FaceId {
        self.resolve_in_step(family, bold, italic, table).0
    }

    fn resolve_in_step(
        &self,
        family: &str,
        bold: bool,
        italic: bool,
        table: &super::font_table::FontTable,
    ) -> (FaceId, FontStep) {
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
    ) -> (FaceId, FontStep) {
        // Word splits rFonts on comma but does not CSS-unquote. Evidence
        // (Quartz PDFs): `Verdana, Geneva, sans-serif` → Verdana;
        // `"Times New Roman", Times, serif` → Cambria, because the first
        // token still carries the quote characters and is not TNR.
        let primary = family_token(family);
        let quoted = primary.starts_with('"') || primary.starts_with('\'');
        if !quoted {
            let key = primary
                .to_ascii_lowercase()
                .replace([' ', '-'], "")
                .replace("mt", "");
            if let Some(id) = Self::mapped_face(&key, bold, italic) {
                return (id, Self::catalogue_step(id));
            }
        }
        let visit_key = primary.to_ascii_lowercase();
        if !visited.insert(visit_key) {
            return (
                Self::face_from_physical(&super::word_subst::unknown_physical(), bold, italic),
                FontStep::Unknown,
            );
        }
        if let Some(alt) = table.alt_name(primary) {
            let (id, _) = self.resolve_walk(alt, bold, italic, table, visited);
            return (id, FontStep::AltName);
        }
        if let Some(physical) = super::word_subst::lookup_physical(primary) {
            return (
                Self::face_from_physical(&physical, bold, italic),
                FontStep::WordSubstitution,
            );
        }
        if let Some(entry) = table.get(primary) {
            let generic = super::word_subst::generic_physical(entry.family, entry.pitch);
            if !generic.is_empty() {
                return (
                    Self::face_from_physical(generic, bold, italic),
                    FontStep::Generic,
                );
            }
        }
        (
            Self::face_from_physical(&super::word_subst::unknown_physical(), bold, italic),
            FontStep::Unknown,
        )
    }

    fn catalogue_step(id: FaceId) -> FontStep {
        if system_override(id).is_some() {
            FontStep::Explicit
        } else {
            FontStep::OpenFallback
        }
    }

    fn face_from_physical(physical: &str, bold: bool, italic: bool) -> FaceId {
        let key = physical
            .to_ascii_lowercase()
            .replace([' ', '-'], "")
            .replace("mt", "");
        Self::mapped_face(&key, bold, italic).unwrap_or(match (bold, italic) {
            (false, false) => FaceId::CambriaRegular,
            (true, false) => FaceId::CambriaBold,
            (false, true) => FaceId::CambriaItalic,
            (true, true) => FaceId::CambriaBoldItalic,
        })
    }
}

fn family_token(family: &str) -> &str {
    let (token, listed) = match family.split_once(',') {
        Some((first, _)) => (first.trim(), true),
        None => (family.trim(), false),
    };
    if listed {
        token
    } else {
        strip_outer_quotes(token)
    }
}

fn strip_outer_quotes(s: &str) -> &str {
    let t = s.trim();
    let bytes = t.as_bytes();
    match (bytes.first(), bytes.last()) {
        (Some(b'"'), Some(b'"')) | (Some(b'\''), Some(b'\'')) if bytes.len() >= 2 => {
            t[1..t.len() - 1].trim()
        }
        _ => t,
    }
}

fn intern_font_bytes(bytes: Vec<u8>) -> &'static [u8] {
    use std::hash::{Hash, Hasher};
    static INTERN: Mutex<Vec<(u64, &'static [u8])>> = Mutex::new(Vec::new());
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    bytes.hash(&mut hasher);
    let h = hasher.finish();
    let mut intern = INTERN
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some((_, existing)) = intern
        .iter()
        .find(|(k, existing)| *k == h && **existing == *bytes)
    {
        return existing;
    }
    let leaked: &'static [u8] = Box::leak(bytes.into_boxed_slice());
    intern.push((h, leaked));
    leaked
}

impl Fonts {
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
            || (key.ends_with("mono") && !key.contains("dejavu"));
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
    fn faceid_key_is_unique_for_every_catalogue_slot() {
        let fonts = Fonts::new();
        let mut seen = HashSet::new();
        for id in FaceId::all() {
            let key = id.key();
            assert!(
                seen.insert(key.clone()),
                "duplicate FaceKey for {id:?}: {key:?}"
            );
            assert_eq!(
                fonts.get_key(&key).pdf_name(),
                fonts.get(id).pdf_name(),
                "shim get_key must match get({id:?})"
            );
        }
    }

    #[test]
    fn faceid_key_calibri_bold_italic() {
        let key = FaceId::CarlitoBoldItalic.key();
        assert_eq!(key.family, "Calibri");
        assert!(key.bold && key.italic);
    }

    #[test]
    fn fonts_get_is_idempotent() {
        let fonts = Fonts::new();
        let a = fonts.get(FaceId::CarlitoRegular) as *const Face;
        let b = fonts.get(FaceId::CarlitoRegular) as *const Face;
        assert_eq!(a, b);
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
    fn resolve_keeps_quoted_css_list_intact() {
        let fonts = Fonts::new();
        let id = fonts.resolve(r#""Times New Roman", Times, serif"#, false, false);
        assert_ne!(
            id,
            FaceId::SerifRegular,
            "quoted first token is not Times New Roman; Word Quartz used Cambria"
        );
        assert_eq!(
            id,
            FaceId::CambriaRegular,
            "unknown quoted CSS list is the evidence-table unknown row (Cambria)"
        );
        assert_eq!(
            fonts.resolve("Verdana, Geneva, sans-serif", false, false),
            FaceId::VerdanaRegular,
            "unquoted first token Verdana is installed"
        );
    }

    #[test]
    fn resolve_quoted_cambria_still_finds_cambria() {
        let fonts = Fonts::new();
        assert_eq!(
            fonts.resolve(r#""Cambria""#, false, false),
            FaceId::CambriaRegular
        );
        assert_eq!(
            fonts.resolve("Cambria", false, false),
            FaceId::CambriaRegular
        );
    }

    #[test]
    fn resolve_dejavu_sans_mono_follows_word_subst_verdana() {
        let fonts = Fonts::new();
        assert_eq!(
            fonts.resolve("DejaVu Sans Mono", false, false),
            FaceId::VerdanaRegular,
            "Word Quartz substituted Verdana, not Courier, for DejaVu Sans Mono"
        );
    }

    #[test]
    fn resolve_unknown_family_is_cambria_not_calibri() {
        let fonts = Fonts::new();
        assert_eq!(
            fonts.resolve("DefinitelyNotAFont", false, false),
            FaceId::CambriaRegular
        );
    }

    #[test]
    fn resolve_wide_latin_stays_calibri_mini_505() {
        let fonts = Fonts::new();
        assert_eq!(
            fonts.resolve("Wide Latin", false, false),
            FaceId::CarlitoRegular,
            "mini 505 ITT-neg WideLatin overlay; keep Calibri"
        );
    }

    #[test]
    fn resolve_empty_family_is_times_new_roman() {
        let fonts = Fonts::new();
        assert_eq!(
            fonts.resolve("", false, false),
            FaceId::SerifRegular,
            "no docDefaults font: Word used Times New Roman"
        );
    }

    #[test]
    fn resolve_font_table_swiss_generic_is_arial() {
        let table = super::super::font_table::parse_font_table_xml(
            r#"<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
                 <w:font w:name="SomeSwiss"><w:family w:val="swiss"/></w:font>
               </w:fonts>"#,
        );
        let fonts = Fonts::new();
        assert_eq!(
            fonts.resolve_in("SomeSwiss", false, false, &table),
            FaceId::SansRegular
        );
    }

    #[test]
    fn resolve_font_table_roman_generic_is_times() {
        let table = super::super::font_table::parse_font_table_xml(
            r#"<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
                 <w:font w:name="SomeRoman"><w:family w:val="roman"/></w:font>
               </w:fonts>"#,
        );
        let fonts = Fonts::new();
        assert_eq!(
            fonts.resolve_in("SomeRoman", false, false, &table),
            FaceId::SerifRegular
        );
    }

    #[test]
    fn resolve_font_table_fixed_pitch_is_courier() {
        let table = super::super::font_table::parse_font_table_xml(
            r#"<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
                 <w:font w:name="SomeFixed"><w:pitch w:val="fixed"/></w:font>
               </w:fonts>"#,
        );
        let fonts = Fonts::new();
        assert_eq!(
            fonts.resolve_in("SomeFixed", false, false, &table),
            FaceId::MonoRegular
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
            FaceId::CambriaRegular,
            "altName cycle uses the unknown-family evidence row"
        );
    }

    #[test]
    fn resolve_prefers_embedded_unknown_family() {
        let mut fonts = Fonts::new();
        assert_eq!(
            fonts.resolve("Press Start 2P", false, false),
            FaceId::CambriaRegular,
            "without an embed, the unknown family is Cambria"
        );
        fonts.insert_embedded(
            "Press Start 2P",
            false,
            false,
            FaceId::MonoRegular.bytes().to_vec(),
        );
        let face = fonts.resolve("Press Start 2P", false, false);
        assert!(
            matches!(face, FaceRef::Embedded(_)),
            "embedded face must win over the unknown-family row; got {face:?}"
        );
        assert_eq!(fonts.get(face).pdf_name(), "LiberationMono");
        assert_eq!(
            fonts.resolve("Press Start 2P", true, false),
            face,
            "missing bold embed falls back to the regular embed"
        );
    }

    #[test]
    fn resolve_embedded_does_not_steal_unrelated_families() {
        let mut fonts = Fonts::new();
        fonts.insert_embedded(
            "Press Start 2P",
            false,
            false,
            FaceId::MonoRegular.bytes().to_vec(),
        );
        assert_eq!(
            fonts.resolve("Calibri", false, false),
            FaceId::CarlitoRegular
        );
        assert_eq!(
            fonts.resolve("Cambria", false, false),
            FaceId::CambriaRegular
        );
    }

    fn obfuscated_embed_docx(family: &str, guid: &str, ttf: &[u8]) -> Vec<u8> {
        use std::io::{Cursor, Write};
        let mut odttf = ttf.to_vec();
        let key = super::super::font_table::parse_font_key(guid).expect("guid");
        super::super::font_table::deobfuscate_font(&mut odttf, &key);
        let mut buf = Vec::new();
        {
            let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
            let opt = zip::write::SimpleFileOptions::default();
            z.start_file("[Content_Types].xml", opt).unwrap();
            z.write_all(
                br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Default Extension="odttf" ContentType="application/vnd.openxmlformats-officedocument.obfuscatedFont"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/><Override PartName="/word/fontTable.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.fontTable+xml"/></Types>"#,
            )
            .unwrap();
            z.start_file("_rels/.rels", opt).unwrap();
            z.write_all(
                br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rIdM" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#,
            )
            .unwrap();
            z.start_file("word/document.xml", opt).unwrap();
            let doc = format!(
                r#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:rPr><w:rFonts w:ascii="{family}" w:hAnsi="{family}"/></w:rPr><w:t>HELLO</w:t></w:r></w:p><w:sectPr><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440"/></w:sectPr></w:body></w:document>"#
            );
            z.write_all(doc.as_bytes()).unwrap();
            z.start_file("word/fontTable.xml", opt).unwrap();
            let table = format!(
                r#"<?xml version="1.0"?><w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><w:font w:name="{family}"><w:embedRegular r:id="rId1" w:fontKey="{guid}"/></w:font></w:fonts>"#
            );
            z.write_all(table.as_bytes()).unwrap();
            z.start_file("word/_rels/fontTable.xml.rels", opt).unwrap();
            z.write_all(
                br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/font" Target="fonts/font1.odttf"/></Relationships>"#,
            )
            .unwrap();
            z.start_file("word/fonts/font1.odttf", opt).unwrap();
            z.write_all(&odttf).unwrap();
            z.finish().unwrap();
        }
        buf
    }

    #[test]
    fn convert_emits_embedded_postscript_name() {
        let guid = "{00000000-0000-0000-0000-000000000001}";
        let docx = obfuscated_embed_docx("Press Start 2P", guid, FaceId::MonoRegular.bytes());
        let pdf = super::super::docx_to_pdf(&docx).expect("convert");
        let text = String::from_utf8_lossy(&pdf);
        assert!(
            text.contains("/LiberationMono"),
            "embedded physical PostScript name must be a PDF font; snippet={}",
            text.chars().take(800).collect::<String>()
        );
    }

    fn simple_docx_with_ascii_font(family: &str) -> Vec<u8> {
        use std::io::{Cursor, Write};
        let mut buf = Vec::new();
        {
            let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
            let opt = zip::write::SimpleFileOptions::default();
            z.start_file("[Content_Types].xml", opt).unwrap();
            z.write_all(
                br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#,
            )
            .unwrap();
            z.start_file("_rels/.rels", opt).unwrap();
            z.write_all(
                br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rIdM" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#,
            )
            .unwrap();
            z.start_file("word/document.xml", opt).unwrap();
            let doc = format!(
                r#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:rPr><w:rFonts w:ascii="{family}" w:hAnsi="{family}"/></w:rPr><w:t>HELLO</w:t></w:r></w:p><w:sectPr><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440"/></w:sectPr></w:body></w:document>"#
            );
            z.write_all(doc.as_bytes()).unwrap();
            z.finish().unwrap();
        }
        buf
    }

    #[test]
    fn docx_to_pdf_report_includes_requested_unknown_family() {
        let docx = simple_docx_with_ascii_font("DefinitelyNotAFont");
        let out = super::super::docx_to_pdf_report(&docx, super::super::PdfOptions::default())
            .expect("convert");
        assert!(out.pdf.starts_with(b"%PDF"));
        let row = out
            .font_report
            .iter()
            .find(|e| e.requested == "DefinitelyNotAFont")
            .expect("unknown family must appear in the report");
        assert_eq!(row.step, FontStep::Unknown);
        let json = font_report_json(&out.font_report);
        assert!(json.starts_with('['), "{json}");
        assert!(json.contains("\"step\":\"unknown\""), "{json}");
    }

    #[test]
    fn classify_unknown_family_is_unknown_step() {
        let fonts = Fonts::new();
        let table = super::super::font_table::FontTable::default();
        let (_, entry) = fonts.classify_in("DefinitelyNotAFont", false, false, &table);
        assert_eq!(entry.step, FontStep::Unknown);
        assert_eq!(entry.requested, "DefinitelyNotAFont");
        assert!(!entry.physical.is_empty());
        assert!(!entry.synthetic);
    }

    #[test]
    fn classify_dejavu_sans_mono_is_word_substitution() {
        let fonts = Fonts::new();
        let table = super::super::font_table::FontTable::default();
        let (_, entry) = fonts.classify_in("DejaVu Sans Mono", false, false, &table);
        assert_eq!(entry.step, FontStep::WordSubstitution);
        assert_eq!(entry.requested, "DejaVu Sans Mono");
    }

    #[test]
    fn classify_empty_family_is_word_substitution() {
        let fonts = Fonts::new();
        let table = super::super::font_table::FontTable::default();
        let (face, entry) = fonts.classify_in("", false, false, &table);
        assert_eq!(entry.step, FontStep::WordSubstitution);
        assert_eq!(face, FaceId::SerifRegular);
    }

    #[test]
    fn classify_quoted_css_list_is_unknown() {
        let fonts = Fonts::new();
        let table = super::super::font_table::FontTable::default();
        let (_, entry) =
            fonts.classify_in(r#""Times New Roman", Times, serif"#, false, false, &table);
        assert_eq!(entry.step, FontStep::Unknown);
        assert_eq!(entry.requested, r#""Times New Roman", Times, serif"#);
    }

    #[test]
    fn classify_altname_step_is_altname() {
        let table = super::super::font_table::parse_font_table_xml(
            r#"<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
                 <w:font w:name="SomeRare"><w:altName w:val="Cambria"/></w:font>
               </w:fonts>"#,
        );
        let fonts = Fonts::new();
        let (_, entry) = fonts.classify_in("SomeRare", false, false, &table);
        assert_eq!(entry.step, FontStep::AltName);
        assert_eq!(entry.requested, "SomeRare");
    }

    #[test]
    fn classify_swiss_generic_step_is_generic() {
        let table = super::super::font_table::parse_font_table_xml(
            r#"<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
                 <w:font w:name="SomeSwiss"><w:family w:val="swiss"/></w:font>
               </w:fonts>"#,
        );
        let fonts = Fonts::new();
        let (_, entry) = fonts.classify_in("SomeSwiss", false, false, &table);
        assert_eq!(entry.step, FontStep::Generic);
    }

    #[test]
    fn classify_calibri_is_explicit_or_open_fallback() {
        let fonts = Fonts::new();
        let table = super::super::font_table::FontTable::default();
        let (_, entry) = fonts.classify_in("Calibri", false, false, &table);
        assert!(
            matches!(entry.step, FontStep::Explicit | FontStep::OpenFallback),
            "Calibri must be installed or bundled, got {:?}",
            entry.step
        );
        assert!(!entry.physical.is_empty());
    }

    #[test]
    fn classify_embedded_unknown_family_is_embedded_step() {
        let mut fonts = Fonts::new();
        fonts.insert_embedded(
            "Press Start 2P",
            false,
            false,
            FaceId::MonoRegular.bytes().to_vec(),
        );
        let table = super::super::font_table::FontTable::default();
        let (_, regular) = fonts.classify_in("Press Start 2P", false, false, &table);
        assert_eq!(regular.step, FontStep::Embedded);
        assert_eq!(regular.physical, "LiberationMono");
        assert!(!regular.synthetic);
        let (_, bold) = fonts.classify_in("Press Start 2P", true, false, &table);
        assert_eq!(bold.step, FontStep::Embedded);
        assert!(bold.synthetic, "missing bold embed is synthetic");
    }

    #[test]
    fn font_report_json_shape() {
        let entry = FontReportEntry {
            requested: r#"Calibri "body""#.to_string(),
            step: FontStep::Explicit,
            physical: "Calibri".to_string(),
            bold: false,
            italic: true,
            synthetic: false,
        };
        let json = font_report_json(std::slice::from_ref(&entry));
        assert_eq!(
            json,
            r#"[{"requested":"Calibri \"body\"","step":"explicit","physical":"Calibri","bold":false,"italic":true,"synthetic":false}]"#
        );
    }

    #[test]
    fn with_font_report_records_distinct_requests_only() {
        let fonts = Fonts::new();
        let ((), report) = with_font_report(|| {
            let _ = fonts.resolve("DefinitelyNotAFont", false, false);
            let _ = fonts.resolve("DefinitelyNotAFont", false, false);
            let _ = fonts.resolve("Calibri", true, false);
        });
        assert_eq!(
            report.len(),
            2,
            "same requested+style must collapse: {report:?}"
        );
        assert_eq!(report[0].requested, "DefinitelyNotAFont");
        assert_eq!(report[0].step, FontStep::Unknown);
        assert_eq!(report[1].requested, "Calibri");
        assert!(report[1].bold);
    }
}

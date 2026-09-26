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
use std::sync::{Arc, LazyLock, Mutex, OnceLock};

thread_local! {
    static ACTIVE_FONT_TABLE: RefCell<super::font_table::FontTable> =
        RefCell::new(super::font_table::FontTable::default());
}

/// Puts the previous thread-local value back when dropped, so a scope
/// that unwinds (a panicking conversion caught by a test harness or a
/// long-lived host) cannot leak its state into the next conversion.
struct RestoreOnDrop<'a, T> {
    slot: &'a RefCell<T>,
    prev: Option<T>,
}

impl<T> Drop for RestoreOnDrop<'_, T> {
    fn drop(&mut self) {
        if let Some(prev) = self.prev.take() {
            self.slot.replace(prev);
        }
    }
}

/// Install `table` for the duration of `f` so `Fonts::resolve` honours altName.
pub(crate) fn with_font_table<T>(table: super::font_table::FontTable, f: impl FnOnce() -> T) -> T {
    ACTIVE_FONT_TABLE.with(|slot| {
        let _restore = RestoreOnDrop {
            slot,
            prev: Some(slot.replace(table)),
        };
        f()
    })
}

thread_local! {
    static FONT_REPORT: RefCell<Option<Vec<FontReportEntry>>> = const { RefCell::new(None) };
}

/// Collect distinct [`FontReportEntry`] rows produced by `Fonts::resolve`
/// inside `f` (plan Step 2f).
pub(crate) fn with_font_report<T>(f: impl FnOnce() -> T) -> (T, Vec<FontReportEntry>) {
    FONT_REPORT.with(|slot| {
        let mut restore = RestoreOnDrop {
            slot,
            prev: Some(slot.replace(Some(Vec::new()))),
        };
        let out = f();
        let prev = restore.prev.take().flatten();
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

/// Parsed metrics + cmap for one bundled, system-overlaid or embedded face.
///
/// `bytes` outlives the face: bundled faces are `include_bytes!` data,
/// system-override files are leaked once into the process-lifetime
/// catalogue (one per slot), and a document's embedded fonts are borrowed
/// from the conversion that loaded them, so the pre-parsed
/// `rustybuzz::Face` below can borrow them without a self-referential
/// struct and nothing untrusted outlives its document.
pub(crate) struct Face<'a> {
    bytes: &'a [u8],
    /// Parsed once here; `shape` runs per text run and must not re-parse
    /// the font table directory each call.
    buzz: Option<rustybuzz::Face<'a>>,
    pdf_name: String,
    pub upem: f32,
    pub descent: f32,
    /// Word's single line in font units: hhea ascender − descender +
    /// lineGap (typo when USE_TYPO_METRICS is set). GDI reaches the same
    /// total as win height + external leading.
    line_height: f32,
    /// hhea descender (typo with USE_TYPO_METRICS): `line_height`'s part
    /// below the baseline.
    line_descent: f32,
    /// Win ascent when USE_TYPO_METRICS is unset (Liberation ↔ Arial).
    paint_ascent: f32,
    /// An East Asian face (OS/2 code pages 932/936/949/950/1361).
    east_asian: bool,
    pub bbox: [i16; 4],
    pub widths: Vec<u16>,
    cmap: HashMap<u32, u16>,
    /// Shape plans by segment (direction, script, language) and kerning:
    /// building one was a fifth of a conversion when every run built its
    /// own (redline 0006f790: 21% of samples in `ShapePlan::new`).
    plans: Mutex<HashMap<PlanKey, Arc<rustybuzz::ShapePlan>>>,
    /// Shaped text in font units by (text, kern); see `shaped_units`.
    shaped: Mutex<HashMap<(String, bool), ShapedUnits>>,
}

/// Glyph ids and x advances in font units.
/// (glyph id, x advance in font units, cluster = byte offset of its text).
type ShapedUnits = Arc<[(u16, i32, u32)]>;

type PlanKey = (
    rustybuzz::Direction,
    rustybuzz::Script,
    Option<rustybuzz::Language>,
    bool,
);

impl<'a> Face<'a> {
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
    fn from_bytes(_id: FaceId, bytes: &'a [u8], pdf_name: String) -> Option<Self> {
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
        // ttf-parser's ascender/descender/line_gap are hhea unless the font
        // sets USE_TYPO_METRICS. Typo metrics under-size Courier (0.80 em
        // vs 1.13) and Arial (1.09 vs 1.15) against Word's line.
        let mut line_height =
            f32::from(face.ascender()) - f32::from(face.descender()) + f32::from(face.line_gap());
        // The line's part below the baseline, from the same table as its
        // height (Courier's typo descender under-sizes it).
        let mut line_descent = f32::from(face.descender()).abs();
        // Word gives an East Asian face (OS/2 code pages 932/936/949/950/
        // 1361) 1.3 times its hhea ascent + descent, lineGap aside, the
        // extra split above and below the text. Live Word at 12pt: SimSun
        // and MS Mincho step 15.6, YaHei 20.7, Meiryo 23.3, Yu Gothic 17.0.
        let east_asian_line = cjk_code_pages(&face).then(|| {
            let body = f32::from(face.ascender()) - f32::from(face.descender());
            (body * 1.3, (body * 0.3) / 2.0)
        });
        if let Some((height, half)) = east_asian_line {
            line_height = height;
            line_descent += half;
        }
        // macOS Helvetica: Word's single line is 1.2 em, not its 1.0 em
        // hhea body, with the win descent below the baseline (English part
        // a 18f71536: 14.40pt lines at 12pt). Futura and Palatino follow
        // their hhea lines, so the rule is Helvetica's own.
        let helvetica = pdf_name == "Helvetica" || pdf_name.starts_with("Helvetica-");
        if helvetica && let Some(os2) = face.tables().os2 {
            line_height = upem * 1.2;
            line_descent = f32::from(os2.windows_descender()).abs();
        }
        // GDI puts the external leading (hhea total − win total) above the
        // text: Word's first TNR 12 baseline is winAscent + 0.51pt down.
        // A win box taller than the line (macOS Palatino: 3396 units over a
        // 2253 hhea line) does not push the text down: Word sets 009df71a's
        // Palatino at its hhea ascent, 9.05pt under the margin at 11pt.
        let paint_ascent = face
            .tables()
            .os2
            .filter(|os2| !os2.use_typographic_metrics())
            .filter(|os2| os2.windows_ascender() > 0)
            .map(|os2| {
                let win_asc = f32::from(os2.windows_ascender());
                let win_total = win_asc + f32::from(os2.windows_descender()).abs();
                if win_total > line_height {
                    line_height - line_descent
                } else {
                    win_asc + (line_height - win_total)
                }
            })
            .unwrap_or(ascent);
        let paint_ascent = match east_asian_line {
            Some((_, half)) => f32::from(face.ascender()) + half,
            None => paint_ascent,
        };
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
            plans: Mutex::new(HashMap::new()),
            shaped: Mutex::new(HashMap::new()),
            pdf_name,
            upem,
            descent,
            line_height,
            line_descent,
            paint_ascent,
            east_asian: east_asian_line.is_some(),
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
        self.line_height * size / self.upem
    }

    /// The single line's part below the baseline (hhea descender, the same
    /// table as `single_line_pt`).
    pub(crate) fn line_descent_pt(&self, size: f32) -> f32 {
        self.line_descent * size / self.upem
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
        let units = self.shaped_units(face, text, kern);
        units
            .iter()
            .map(|&(gid, x_advance, _)| {
                let adv = x_advance as f32 / self.upem * size + word_device_track(size);
                (gid, adv)
            })
            .collect()
    }

    /// The text behind each glyph `shape_kern(text, _, kern)` returns: its
    /// cluster's characters on the cluster's first glyph, empty on the
    /// rest. A glyph shaped from several characters (`e` + U+0301 composed
    /// to `é`, a lam-alef) carries all of them, for `/ToUnicode`.
    pub(crate) fn glyph_texts(&self, text: &str, kern: bool) -> Vec<String> {
        let Some(face) = self.buzz.as_ref() else {
            return text.chars().map(String::from).collect();
        };
        let units = self.shaped_units(face, text, kern);
        let mut starts: Vec<usize> = units.iter().map(|u| u.2 as usize).collect();
        starts.sort_unstable();
        starts.dedup();
        let mut seen = std::collections::HashSet::new();
        units
            .iter()
            .map(|u| {
                let at = u.2 as usize;
                if !seen.insert(at) {
                    return String::new();
                }
                let end = starts
                    .iter()
                    .find(|&&s| s > at)
                    .copied()
                    .unwrap_or(text.len());
                text.get(at..end).unwrap_or_default().to_string()
            })
            .collect()
    }

    /// `text` shaped in font units (glyph id, x advance), remembered per
    /// face: documents repeat their words, and shaping each again was a
    /// fifth of a conversion.
    fn shaped_units(&self, face: &rustybuzz::Face, text: &str, kern: bool) -> ShapedUnits {
        let memo = (text.to_string(), kern);
        if let Some(hit) = self.shaped.lock().ok().and_then(|c| c.get(&memo).cloned()) {
            return hit;
        }
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
        buf.guess_segment_properties();
        let key = (buf.direction(), buf.script(), buf.language(), kern);
        let cached = self.plans.lock().ok().and_then(|p| p.get(&key).cloned());
        let plan = cached.unwrap_or_else(|| {
            let plan = Arc::new(rustybuzz::ShapePlan::new(
                face,
                key.0,
                Some(key.1),
                key.2.as_ref(),
                &word_pdf,
            ));
            if let Ok(mut plans) = self.plans.lock() {
                plans.insert(key.clone(), Arc::clone(&plan));
            }
            plan
        });
        let out = rustybuzz::shape_with_plan(face, &plan, buf);
        let mut glyphs: Vec<(u16, i32, u32)> = out
            .glyph_infos()
            .iter()
            .zip(out.glyph_positions())
            .map(|(info, p)| (info.glyph_id as u16, p.x_advance, info.cluster))
            .collect();
        // rustybuzz 0.20 reverses an RTL buffer for a legacy `kern` table
        // and, with kerning off, skips the reverse back: Arial's Persian
        // came out in logical order, each word mirrored (00205272).
        // Visual RTL order runs from the last cluster to the first.
        if key.0 == rustybuzz::Direction::RightToLeft
            && glyphs
                .first()
                .zip(glyphs.last())
                .is_some_and(|(a, z)| a.2 < z.2)
        {
            glyphs.reverse();
        }
        let units: ShapedUnits = glyphs.into();
        if let Ok(mut cache) = self.shaped.lock() {
            // A long-lived caller (Python / WASM) converts many documents
            // through one face; keep the memory bounded.
            if cache.len() > 50_000 {
                cache.clear();
            }
            cache.insert(memo, Arc::clone(&units));
        }
        units
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
    faces: [OnceLock<Box<Face<'static>>>; 47],
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
    fn get(&self, id: FaceId) -> &Face<'static> {
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

/// A document's decoded embedded fonts (`.odttf`), keyed by
/// (family, bold, italic). The conversion owns them; [`Fonts`] borrows.
/// Values are shared: installed faces come from a process-wide cache and
/// one face can answer to a family and its altName.
pub(crate) type EmbeddedFonts = HashMap<(String, bool, bool), Arc<[u8]>>;

/// Bundled catalogue plus per-document embedded faces (`.odttf`).
pub(crate) struct Fonts<'a> {
    extra: Vec<Face<'a>>,
    extra_index: HashMap<FaceKey, u16>,
}

impl<'a> Fonts<'a> {
    pub(crate) fn new() -> Self {
        Self {
            extra: Vec::new(),
            extra_index: HashMap::new(),
        }
    }

    /// Faces for one document, borrowing its embedded font bytes: they are
    /// dropped with the conversion (#11: they were leaked process-wide).
    pub(crate) fn for_document(embedded: &'a EmbeddedFonts) -> Self {
        let mut fonts = Self::new();
        for ((family, bold, italic), bytes) in embedded {
            fonts.insert_embedded(family, *bold, *italic, bytes.as_ref());
        }
        fonts
    }

    pub(crate) fn insert_embedded(
        &mut self,
        family: &str,
        bold: bool,
        italic: bool,
        bytes: &'a [u8],
    ) {
        // PDF FontFile2 carries TrueType outlines only: a CFF face (an .otf
        // or a CFF .odttf) would reach the writer unembeddable. Such a family
        // resolves as if the face were absent.
        if ttf_parser::Face::parse(bytes, 0)
            .ok()
            .is_none_or(|face| face.tables().glyf.is_none())
        {
            return;
        }
        let ps = ttf_postscript_name(bytes).unwrap_or_else(|| family.to_string());
        let Some(face) = Face::from_bytes(FaceId::CarlitoRegular, bytes, sanitize_pdf_name(&ps))
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

    /// Word's face for an East Asian family it does not have: Microsoft
    /// YaHei for Chinese (0025b0d3's absent 標楷體, the 方正 families),
    /// Yu Gothic for Japanese (font-table charset 80 or kana in the name).
    fn cjk_fallback_index(
        &self,
        family: &str,
        bold: bool,
        table: &super::font_table::FontTable,
    ) -> Option<u16> {
        let charset = table.get(family).and_then(|e| e.charset.as_deref());
        let east_asian_charset = matches!(charset, Some("80" | "86" | "88" | "81"));
        if !east_asian_charset && !family.chars().any(is_cjk_name_char) {
            return None;
        }
        let japanese = charset == Some("80")
            || family.chars().any(|c| {
                ('\u{3040}'..='\u{30FF}').contains(&c) || ('\u{FF66}'..='\u{FF9F}').contains(&c)
            });
        let key = if japanese {
            CJK_FALLBACK_JA
        } else {
            CJK_FALLBACK
        };
        self.embedded_index(key, bold, false)
    }

    /// Word's Thaana face (MV Boli) for Dhivehi the resolved face lacks.
    pub(crate) fn thaana_glyph_fallback(&self, bold: bool) -> Option<FaceRef> {
        self.embedded_index(THAANA_FALLBACK, bold, false)
            .map(FaceRef::Embedded)
    }

    /// The CJK fallback face for a glyph the resolved face lacks.
    pub(crate) fn cjk_glyph_fallback(&self, bold: bool) -> Option<FaceRef> {
        self.embedded_index(CJK_FALLBACK, bold, false)
            .map(FaceRef::Embedded)
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

    /// `family` is present and not an East Asian face: Calibri, not SimSun
    /// (or a CJK name that is not installed, whose substitute says nothing).
    pub(crate) fn family_is_latin_only(&self, family: &str) -> bool {
        if !family.is_ascii() || !cjk_file_stems(family).is_empty() {
            return false;
        }
        let present =
            self.embedded_index(family, false, false).is_some() || catalogue_paints_family(family);
        present && !self.get(self.resolve(family, false, false)).east_asian
    }

    pub(crate) fn get(&self, id: impl Into<FaceRef>) -> &Face<'a> {
        match id.into() {
            FaceRef::Catalogue(id) => self.get_key(&id.key()),
            FaceRef::Embedded(i) => self
                .extra
                .get(usize::from(i))
                .unwrap_or_else(|| catalogue().get(FaceId::CarlitoRegular)),
        }
    }

    pub(crate) fn get_key(&self, key: &FaceKey) -> &Face<'a> {
        if let Some(idx) = self.embedded_index(&key.family, key.bold, key.italic) {
            return self
                .extra
                .get(usize::from(idx))
                .unwrap_or_else(|| catalogue().get(FaceId::CarlitoRegular));
        }
        catalogue().get(Self::id_from_key(key))
    }

    /// Unknown families fall back to the Cambria face of the same style,
    /// as [`Self::face_from_physical`] does (not always Cambria Regular).
    fn id_from_key(key: &FaceKey) -> FaceId {
        Self::face_from_physical(&key.family, key.bold, key.italic)
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
        if let Some(idx) = self.cjk_fallback_index(primary, bold, table) {
            let face = FaceRef::Embedded(idx);
            return (
                face,
                FontReportEntry {
                    requested: family.to_string(),
                    step: FontStep::Generic,
                    physical: self.get(face).pdf_name().to_string(),
                    bold,
                    italic,
                    synthetic: false,
                },
            );
        }
        let mut visited = HashSet::new();
        let (id, step, via_default) = self.resolve_walk(family, bold, italic, table, &mut visited);
        // An unknown family="auto" face paints in the document default,
        // which may itself be an embedded-only face (PR #167 review).
        if let Some(default) = via_default
            && let Some(idx) = self.embedded_index(family_token(default), bold, italic)
        {
            let face = FaceRef::Embedded(idx);
            let exact = self.extra_index.contains_key(&FaceKey {
                family: family_token(default).to_ascii_lowercase(),
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

    #[cfg(test)]
    fn resolve_in_step(
        &self,
        family: &str,
        bold: bool,
        italic: bool,
        table: &super::font_table::FontTable,
    ) -> (FaceId, FontStep) {
        let mut visited = HashSet::new();
        let (id, step, _) = self.resolve_walk(family, bold, italic, table, &mut visited);
        (id, step)
    }

    /// Follow `family` through the font table. Iterative: an altName chain
    /// is bounded by the table's size, never by the call stack. The third
    /// value is the document default an unknown family="auto" face fell
    /// back to, if the walk took that hop.
    fn resolve_walk<'t>(
        &self,
        family: &'t str,
        bold: bool,
        italic: bool,
        table: &'t super::font_table::FontTable,
        visited: &mut HashSet<String>,
    ) -> (FaceId, FontStep, Option<&'t str>) {
        let mut current = family;
        let mut via_alt = false;
        let mut via_default = None;
        // The generic of a name the altName chain passed through: a chain
        // that dead-ends (Myriad Pro → absent Segoe UI) keeps it.
        let mut chain_generic = "";
        let (id, step) = loop {
            // Word splits rFonts on comma but does not CSS-unquote. Evidence
            // (Quartz PDFs): `Verdana, Geneva, sans-serif` → Verdana;
            // `"Times New Roman", Times, serif` → Cambria, because the first
            // token still carries the quote characters and is not TNR.
            let primary = family_token(current);
            let quoted = primary.starts_with('"') || primary.starts_with('\'');
            if !quoted {
                let key = primary
                    .to_ascii_lowercase()
                    .replace([' ', '-'], "")
                    .replace("mt", "");
                if let Some(id) = Self::mapped_face(&key, bold, italic) {
                    break (id, Self::catalogue_step(id));
                }
            }
            let visit_key = primary.to_ascii_lowercase();
            if !visited.insert(visit_key) {
                break (
                    Self::face_from_physical(&super::word_subst::unknown_physical(), bold, italic),
                    FontStep::Unknown,
                );
            }
            // Word records its substitution against the whole `w:name`, so a
            // CSS-style list row (`"Foo", Bar, serif`) is keyed by the full
            // string, not by its first token.
            let whole = current.trim();
            // The family/pitch generic stands behind an entry that describes
            // its face (a panose past the family kind) or names an altName:
            // 019d9ee6's Myriad Pro (swiss, altName Segoe UI) is Arial in
            // Word, while 01177cdf's Museo Sans (0200…0, "modern", no
            // altName) takes the document's default.
            if chain_generic.is_empty()
                && let Some(entry) = table.get(primary)
                && entry_describes(entry)
            {
                chain_generic = super::word_subst::generic_physical(entry.family, entry.pitch);
            }
            let alt = table
                .alt_name(primary)
                .or_else(|| (whole != primary).then(|| table.alt_name(whole)).flatten());
            if let Some(alt) = alt {
                current = alt;
                via_alt = true;
                continue;
            }
            if let Some(physical) = super::word_subst::lookup_physical(primary) {
                break (
                    Self::face_from_physical(&physical, bold, italic),
                    FontStep::WordSubstitution,
                );
            }
            if !chain_generic.is_empty() {
                break (
                    Self::face_from_physical(chain_generic, bold, italic),
                    FontStep::Generic,
                );
            }
            // A missing Arabic or Hebrew face (charset B2 / B1) is Arial in
            // Word, spaces and all: 00205272's absent "B Compset" draws
            // Persian and 3.33pt spaces in Arial, not Cambria.
            if let Some(entry) = table.get(primary)
                && entry
                    .charset
                    .as_deref()
                    .is_some_and(|c| c.eq_ignore_ascii_case("B2") || c.eq_ignore_ascii_case("B1"))
            {
                break (
                    Self::face_from_physical("Arial", bold, italic),
                    FontStep::Generic,
                );
            }
            // An unknown face Word knows nothing about (family="auto", no
            // panose) paints in the document's default font: 010300e3's
            // Serenity and 00b5aa69's Shivaji01 are Calibri there.
            if let Some(entry) = table.get(primary)
                && !entry_describes(entry)
            {
                // A default that is itself missing leaves Word's own
                // Calibri (01177cdf's theme Museo Sans).
                match table.default_family() {
                    Some(default)
                        if via_default.is_none() && !default.eq_ignore_ascii_case(primary) =>
                    {
                        current = default;
                        via_default = Some(default);
                        continue;
                    }
                    _ => {
                        break (
                            Self::face_from_physical("Calibri", bold, italic),
                            FontStep::Generic,
                        );
                    }
                }
            }
            break (
                Self::face_from_physical(&super::word_subst::unknown_physical(), bold, italic),
                FontStep::Unknown,
            );
        };
        (
            id,
            if via_alt { FontStep::AltName } else { step },
            via_default,
        )
    }

    /// `family` names an installed catalogue face directly (the report's
    /// `explicit` step, no altName / substitution / generic hop).
    pub(crate) fn is_installed_family(family: &str) -> bool {
        let primary = family_token(family);
        if primary.starts_with('"') || primary.starts_with('\'') {
            return false;
        }
        let key = primary
            .to_ascii_lowercase()
            .replace([' ', '-'], "")
            .replace("mt", "");
        Self::mapped_face(&key, false, false)
            .is_some_and(|id| Self::catalogue_step(id) == FontStep::Explicit)
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

impl Fonts<'_> {
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

/// `system_override_uncached`, remembered for the process: font
/// resolution asks for every run, and each ask was up to eight `stat`s
/// (a fifth of a conversion's samples).
fn system_override(id: FaceId) -> Option<PathBuf> {
    static CACHE: LazyLock<Mutex<HashMap<FaceId, Option<PathBuf>>>> =
        LazyLock::new(|| Mutex::new(HashMap::new()));
    if let Some(hit) = CACHE.lock().ok().and_then(|c| c.get(&id).cloned()) {
        return hit;
    }
    let found = system_override_uncached(id);
    if let Ok(mut cache) = CACHE.lock() {
        cache.insert(id, found.clone());
    }
    found
}

fn system_override_uncached(id: FaceId) -> Option<PathBuf> {
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
    // Times New Roman: Word draws the installed macOS face (5.01, hhea
    // lineGap 87 → 13.8pt at 12) over its private DFonts copy (7.0,
    // lineGap 0 → 13.29); fixtures_500 014babb2 double lines are 27.6pt.
    const INSTALLED_FIRST: &[&str] = &[
        "/System/Library/Fonts/Supplemental",
        "/Library/Fonts",
        "/Applications/Microsoft Word.app/Contents/Resources/DFonts",
        "/Library/Fonts/Microsoft",
    ];
    let dirs = match id {
        FaceId::Symbol => WORD_DIRS,
        FaceId::SerifRegular
        | FaceId::SerifBold
        | FaceId::SerifItalic
        | FaceId::SerifBoldItalic => INSTALLED_FIRST,
        _ => DIRS,
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

/// Installed faces for a family the catalogue has no slot for, keyed by
/// (bold, italic). Word draws these with the real file (fixtures_500:
/// Tahoma in 158 documents, Segoe UI from Word's cloud-font cache,
/// Century Gothic in DFonts); we painted them as Arial or Calibri.
/// Candidates are files whose normalised name starts with the family's,
/// confirmed against the font's own family name.
/// Word's cloud-font folder for `family`, when the name is one plain path
/// component: `w:name` is document data, and "../.." or an absolute name
/// would read fonts from anywhere (PR #167 review).
fn cloud_font_dir(home: &Path, family: &str) -> Option<PathBuf> {
    let mut parts = Path::new(family).components();
    match (parts.next(), parts.next()) {
        (Some(std::path::Component::Normal(_)), None) => Some(
            home.join("Library/Group Containers/UBF8T346G9.Office/FontCache/4/CloudFonts")
                .join(family),
        ),
        _ => None,
    }
}

/// A font folder's entries, sorted, listed once per process: every
/// family lookup walks the same system folders (hundreds of entries).
fn sorted_dir_listing(dir: &Path) -> Arc<Vec<PathBuf>> {
    static CACHE: LazyLock<Mutex<HashMap<PathBuf, Arc<Vec<PathBuf>>>>> =
        LazyLock::new(|| Mutex::new(HashMap::new()));
    if let Some(hit) = CACHE.lock().ok().and_then(|c| c.get(dir).cloned()) {
        return hit;
    }
    let mut paths: Vec<PathBuf> = fs::read_dir(dir)
        .map(|entries| entries.flatten().map(|e| e.path()).collect())
        .unwrap_or_default();
    paths.sort();
    let paths = Arc::new(paths);
    if let Ok(mut cache) = CACHE.lock() {
        cache.insert(dir.to_path_buf(), Arc::clone(&paths));
    }
    paths
}

pub(crate) fn installed_family_faces(family: &str) -> Vec<((bool, bool), Vec<u8>)> {
    let stems = cjk_file_stems(family);
    if !stems.is_empty() {
        return cjk_family_faces(family, stems);
    }
    const DIRS: &[&str] = &[
        "/System/Library/Fonts/Supplemental",
        "/Library/Fonts",
        "/Applications/Microsoft Word.app/Contents/Resources/DFonts",
        "/Library/Fonts/Microsoft",
        "/System/Library/Fonts",
    ];
    let norm = |s: &str| -> String {
        s.chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .map(|c| c.to_ascii_lowercase())
            .collect()
    };
    // A name with no Latin key (华文仿宋) can only match a cloud folder by
    // the family names inside its fonts; file names say nothing about it.
    let latin = norm(family).len() >= 3;
    // (dir, whole folder is the family): Word's cloud-font cache keeps each
    // family in its own folder under numeric file names (Poppins/2397….ttf).
    let mut dirs: Vec<(PathBuf, bool)> = if latin {
        DIRS.iter().map(|d| (PathBuf::from(d), false)).collect()
    } else {
        Vec::new()
    };
    dirs.extend(cloud_font_dirs(family));
    if dirs.is_empty() {
        return Vec::new();
    }
    faces_with_user_fonts(family, &dirs, user_font_dir().as_deref())
}

/// Word's cloud-font folders that may hold `family`, each a whole-family
/// folder: its own or, when it has none, any whose name starts it. Word files a style
/// family under its parent (fixtures_500 001d945a: "Script MT Bold" in
/// "Script MT/", 01838a08: "Roboto Condensed" in "Roboto/"); the faces
/// still answer only to their own family name. Parent names come from the
/// cache's own listing, never from the document.
fn cloud_font_dirs(family: &str) -> Vec<(PathBuf, bool)> {
    let Some(home) = std::env::var_os("HOME") else {
        return Vec::new();
    };
    let home = Path::new(&home);
    let own = cloud_font_dir(home, family);
    if let Some(dir) = own.as_ref().filter(|d| d.is_dir()) {
        return vec![(dir.clone(), true)];
    }
    let mut out: Vec<(PathBuf, bool)> = own.map(|dir| (dir, true)).into_iter().collect();
    let Some(root) = cloud_font_dir(home, "x").and_then(|d| d.parent().map(Path::to_path_buf))
    else {
        return out;
    };
    let want = fold_family(family);
    for dir in sorted_dir_listing(&root).iter() {
        let Some(name) = dir.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let parent = fold_family(name);
        if parent.len() >= 3 && parent.len() < want.len() && want.starts_with(&parent) {
            out.push((dir.clone(), true));
        }
    }
    let latin = want.chars().filter(char::is_ascii_alphanumeric).count() >= 3;
    if !latin && out.iter().all(|(d, _)| !d.is_dir()) {
        // Word files a family under its English name and answers to its
        // localized one too: 华文仿宋 lives in CloudFonts/STFangsong/, whose
        // name table carries 华文仿宋 for zh-CN (fixtures_500 004599833e).
        out.extend(
            cloud_folder_names(&root)
                .iter()
                .filter(|(_, names)| names.contains(&want))
                .map(|(dir, _)| (dir.clone(), true)),
        );
    }
    out
}

/// Cloud-font folders, each with the folded family names its fonts carry.
type FolderNames = Vec<(PathBuf, Vec<String>)>;

/// Each cloud-font folder with the family names (IDs 1 and 16, every
/// language, folded) of its first font, read once per process: a folder
/// holds one family, and the cache is ~60 MB, too much to read whole.
fn cloud_folder_names(root: &Path) -> Arc<FolderNames> {
    static CACHE: LazyLock<Mutex<HashMap<PathBuf, Arc<FolderNames>>>> =
        LazyLock::new(|| Mutex::new(HashMap::new()));
    if let Some(hit) = CACHE.lock().ok().and_then(|c| c.get(root).cloned()) {
        return hit;
    }
    let mut out = Vec::new();
    for dir in sorted_dir_listing(root).iter().filter(|d| d.is_dir()) {
        let mut names = Vec::new();
        let first = sorted_dir_listing(dir)
            .iter()
            .find_map(|file| fs::read(file).ok());
        if let Some(bytes) = first
            && let Ok(face) = ttf_parser::Face::parse(&bytes, 0)
        {
            names.extend(face_family_names(&face, ttf_parser::name_id::FAMILY));
            names.extend(face_family_names(
                &face,
                ttf_parser::name_id::TYPOGRAPHIC_FAMILY,
            ));
        }
        names.sort();
        names.dedup();
        out.push((dir.clone(), names));
    }
    let out = Arc::new(out);
    if let Ok(mut cache) = CACHE.lock() {
        cache.insert(root.to_path_buf(), Arc::clone(&out));
    }
    out
}

/// `family`'s faces from `dirs`, then from jubarte's own font folder
/// (`scripts/install.sh` fills it), then an open stand-in from that folder
/// when the real face is nowhere (Selawik for Segoe UI).
fn faces_with_user_fonts(
    family: &str,
    dirs: &[(PathBuf, bool)],
    user: Option<&Path>,
) -> Vec<((bool, bool), Vec<u8>)> {
    let mut all = dirs.to_vec();
    if let Some(user) = user {
        all.push((user.to_path_buf(), false));
    }
    let found = family_faces_in(family, &all);
    if !found.is_empty() {
        return found;
    }
    match (open_stand_in(family), user) {
        (Some(stand_in), Some(user)) => family_faces_in(stand_in, &[(user.to_path_buf(), false)]),
        _ => Vec::new(),
    }
}

/// The open family that stands in for a Microsoft one jubarte cannot ship:
/// Selawik is Microsoft's own metric-compatible Segoe UI substitute (with
/// Word's cloud cache hidden, 010ec7df 0.442 -> 0.447 and 015beda9 gets
/// Word's 4 pages). EB Garamond for Garamond was tried and dropped: its
/// metrics are further from Monotype's than the Times fallback (00dd36c7
/// 0.181 -> 0.079); Word's own GARA*.ttf is now found by family name. Cooper Black and Script MT have no open equivalent.
fn open_stand_in(family: &str) -> Option<&'static str> {
    (fold_family(family) == "segoeui").then_some("Selawik")
}

/// `family`'s faces in `dirs` ((folder, whole folder is the family)).
fn family_faces_in(family: &str, dirs: &[(PathBuf, bool)]) -> Vec<((bool, bool), Vec<u8>)> {
    let norm = |s: &str| -> String {
        s.chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .map(|c| c.to_ascii_lowercase())
            .collect()
    };
    let key = norm(family);
    // Without a Latin key only a whole-family folder can answer; its faces
    // still have to carry the requested name.
    let latin = key.len() >= 3;
    if !latin && dirs.iter().all(|(_, family_folder)| !family_folder) {
        return Vec::new();
    }
    let mut found: Vec<(u8, (bool, bool), Vec<u8>)> = Vec::new();
    // Faces from collections rank after single-face files: Word draws its
    // own DFonts Rockwell (hhea 1.174em) over macOS's Rockwell.ttc (1.0em
    // plus a gap), fixtures_500 0071d504's 15.6pt sidebar lines.
    let mut collected: Vec<(u8, (bool, bool), Vec<u8>)> = Vec::new();
    for (dir, family_folder) in dirs {
        let family_folder = *family_folder;
        for path in sorted_dir_listing(dir).iter() {
            let path = path.clone();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let is_font = ext.eq_ignore_ascii_case("ttf") || ext.eq_ignore_ascii_case("otf");
            let stem = path.file_stem().and_then(|s| s.to_str()).map(norm);
            // A collection is named for its whole family (Avenir.ttc holds
            // "Avenir Book"); its faces answer to their own names.
            if ext.eq_ignore_ascii_case("ttc")
                && latin
                && stem
                    .as_deref()
                    .is_some_and(|s| s.len() >= 3 && key.starts_with(s))
            {
                let Ok(bytes) = fs::read(&path) else {
                    continue;
                };
                let count = ttf_parser::fonts_in_collection(&bytes).unwrap_or(0);
                for index in 0..count {
                    let Some(data) = ttc_face_bytes(&bytes, index) else {
                        continue;
                    };
                    if let Some((pass, style)) = face_family_style(&data, family) {
                        collected.push((pass, style, data));
                    }
                }
                continue;
            }
            // A system file may be named short of its family ("Arial
            // Unicode.ttf" holds "Arial Unicode MS", which Word draws in
            // international_terrorism_thesis's header); its own name decides.
            // Word's DFonts keep the name-index path below: Word paints
            // 012128d3's regular TH SarabunPSK runs in its thsarabun-bold.
            let short_ok = !dir.ends_with("DFonts");
            let named = latin
                && stem.is_some_and(|s| {
                    s.starts_with(&key) || (short_ok && s.len() >= 5 && key.starts_with(&s))
                });
            if !is_font || !(family_folder || named) {
                continue;
            }
            let Ok(bytes) = fs::read(&path) else {
                continue;
            };
            let Some((pass, style)) = face_family_style(&bytes, family) else {
                continue;
            };
            found.push((pass, style, bytes));
        }
    }
    found.append(&mut collected);
    if found.is_empty() && latin {
        // Word's own fonts carry abbreviated file names (Garamond is
        // GARA.ttf / GARAIT.ttf): match its folder by internal family name
        // (fixtures_500 00dd36c7 painted Garamond Italic as Times).
        for (dir, _) in dirs.iter().filter(|(d, _)| d.ends_with("DFonts")) {
            for (path, names) in font_name_index(dir).iter() {
                if !names.contains(&key) {
                    continue;
                }
                let Ok(bytes) = fs::read(path) else {
                    continue;
                };
                if let Some((pass, style)) = face_family_style(&bytes, family) {
                    found.push((pass, style, bytes));
                }
            }
        }
    }
    pick_ranked_faces(found)
}

/// Font files with their normalised family names.
type FontNameIndex = Arc<Vec<(PathBuf, Vec<String>)>>;

/// Each `.ttf`/`.otf` in `dir` with its normalised family names (name IDs
/// 1 and 16), read once per process.
fn font_name_index(dir: &Path) -> FontNameIndex {
    static CACHE: LazyLock<Mutex<HashMap<PathBuf, FontNameIndex>>> =
        LazyLock::new(|| Mutex::new(HashMap::new()));
    if let Some(hit) = CACHE.lock().ok().and_then(|c| c.get(dir).cloned()) {
        return hit;
    }
    let norm = |s: &str| -> String {
        s.chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .map(|c| c.to_ascii_lowercase())
            .collect()
    };
    let mut out = Vec::new();
    for path in sorted_dir_listing(dir).iter() {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !(ext.eq_ignore_ascii_case("ttf") || ext.eq_ignore_ascii_case("otf")) {
            continue;
        }
        let Ok(bytes) = fs::read(path) else {
            continue;
        };
        let Ok(face) = ttf_parser::Face::parse(&bytes, 0) else {
            continue;
        };
        let names: Vec<String> = face
            .names()
            .into_iter()
            .filter(|n| matches!(n.name_id, 1 | 16) && n.is_unicode())
            .filter_map(|n| n.to_string())
            .map(|n| norm(&n))
            .collect();
        out.push((path.clone(), names));
    }
    let out = Arc::new(out);
    if let Ok(mut cache) = CACHE.lock() {
        cache.insert(dir.to_path_buf(), Arc::clone(&out));
    }
    out
}

/// jubarte's own font folder: `$JUBARTE_FONT_DIR`, else the platform's
/// per-user data folder (`~/Library/Application Support/jubarte/fonts`,
/// `$XDG_DATA_HOME/jubarte/fonts` or `~/.local/share/jubarte/fonts`,
/// `%APPDATA%\jubarte\fonts`). Fonts live there, not in the binary.
pub(crate) fn user_font_dir() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os("JUBARTE_FONT_DIR").filter(|d| !d.is_empty()) {
        return Some(PathBuf::from(dir));
    }
    if cfg!(target_os = "windows") {
        return std::env::var_os("APPDATA").map(|a| PathBuf::from(a).join("jubarte").join("fonts"));
    }
    let home = PathBuf::from(std::env::var_os("HOME")?);
    let data = if cfg!(target_os = "macos") {
        home.join("Library/Application Support")
    } else {
        std::env::var_os("XDG_DATA_HOME")
            .filter(|d| !d.is_empty())
            .map_or_else(|| home.join(".local/share"), PathBuf::from)
    };
    Some(data.join("jubarte").join("fonts"))
}

/// A family name folded for comparison: full-width Latin to ASCII (the
/// Japanese "ＭＳ 明朝" is MS Mincho's own name), ASCII lowercase, and no
/// spaces, underscores or hyphens.
fn fold_family(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '\u{FF01}'..='\u{FF5E}' => char::from_u32(c as u32 - 0xFEE0).unwrap_or(c),
            _ => c,
        })
        .filter(|c| !c.is_whitespace() && *c != '_' && *c != '-')
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

/// East Asian families Word ships in its DFonts, by every name documents
/// use for them, and the file stems that hold them. Word draws these real
/// faces (fixtures_500: MS Mincho in 21 documents, YaHei 16, JhengHei 12,
/// Yu Gothic 11); we had no face for them and their text vanished.
const CJK_FAMILIES: &[(&[&str], &[&str])] = &[
    (
        &["msmincho", "ms明朝", "mspmincho", "msp明朝"],
        &["msmincho"],
    ),
    (
        &[
            "msgothic",
            "msゴシック",
            "mspgothic",
            "mspゴシック",
            "msuigothic",
        ],
        &["msgothic"],
    ),
    (
        &[
            "yugothic",
            "游ゴシック",
            "yugothicui",
            "yugothicmedium",
            "yugothiclight",
            "游ゴシックmedium",
            "游ゴシックlight",
        ],
        &["yugothr", "yugothm", "yugothb", "yugothl"],
    ),
    (
        &[
            "yumincho",
            "游明朝",
            "yuminchodemibold",
            "yumincholight",
            "游明朝demibold",
            "游明朝light",
        ],
        &["yumin", "yumindb", "yuminl"],
    ),
    (&["meiryo", "メイリオ", "meiryoui"], &["meiryo", "meiryob"]),
    (
        &[
            "microsoftyahei",
            "微软雅黑",
            "microsoftyaheiui",
            "microsoftyaheilight",
        ],
        &["msyh", "msyhbd", "msyhl"],
    ),
    (
        &["microsoftjhenghei", "微軟正黑體", "microsoftjhengheiui"],
        &["msjh", "msjhbd"],
    ),
    (&["simsun", "宋体", "nsimsun", "新宋体"], &["simsun"]),
    (&["simhei", "黑体"], &["simhei"]),
    (
        &["fangsong", "仿宋", "fangsonggb2312", "仿宋gb2312"],
        &["fangsong"],
    ),
    (&["kaiti", "楷体", "kaitigb2312", "楷体gb2312"], &["kaiti"]),
    (
        &["mingliu", "細明體", "pmingliu", "新細明體", "mingliuhkscs"],
        &["mingliu", "mingliub"],
    ),
    (
        &["batang", "바탕", "batangche", "gungsuh", "궁서"],
        &["batang"],
    ),
    (
        &["gulim", "굴림", "gulimche", "dotum", "돋움", "dotumche"],
        &["gulim"],
    ),
    (&["malgungothic", "맑은고딕"], &["malgun", "malgunbd"]),
    (
        &["dengxian", "等线", "dengxianlight"],
        &["deng", "dengb", "dengl"],
    ),
    (
        &[
            "hg創英角ｺﾞｼｯｸub",
            "hgp創英角ｺﾞｼｯｸub",
            "hgs創英角ｺﾞｼｯｸub",
            "hgsoeikakugothicub",
            "hgpsoeikakugothicub",
            "hgssoeikakugothicub",
        ],
        &["hgrsgu"],
    ),
    (
        &[
            "hgｺﾞｼｯｸe",
            "hgpｺﾞｼｯｸe",
            "hgsｺﾞｼｯｸe",
            "hggothice",
            "hgpgothice",
            "hgsgothice",
        ],
        &["hgrge"],
    ),
    (
        &[
            "hg明朝e",
            "hgp明朝e",
            "hgs明朝e",
            "hgminchoe",
            "hgpminchoe",
            "hgsminchoe",
        ],
        &["hgrme"],
    ),
];

fn cjk_file_stems(family: &str) -> &'static [&'static str] {
    let key = fold_family(family);
    CJK_FAMILIES
        .iter()
        .find(|(names, _)| names.contains(&key.as_str()))
        .map_or(&[], |(_, stems)| stems)
}

/// Family names (name IDs `id`) of one face, folded, in every language.
fn face_family_names(face: &ttf_parser::Face<'_>, id: u16) -> Vec<String> {
    face.names()
        .into_iter()
        .filter(|n| n.name_id == id)
        .filter_map(|n| name_text(&n))
        .map(|n| fold_family(&n))
        .collect()
}

/// The faces of an East Asian family from Word's DFonts. A collection
/// (`.ttc`) holds several families (MS Mincho / MS PMincho); the face whose
/// own family name is the requested one wins, name ID 1 before the
/// typographic ID 16 (Yu Gothic Medium is ID 1 "Yu Gothic Medium"). A name
/// no face carries (FangSong_GB2312) takes the group's first face.
fn cjk_family_faces(family: &str, stems: &[&str]) -> Vec<((bool, bool), Vec<u8>)> {
    const DIRS: &[&str] = &[
        "/Applications/Microsoft Word.app/Contents/Resources/DFonts",
        "/Library/Fonts/Microsoft",
        "/Library/Fonts",
    ];
    let want = fold_family(family);
    let mut files: Vec<(usize, PathBuf)> = Vec::new();
    for dir in DIRS {
        let Ok(entries) = fs::read_dir(dir) else {
            continue;
        };
        for path in entries.flatten().map(|e| e.path()) {
            let ext_ok = path.extension().and_then(|e| e.to_str()).is_some_and(|e| {
                ["ttf", "otf", "ttc"]
                    .iter()
                    .any(|x| e.eq_ignore_ascii_case(x))
            });
            let stem = path.file_stem().and_then(|s| s.to_str()).map(fold_family);
            if let (true, Some(stem)) = (ext_ok, stem)
                && let Some(rank) = stems.iter().position(|s| *s == stem)
                && files.iter().all(|(_, p)| p.file_name() != path.file_name())
            {
                files.push((rank, path));
            }
        }
    }
    files.sort();
    // (pass, style, bytes): pass 0 = ID 1 match, 1 = ID 16, 2 = fallback.
    let mut found: Vec<(u8, (bool, bool), Vec<u8>)> = Vec::new();
    for (rank, path) in &files {
        let Ok(bytes) = fs::read(path) else {
            continue;
        };
        let count = ttf_parser::fonts_in_collection(&bytes).unwrap_or(1);
        for index in 0..count {
            let Ok(face) = ttf_parser::Face::parse(&bytes, index) else {
                continue;
            };
            if face.tables().glyf.is_none() {
                continue;
            }
            let pass = if face_family_names(&face, ttf_parser::name_id::FAMILY).contains(&want) {
                0
            } else if face_family_names(&face, ttf_parser::name_id::TYPOGRAPHIC_FAMILY)
                .contains(&want)
            {
                1
            } else if *rank == 0 && index == 0 {
                2
            } else {
                continue;
            };
            let style = (face.is_bold(), face.is_italic());
            let data = if count > 1 {
                match ttc_face_bytes(&bytes, index) {
                    Some(data) => data,
                    None => continue,
                }
            } else {
                bytes.clone()
            };
            found.push((pass, style, data));
        }
    }
    pick_ranked_faces(found)
}

/// One face of a TrueType collection as a standalone sfnt: the face's
/// table directory with its tables copied after it (PDF `FontFile2`
/// cannot hold a collection).
fn ttc_face_bytes(ttc: &[u8], index: u32) -> Option<Vec<u8>> {
    let u32_at = |at: usize| -> Option<u32> {
        ttc.get(at..at + 4)
            .map(|b| u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    };
    if ttc.get(0..4)? != b"ttcf" {
        return None;
    }
    let dir = usize::try_from(u32_at(12 + 4 * usize::try_from(index).ok()?)?).ok()?;
    let num_tables = usize::from(u16::from_be_bytes([*ttc.get(dir + 4)?, *ttc.get(dir + 5)?]));
    let header_len = 12 + 16 * num_tables;
    let mut out = ttc.get(dir..dir + 12)?.to_vec();
    let mut records = Vec::with_capacity(16 * num_tables);
    let mut data: Vec<u8> = Vec::new();
    for t in 0..num_tables {
        let rec = dir + 12 + 16 * t;
        let offset = usize::try_from(u32_at(rec + 8)?).ok()?;
        let length = usize::try_from(u32_at(rec + 12)?).ok()?;
        let new_offset = u32::try_from(header_len + data.len()).ok()?;
        records.extend_from_slice(ttc.get(rec..rec + 8)?);
        records.extend_from_slice(&new_offset.to_be_bytes());
        records.extend_from_slice(ttc.get(rec + 12..rec + 16)?);
        data.extend_from_slice(ttc.get(offset..offset + length)?);
        while !data.len().is_multiple_of(4) {
            data.push(0);
        }
    }
    out.extend_from_slice(&records);
    out.extend_from_slice(&data);
    Some(out)
}

/// Embedded-map keys of the East Asian fallback faces (see
/// `Fonts::cjk_fallback_index`); no document family can be named this.
pub(crate) const CJK_FALLBACK: &str = "@cjk";
pub(crate) const CJK_FALLBACK_JA: &str = "@cjk-ja";
/// Embedded-map key of the Thaana fallback face.
pub(crate) const THAANA_FALLBACK: &str = "@thaana";

/// Loads the face Word paints Dhivehi in when the document's font is
/// missing: MV Boli from Office's cloud fonts (fixtures_500 0003fc93's
/// Faruma), else the system's Noto Sans Thaana.
pub(crate) fn add_thaana_fallback(embedded: &mut EmbeddedFonts) {
    let faces = cached_faces(THAANA_FALLBACK, || {
        let faces = installed_family_faces("MV Boli");
        if faces.is_empty() {
            installed_family_faces("Noto Sans Thaana")
        } else {
            faces
        }
    });
    for ((bold, italic), bytes) in faces {
        embedded.insert((THAANA_FALLBACK.to_string(), bold, italic), bytes);
    }
}

fn is_cjk_name_char(c: char) -> bool {
    matches!(c as u32, 0x2E80..=0x9FFF | 0xAC00..=0xD7AF | 0xF900..=0xFAFF | 0xFF66..=0xFF9F)
}

type SharedFaces = Vec<((bool, bool), Arc<[u8]>)>;

/// Installed-face lookups, cached for the process: a long-lived caller
/// (the Python and WASM bindings) would otherwise re-read 10-20 MB font
/// collections for every conversion.
fn cached_faces(key: &str, load: impl FnOnce() -> Vec<((bool, bool), Vec<u8>)>) -> SharedFaces {
    static CACHE: LazyLock<Mutex<HashMap<String, SharedFaces>>> =
        LazyLock::new(|| Mutex::new(HashMap::new()));
    if let Ok(cache) = CACHE.lock()
        && let Some(faces) = cache.get(key)
    {
        return faces.clone();
    }
    let faces: SharedFaces = load()
        .into_iter()
        .map(|(style, bytes)| (style, Arc::from(bytes)))
        .collect();
    if let Ok(mut cache) = CACHE.lock() {
        cache.insert(key.to_string(), faces.clone());
    }
    faces
}

/// Loads Word's East Asian fallback faces (YaHei, Yu Gothic) for a
/// document that has East Asian text.
pub(crate) fn add_cjk_fallbacks(embedded: &mut EmbeddedFonts) {
    for (key, family, stems) in [
        (CJK_FALLBACK, "Microsoft YaHei", &["msyh", "msyhbd"][..]),
        (CJK_FALLBACK_JA, "Yu Gothic", &["yugothr", "yugothb"][..]),
    ] {
        for ((bold, italic), bytes) in cached_faces(key, || cjk_family_faces(family, stems)) {
            embedded.insert((key.to_string(), bold, italic), bytes);
        }
    }
}

/// Adds the installed faces of every font-table family that the catalogue
/// does not cover and the document does not embed.
///
/// `extra` are the families styles and the theme name; `run_faces` those
/// runs paint non-East-Asian text in (ascii / hAnsi / cs, theme latin).
/// A theme lists ~45 script fonts (Mangal, Sylfaen, 游明朝…): loading each
/// from disk was a third of every conversion, so an East Asian family
/// loads only for East Asian text or a run that paints in it, and any
/// other family only when a run paints in it.
pub(crate) fn add_installed_faces(
    embedded: &mut EmbeddedFonts,
    table: &super::font_table::FontTable,
    extra: &[String],
    run_faces: &[String],
    has_cjk: bool,
) {
    // East Asian families named only in styles or the theme ("宋体" as the
    // theme's Hans font) are not in the font table but Word draws them.
    for name in extra {
        let lower = name.to_ascii_lowercase();
        let painted = run_faces.iter().any(|n| n == name);
        let wanted = if cjk_file_stems(name).is_empty() {
            painted && !catalogue_paints_family(name)
        } else {
            has_cjk || painted
        };
        if !wanted || table.get(name).is_some() || embedded.keys().any(|(f, _, _)| *f == lower) {
            continue;
        }
        // A family runs name but the table omits (015beda9 has no table
        // part; Word still draws its Segoe UI) loads like a table family.
        let faces = cached_faces(name, || installed_family_faces(name));
        for ((bold, italic), bytes) in faces {
            embedded.insert((lower.clone(), bold, italic), bytes);
        }
    }
    for entry in table.iter() {
        let lower = entry.name.to_ascii_lowercase();
        if catalogue_paints_family(&entry.name) || embedded.keys().any(|(f, _, _)| *f == lower) {
            continue;
        }
        let faces = cached_faces(&entry.name, || installed_family_faces(&entry.name));
        // Runs may name the family by its altName ("MS Mincho" for the
        // table's "ＭＳ 明朝"); the same faces answer to both.
        if let Some(alt) = entry.alt_name.as_deref()
            && !cjk_file_stems(&entry.name).is_empty()
        {
            let alt = alt.to_ascii_lowercase();
            if embedded.keys().all(|(f, _, _)| *f != alt) {
                for ((bold, italic), bytes) in &faces {
                    embedded.insert((alt.clone(), *bold, *italic), Arc::clone(bytes));
                }
            }
        }
        for ((bold, italic), bytes) in faces {
            embedded.insert((lower.clone(), bold, italic), bytes);
        }
    }
}

/// The catalogue slot for `family` really is that family, not a stand-in
/// (Tahoma, Trebuchet and Roboto all fold into the Arial slot).
fn catalogue_paints_family(family: &str) -> bool {
    let norm = |s: &str| -> String {
        s.chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .map(|c| c.to_ascii_lowercase())
            .collect()
    };
    let key = FaceKey {
        family: family.to_ascii_lowercase(),
        bold: false,
        italic: false,
    };
    let face = catalogue().get(Fonts::id_from_key(&key));
    // A name with no ASCII letters ("ＭＳ 明朝") folds to "", which every
    // catalogue name starts with; no catalogue slot is that family.
    let want = norm(family);
    !want.is_empty() && norm(face.pdf_name()).starts_with(&want)
}

/// (bold, italic) when the font's own family name is `family`.
/// One face per (bold, italic): the best-ranked candidate (lower pass),
/// path order breaking ties. A typographic-family (ID 16) match never
/// takes a style an ID 1 match fills (Roboto Black vs Roboto Regular).
fn pick_ranked_faces(mut found: Vec<(u8, (bool, bool), Vec<u8>)>) -> Vec<((bool, bool), Vec<u8>)> {
    found.sort_by_key(|(pass, _, _)| *pass);
    let mut out: Vec<((bool, bool), Vec<u8>)> = Vec::new();
    for (_, style, bytes) in found {
        if out.iter().all(|(s, _)| *s != style) {
            out.push((style, bytes));
        }
    }
    out
}

/// (pass, (bold, italic)) when the font's own family name is `family`:
/// pass 0 for name ID 1, 4 for the typographic ID 16 only, plus 2 for a
/// face that is not normal width and 1 for one off its style's weight. A face without
/// TrueType outlines is skipped: PDF FontFile2 cannot carry CFF.
/// OS/2 `ulCodePageRange1` names a Japanese, Chinese or Korean code page
/// (bits 17-21).
fn cjk_code_pages(face: &ttf_parser::Face) -> bool {
    face.raw_face()
        .table(ttf_parser::Tag::from_bytes(b"OS/2"))
        .and_then(|os2| os2.get(78..82))
        .map(|b| u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
        .is_some_and(|range| range & (0b1_1111 << 17) != 0)
}

/// A name record's text: Unicode records as ttf-parser decodes them, and
/// Macintosh Roman ones when plain ASCII. Apple's Futura.ttc names its
/// family only in a Mac Roman record, which `to_string` leaves undecoded.
fn name_text(n: &ttf_parser::name::Name) -> Option<String> {
    n.to_string().or_else(|| {
        (n.platform_id == ttf_parser::PlatformId::Macintosh
            && n.encoding_id == 0
            && n.name.is_ascii())
        .then(|| n.name.iter().map(|&b| char::from(b)).collect())
    })
}

fn face_family_style(bytes: &[u8], family: &str) -> Option<(u8, (bool, bool))> {
    let face = ttf_parser::Face::parse(bytes, 0).ok()?;
    face.tables().glyf?;
    let has = |id: u16| {
        face.names().into_iter().any(|n| {
            n.name_id == id && name_text(&n).is_some_and(|f| f.eq_ignore_ascii_case(family))
        })
    };
    let named = if has(ttf_parser::name_id::FAMILY) {
        0
    } else if has(ttf_parser::name_id::TYPOGRAPHIC_FAMILY) {
        4
    } else {
        return None;
    };
    // A normal-width face ranks before a condensed or expanded one of the
    // same style (Papyrus.ttc holds Condensed ahead of Regular; Word draws
    // the Regular, English corpus 469e5710), and a face at its style's
    // weight before a SemiBold or Light one (20c18b4b's Open Sans).
    let target = if face.is_bold() { 700 } else { 400 };
    let pass = named
        + 2 * u8::from(face.width() != ttf_parser::Width::Normal)
        + u8::from(face.weight().to_number().abs_diff(target) > 50);
    // A slant alone is not italic: MV Boli leans -16 degrees but is the
    // Regular face. Apple's Avenir Next Bold Italic sets no italic bit,
    // so a slanted face whose subfamily says so still counts.
    let slanted_name = face.names().into_iter().any(|n| {
        matches!(n.name_id, 2 | 17)
            && name_text(&n).is_some_and(|f| {
                let f = f.to_ascii_lowercase();
                f.contains("italic") || f.contains("oblique")
            })
    });
    let italic = face.style() != ttf_parser::Style::Normal
        || (face.italic_angle() != 0.0 && (slanted_name || face.tables().os2.is_none()));
    Some((pass, (face.is_bold(), italic)))
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
    let records: Vec<_> = name
        .names
        .into_iter()
        .filter(|n| n.name_id == ttf_parser::name_id::POST_SCRIPT_NAME)
        .collect();
    records
        .iter()
        .find_map(|n| if n.is_unicode() { n.to_string() } else { None })
        .or_else(|| records.iter().find_map(|n| ascii_record_name(n.name)))
}

/// Name ID 6 is printable ASCII by definition (OpenType `name`), so a
/// Macintosh-Roman or other 8-bit record decodes byte for byte.
fn ascii_record_name(raw: &[u8]) -> Option<String> {
    (!raw.is_empty() && raw.iter().all(|b| (0x21..=0x7e).contains(b)))
        .then(|| String::from_utf8_lossy(raw).into_owned())
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

/// A font-table entry Word substitutes by its generic: one with a panose
/// that says something beyond its family kind (`0200…0` and all-zero say
/// nothing) or with an altName.
fn entry_describes(entry: &super::font_table::FontEntry) -> bool {
    entry.alt_name.is_some()
        || entry
            .panose
            .is_some_and(|p| p.iter().skip(1).any(|b| *b != 0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_east_asian_face_takes_word_s_taller_line() {
        // Live Word at 12pt: SimSun lines step 15.6 (1.3 x its 1.0 em
        // hhea body), not the hhea 13.7 with its lineGap; the extra splits
        // around the text (baseline 84.2 under a 72pt margin).
        let path =
            Path::new("/Applications/Microsoft Word.app/Contents/Resources/DFonts/Simsun.ttc");
        let Ok(bytes) = fs::read(path) else {
            return;
        };
        let bytes: &'static [u8] = Box::leak(bytes.into_boxed_slice());
        let face = Face::from_bytes(FaceId::SansRegular, bytes, "SimSun".into()).expect("SimSun");
        assert!(
            (face.single_line_pt(12.0) - 15.6).abs() < 0.05,
            "{}",
            face.single_line_pt(12.0)
        );
        assert!(
            (face.ascent_pt(12.0) - 12.11).abs() < 0.1,
            "{}",
            face.ascent_pt(12.0)
        );
    }

    #[test]
    fn a_slanted_regular_face_is_the_upright_one() {
        // MV Boli (Word's Thaana face) leans -16 degrees yet is the family's
        // Regular: Word files it by its style bits, not the angle.
        let Some(home) = std::env::var_os("HOME") else {
            return;
        };
        let path = Path::new(&home).join(
            "Library/Group Containers/UBF8T346G9.Office/FontCache/4/CloudFonts/MV Boli/29162370688.ttf",
        );
        let Ok(bytes) = fs::read(&path) else {
            return;
        };
        assert_eq!(
            face_family_style(&bytes, "MV Boli"),
            Some((0, (false, false)))
        );
    }

    #[test]
    fn right_to_left_text_shapes_in_visual_order() {
        // HarfBuzz returns an RTL run left to right on the page: the last
        // letter first. Word draws "متن" with its initial meem rightmost.
        let fonts = Fonts::new();
        let face = fonts.get(FaceId::SansRegular);
        let buzz = face.buzz.as_ref().expect("bundled face shapes");
        let clusters: Vec<u32> = face
            .shaped_units(buzz, "متن", false)
            .iter()
            .map(|u| u.2)
            .collect();
        assert_eq!(clusters, vec![4, 2, 0]);
    }

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
        // The generic needs a panose that describes the face; a bare entry
        // takes the default (a_bare_missing_entry_takes_the_default…).
        let table = super::super::font_table::parse_font_table_xml(
            r#"<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
                 <w:font w:name="SomeSwiss"><w:panose1 w:val="020B0604020202020204"/><w:family w:val="swiss"/></w:font>
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
                 <w:font w:name="SomeRoman"><w:panose1 w:val="02020603050405020304"/><w:family w:val="roman"/></w:font>
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
        // A fixed-pitch entry that describes its face (a panose past the
        // family kind) still falls to Courier; a bare one does not (see
        // the next test, live Word 2026-09-25: Calibri).
        let table = super::super::font_table::parse_font_table_xml(
            r#"<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
                 <w:font w:name="SomeFixed"><w:panose1 w:val="02070309020205020404"/><w:pitch w:val="fixed"/></w:font>
               </w:fonts>"#,
        );
        let fonts = Fonts::new();
        assert_eq!(
            fonts.resolve_in("SomeFixed", false, false, &table),
            FaceId::MonoRegular
        );
    }

    #[test]
    fn a_bare_missing_entry_takes_the_default_not_its_generic() {
        // fixtures_500 01177cdf: theme "Museo Sans 300" is absent; its entry
        // says modern/variable with panose 0200…0 and no altName. Word
        // paints it in Calibri, not the modern generic Courier New (live
        // Word gives Calibri for bare modern/roman/swiss entries alike).
        let table = super::super::font_table::parse_font_table_xml(
            r#"<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
                 <w:font w:name="Museo Sans 300"><w:panose1 w:val="02000000000000000000"/>
                   <w:family w:val="modern"/><w:notTrueType/><w:pitch w:val="variable"/></w:font>
               </w:fonts>"#,
        );
        let fonts = Fonts::new();
        assert_eq!(
            fonts.resolve_in("Museo Sans 300", false, false, &table),
            FaceId::CarlitoRegular
        );
    }

    #[test]
    fn a_missing_entry_with_an_alt_name_keeps_its_generic() {
        // fixtures_500 019d9ee6: Myriad Pro (swiss, panose all zero, altName
        // an absent Segoe UI) is Arial in Word's PDF.
        let table = super::super::font_table::parse_font_table_xml(
            r#"<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
                 <w:font w:name="Myriad Pro"><w:altName w:val="Qwertzu Absent"/>
                   <w:panose1 w:val="00000000000000000000"/><w:family w:val="swiss"/>
                   <w:notTrueType/><w:pitch w:val="variable"/></w:font>
               </w:fonts>"#,
        );
        let fonts = Fonts::new();
        assert_eq!(
            fonts.resolve_in("Myriad Pro", false, false, &table),
            FaceId::SansRegular
        );
    }

    #[test]
    fn installed_faces_are_read_once_per_process() {
        // CodeRabbit #166: every conversion re-read the 10-20 MB collections.
        let first = cached_faces("@test-cache-key", || {
            vec![((false, false), b"face".to_vec())]
        });
        let second = cached_faces("@test-cache-key", || panic!("loaded twice"));
        assert!(
            Arc::ptr_eq(&first[0].1, &second[0].1),
            "one shared allocation"
        );
    }

    fn repo_font_dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/extra")
    }

    #[test]
    fn jubartes_own_font_folder_is_searched() {
        // fixtures_500 01838a08: Word embeds RobotoCondensed-Regular/-Bold
        // from its cloud cache; install.sh puts the same Apache-2.0 files in
        // jubarte's font folder instead of growing the binary.
        let faces = faces_with_user_fonts("Roboto Condensed", &[], Some(&repo_font_dir()));
        for style in [(false, false), (true, false)] {
            assert!(
                faces.iter().any(|(s, _)| *s == style),
                "Roboto Condensed {style:?} face"
            );
        }
    }

    #[test]
    fn an_open_stand_in_answers_for_an_absent_microsoft_face() {
        // Segoe UI (015beda9) is Microsoft's; without it we fell to
        // Cambria. Selawik is Microsoft's open Segoe UI stand-in.
        let faces = faces_with_user_fonts("Segoe UI", &[], Some(&repo_font_dir()));
        assert!(
            faces.iter().any(|(s, _)| *s == (false, false)),
            "a regular Selawik for Segoe UI"
        );
    }

    #[test]
    fn a_theme_script_font_no_run_paints_is_not_loaded() {
        // A theme lists ~45 per-script faces; reading each from disk was a
        // third of every conversion though no run paints in them.
        let mut embedded = EmbeddedFonts::new();
        let table = super::super::font_table::FontTable::default();
        let names = ["Roboto Condensed".to_string()];
        add_installed_faces(&mut embedded, &table, &names, &[], false);
        assert!(embedded.is_empty(), "nothing painted, nothing loaded");
    }

    #[test]
    fn a_run_family_missing_from_the_font_table_still_loads_its_faces() {
        // fixtures_500 015beda9 has no fontTable part; its runs name
        // Segoe UI, which Word draws, and only table families were loaded.
        if !Path::new("/System/Library/Fonts/HelveticaNeue.ttc").is_file() {
            return;
        }
        let mut embedded = EmbeddedFonts::new();
        let table = super::super::font_table::FontTable::default();
        let names = ["Helvetica Neue".to_string()];
        add_installed_faces(&mut embedded, &table, &names, &names, false);
        assert!(embedded.contains_key(&("helvetica neue".to_string(), false, false)));
    }

    #[test]
    fn a_cloud_face_filed_under_its_parent_family_is_found() {
        // fixtures_500 001d945a: Word embeds ScriptMTBold, which its cloud
        // cache files under "Script MT/"; we looked only in
        // "Script MT Bold/" and drew Cambria.
        let Some(home) = std::env::var_os("HOME") else {
            return;
        };
        let dir = PathBuf::from(home)
            .join("Library/Group Containers/UBF8T346G9.Office/FontCache/4/CloudFonts/Script MT");
        if !dir.is_dir() {
            return;
        }
        let faces =
            faces_with_user_fonts("Script MT Bold", &cloud_font_dirs("Script MT Bold"), None);
        assert!(
            !faces.is_empty(),
            "Script MT Bold from the Script MT folder"
        );
    }

    #[test]
    fn word_cloud_fonts_are_found_despite_numeric_file_names() {
        // fixtures_500 014b42f2 / 01635d97: Poppins and Lato live in Word's
        // cloud-font cache as <family>/<number>.ttf; the file-stem prefix
        // filter rejected them and the text fell to Cambria.
        let Some(home) = std::env::var_os("HOME") else {
            return;
        };
        let dir = PathBuf::from(home)
            .join("Library/Group Containers/UBF8T346G9.Office/FontCache/4/CloudFonts/Poppins");
        if !dir.is_dir() {
            return;
        }
        let faces = installed_family_faces("Poppins");
        assert!(
            faces.iter().any(|(style, _)| *style == (false, false)),
            "a regular Poppins face from the cloud cache"
        );
    }

    #[test]
    fn a_system_collection_face_is_found_by_its_family() {
        // fixtures_500 00d0925f: Word draws "Avenir Book" from macOS's
        // /System/Library/Fonts/Avenir.ttc (hhea 1.366em: 15.1pt lines at
        // 11pt). Only loose .ttf/.otf were searched; the text fell to Arial.
        if !Path::new("/System/Library/Fonts/Avenir.ttc").is_file() {
            return;
        }
        let faces = installed_family_faces("Avenir Book");
        assert!(
            faces.iter().any(|(style, _)| *style == (false, false)),
            "a regular Avenir Book face from the system collection"
        );
    }

    #[test]
    fn macos_helvetica_takes_word_s_one_point_two_em_line() {
        // English part a 18f71536 (12pt, line=248 auto): Word's lines step
        // 14.88, a 14.40pt single line, 1.2 em, where Helvetica's hhea body
        // is 1.0 em; the first baseline sits 0.975 em down (win ascent 0.95
        // plus the 0.025 em the win box leaves of that line).
        if !Path::new("/System/Library/Fonts/Helvetica.ttc").is_file() {
            return;
        }
        let faces = installed_family_faces("Helvetica");
        let (_, bytes) = faces
            .into_iter()
            .find(|(style, _)| *style == (false, false))
            .expect("regular Helvetica");
        let bytes: &'static [u8] = Box::leak(bytes.into_boxed_slice());
        let ps = ttf_postscript_name(bytes).expect("postscript name");
        let face = Face::from_bytes(FaceId::SansRegular, bytes, sanitize_pdf_name(&ps))
            .expect("Helvetica");
        assert!(
            (face.single_line_pt(12.0) - 14.4).abs() < 0.02,
            "{}",
            face.single_line_pt(12.0)
        );
        assert!(
            (face.ascent_pt(12.0) - 11.7).abs() < 0.02,
            "{}",
            face.ascent_pt(12.0)
        );
    }

    #[test]
    fn a_collection_whose_upright_face_is_medium_is_the_regular() {
        // English corpus 87098dc3: Word draws "Futura" from macOS's
        // Futura.ttc, whose upright face is "Futura Medium" (weight 500,
        // no Regular). The text fell to Arial.
        if !Path::new("/System/Library/Fonts/Supplemental/Futura.ttc").is_file() {
            return;
        }
        let faces = installed_family_faces("Futura");
        assert!(
            faces.iter().any(|(style, _)| *style == (false, false)),
            "a regular Futura face from the system collection"
        );
    }

    #[test]
    fn a_collection_prefers_its_normal_width_face() {
        // English corpus 469e5710: macOS's Papyrus.ttc holds "Papyrus
        // Condensed" before "Papyrus Regular", both family "Papyrus". Word
        // draws the regular; we took the condensed face.
        if !Path::new("/System/Library/Fonts/Supplemental/Papyrus.ttc").is_file() {
            return;
        }
        let faces = installed_family_faces("Papyrus");
        let regular = faces
            .iter()
            .find(|(style, _)| *style == (false, false))
            .expect("a regular Papyrus face");
        let face = ttf_parser::Face::parse(&regular.1, 0).expect("face");
        assert_eq!(face.width(), ttf_parser::Width::Normal);
    }

    #[test]
    fn a_cff_face_is_not_embedded() {
        // CodeRabbit #166: the PDF writer emits FontFile2 (TrueType); a CFF
        // .otf would be embedded as an unreadable program. It is refused.
        let otf = "/System/Library/Fonts/Supplemental/STIXSizOneSymBol.otf";
        let Ok(bytes) = fs::read(otf) else {
            return;
        };
        let parsed = ttf_parser::Face::parse(&bytes, 0).expect("STIX parses");
        assert!(parsed.tables().glyf.is_none(), "a CFF face");
        let mut fonts = Fonts::new();
        fonts.insert_embedded("stix", false, false, &bytes);
        assert!(fonts.embedded_index("stix", false, false).is_none());
    }

    #[test]
    fn a_family_name_match_outranks_a_typographic_one_per_style() {
        // CodeRabbit #166: Roboto-Black.ttf (ID 1 "Roboto Black", ID 16
        // "Roboto") sorts before Roboto-Regular.ttf and took the regular
        // slot. Pass 0 (ID 1) beats pass 1 (ID 16) per style; path order
        // decides within a pass.
        let found = vec![
            (1, (false, false), b"black".to_vec()),
            (1, (true, false), b"heavy".to_vec()),
            (0, (false, false), b"regular".to_vec()),
            (0, (false, false), b"regular2".to_vec()),
        ];
        let picked = pick_ranked_faces(found);
        assert_eq!(
            picked,
            vec![
                ((false, false), b"regular".to_vec()),
                ((true, false), b"heavy".to_vec()),
            ]
        );
    }

    #[test]
    fn fold_family_maps_full_width_names_to_their_ascii_key() {
        assert_eq!(fold_family("ＭＳ 明朝"), "ms明朝");
        assert_eq!(fold_family("MS Mincho"), "msmincho");
        assert_eq!(fold_family("FangSong_GB2312"), "fangsonggb2312");
        assert_eq!(cjk_file_stems("ＭＳ ゴシック"), &["msgothic"]);
        assert!(cjk_file_stems("Calibri").is_empty());
    }

    #[test]
    fn cjk_family_takes_its_own_face_out_of_word_s_collection() {
        // fixtures_500 0016d88a: "ＭＳ 明朝" text vanished (no face held
        // its glyphs). Word draws msmincho.ttc face 0, MS Mincho.
        let ttc = "/Applications/Microsoft Word.app/Contents/Resources/DFonts/msmincho.ttc";
        if !Path::new(ttc).is_file() {
            return;
        }
        let faces = installed_family_faces("ＭＳ 明朝");
        let (_, bytes) = faces
            .iter()
            .find(|(style, _)| *style == (false, false))
            .expect("a regular MS Mincho face");
        let face = ttf_parser::Face::parse(bytes, 0).expect("standalone sfnt");
        assert!(
            face_family_names(&face, ttf_parser::name_id::FAMILY).contains(&"msmincho".to_string())
        );
        assert!(face.glyph_index('明').is_some(), "CJK glyphs present");
        let pfaces = installed_family_faces("MS PMincho");
        let (_, pbytes) = pfaces.first().expect("MS PMincho face");
        let pface = ttf_parser::Face::parse(pbytes, 0).expect("standalone sfnt");
        assert!(
            face_family_names(&pface, ttf_parser::name_id::FAMILY)
                .contains(&"mspmincho".to_string()),
            "the collection's second face, not the first"
        );
    }

    #[test]
    fn missing_east_asian_family_falls_to_word_s_cjk_face() {
        // fixtures_500 0025b0d3: 標楷體 (charset 88) is not installed;
        // Word draws it in Microsoft YaHei. A Japanese family Word lacks
        // (0041d394's HGP行書体, charset 80) is Yu Gothic.
        let dfonts = "/Applications/Microsoft Word.app/Contents/Resources/DFonts";
        if !Path::new(dfonts).join("msyh.ttc").is_file()
            || !Path::new(dfonts).join("YuGothR.ttc").is_file()
        {
            return;
        }
        let table = super::super::font_table::parse_font_table_xml(
            r#"<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
                 <w:font w:name="標楷體"><w:charset w:val="88"/><w:family w:val="script"/><w:pitch w:val="fixed"/></w:font>
                 <w:font w:name="HGP行書体"><w:charset w:val="80"/><w:family w:val="script"/></w:font>
                 <w:font w:name="SomeLatin"><w:panose1 w:val="020B0604020202020204"/><w:family w:val="swiss"/></w:font>
               </w:fonts>"#,
        );
        let mut embedded = EmbeddedFonts::new();
        add_cjk_fallbacks(&mut embedded);
        let fonts = Fonts::for_document(&embedded);
        let physical = |family: &str| {
            let (face, _) = fonts.classify_in(family, false, false, &table);
            fonts.get(face).pdf_name().to_string()
        };
        assert_eq!(physical("標楷體"), "MicrosoftYaHei");
        assert!(physical("HGP行書体").starts_with("YuGothic"));
        assert_eq!(
            physical("SomeLatin"),
            "ArialMT",
            "Latin families are untouched"
        );
    }

    #[test]
    fn resolve_dead_end_altname_keeps_the_original_generic() {
        // fixtures_500 019d9ee6: Myriad Pro (swiss) → altName Segoe UI,
        // which is neither installed nor in the table. Word paints Arial,
        // the swiss generic, not the unknown-family Cambria.
        let table = super::super::font_table::parse_font_table_xml(
            r#"<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
                 <w:font w:name="SomeMyriad"><w:altName w:val="SomeSegoe"/><w:family w:val="swiss"/></w:font>
               </w:fonts>"#,
        );
        let fonts = Fonts::new();
        assert_eq!(
            fonts.resolve_in("SomeMyriad", false, false, &table),
            FaceId::SansRegular
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
        fonts.insert_embedded("Press Start 2P", false, false, FaceId::MonoRegular.bytes());
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
    fn embedded_faces_borrow_the_conversion_bytes() {
        // #11: embedded payloads were copied into a process-global intern
        // table and Box::leak-ed, so a long-lived caller kept every
        // document's fonts. The face now borrows the conversion's buffer
        // (same allocation), which the compiler ties to the Fonts value.
        let embedded: EmbeddedFonts = HashMap::from([(
            ("Press Start 2P".to_string(), false, false),
            Arc::from(FaceId::MonoRegular.bytes()),
        )]);
        let fonts = Fonts::for_document(&embedded);
        let face = fonts.resolve("Press Start 2P", false, false);
        assert!(matches!(face, FaceRef::Embedded(_)));
        let owned = &embedded[&("Press Start 2P".to_string(), false, false)];
        assert_eq!(fonts.get(face).bytes().as_ptr(), owned.as_ptr());
        assert_eq!(fonts.get(face).pdf_name(), "LiberationMono");
    }

    #[test]
    fn resolve_embedded_does_not_steal_unrelated_families() {
        let mut fonts = Fonts::new();
        fonts.insert_embedded("Press Start 2P", false, false, FaceId::MonoRegular.bytes());
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
                 <w:font w:name="SomeSwiss"><w:panose1 w:val="020B0604020202020204"/><w:family w:val="swiss"/></w:font>
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
        fonts.insert_embedded("Press Start 2P", false, false, FaceId::MonoRegular.bytes());
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
    fn an_unknown_auto_family_takes_the_embedded_default_face() {
        // PR #167 review: the family="auto" fallback resolved the document
        // default through the catalogue only, so an embedded default face
        // lost to Cambria.
        let mut fonts = Fonts::new();
        fonts.insert_embedded("Press Start 2P", false, false, FaceId::MonoRegular.bytes());
        let mut table = super::super::font_table::parse_font_table_xml(
            "<w:fonts xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
             <w:font w:name=\"Serenity\"><w:family w:val=\"auto\"/></w:font></w:fonts>",
        );
        table.set_default_family("Press Start 2P");
        let (_, entry) = fonts.classify_in("Serenity", false, false, &table);
        assert_eq!(entry.step, FontStep::Embedded);
        assert_eq!(entry.physical, "LiberationMono");
    }

    #[test]
    fn a_family_name_cannot_leave_the_cloud_font_cache() {
        // PR #167 review: w:name is document data; joining "../.." or an
        // absolute name onto the cache path read fonts from anywhere.
        let home = std::path::Path::new("/Users/someone");
        assert!(cloud_font_dir(home, "Poppins").is_some());
        assert!(cloud_font_dir(home, "Roboto Condensed").is_some());
        for bad in ["../../../../etc", "/Library/Fonts", "a/b", "..", ".", ""] {
            assert!(
                cloud_font_dir(home, bad).is_none(),
                "{bad:?} must not map to a folder"
            );
        }
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
    fn times_new_roman_uses_the_installed_face_word_uses() {
        // fixtures_500 014babb2: Word's double-spaced Times 12 lines are
        // 27.6pt apart (13.8 single). macOS Supplemental Times New Roman
        // 5.01 has hhea lineGap 87 (13.80); Word's private DFonts copy 7.0
        // has lineGap 0 (13.29). Word draws with the installed face.
        if !Path::new("/System/Library/Fonts/Supplemental/Times New Roman.ttf").is_file() {
            return;
        }
        let fonts = Fonts::new();
        let times = fonts.get(FaceId::SerifRegular);
        assert!(
            (times.single_line_pt(12.0) - 13.8).abs() < 0.02,
            "Times 12 single line {}",
            times.single_line_pt(12.0)
        );
    }

    #[test]
    fn single_line_is_the_hhea_line_not_the_typo_line() {
        // Word's single line is hhea ascender - descender + lineGap (GDI:
        // win height + external leading). Typo metrics differ for Courier
        // (0.80 em) and Arial (1.09 em); Word's file_146 Courier 9.5 lines
        // are 10.8pt apart (1.133 em), and Arial 11 is 12.65pt.
        let fonts = Fonts::new();
        let mono = fonts.get(FaceId::MonoRegular);
        assert!((mono.single_line_pt(9.5) - 9.5 * 2320.0 / 2048.0).abs() < 0.02);
        let sans = fonts.get(FaceId::SansRegular);
        assert!((sans.single_line_pt(11.0) - 11.0 * 2355.0 / 2048.0).abs() < 0.02);
        let carlito = fonts.get(FaceId::CarlitoRegular);
        assert!(
            (carlito.single_line_pt(11.0) - 11.0 * 2500.0 / 2048.0).abs() < 0.02,
            "Calibri/Carlito hhea equals typo: unchanged at 13.43"
        );
    }

    #[test]
    fn postscript_name_decodes_eight_bit_records() {
        assert_eq!(
            ascii_record_name(b"PressStart2P-Regular").as_deref(),
            Some("PressStart2P-Regular")
        );
        assert_eq!(ascii_record_name(b""), None);
        assert_eq!(
            ascii_record_name(b"Has Space"),
            None,
            "PostScript names have no spaces"
        );
        assert_eq!(ascii_record_name(&[0xC3, 0xA9]), None);
        assert_eq!(
            ttf_postscript_name(FaceId::MonoRegular.bytes()).as_deref(),
            Some("LiberationMono"),
            "Unicode records still win"
        );
    }

    #[test]
    fn font_table_scope_is_restored_after_a_panicking_conversion() {
        let table = super::super::font_table::parse_font_table_xml(
            r#"<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
                 <w:font w:name="SomeRare"><w:altName w:val="Arial"/></w:font>
               </w:fonts>"#,
        );
        let fonts = Fonts::new();
        let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            with_font_table(table, || panic!("conversion failed mid-document"))
        }));
        assert!(caught.is_err());
        assert_eq!(
            fonts.resolve("SomeRare", false, false),
            FaceId::CambriaRegular,
            "a panicking conversion must not leave its altName table installed"
        );
    }

    #[test]
    fn font_report_scope_is_restored_after_a_panic() {
        let fonts = Fonts::new();
        let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            with_font_report(|| {
                let _ = fonts.resolve("Calibri", false, false);
                panic!("conversion failed mid-document")
            })
        }));
        assert!(caught.is_err());
        assert!(
            FONT_REPORT.with(|slot| slot.borrow().is_none()),
            "a panicking conversion must not leave a report collector installed"
        );
    }

    #[test]
    fn long_alt_name_chain_resolves_without_deep_recursion() {
        // 20k chained rows; a recursive walk needs megabytes of stack.
        let mut xml = String::from(
            r#"<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">"#,
        );
        for i in 0..20_000 {
            write!(
                xml,
                r#"<w:font w:name="Chain{i}"><w:altName w:val="Chain{}"/></w:font>"#,
                i + 1
            )
            .unwrap();
        }
        xml.push_str(
            r#"<w:font w:name="Chain20000"><w:altName w:val="Consolas"/></w:font></w:fonts>"#,
        );
        let table = super::super::font_table::parse_font_table_xml(&xml);
        let face = std::thread::Builder::new()
            .stack_size(256 * 1024)
            .spawn(move || Fonts::new().resolve_in("Chain0", false, false, &table))
            .unwrap()
            .join()
            .expect("resolution must not overflow a 256 KiB stack");
        assert_eq!(face, FaceId::ConsolasRegular);
    }

    #[test]
    fn css_list_row_is_found_by_its_full_name() {
        // Word writes the altName against the whole w:name, quotes and all.
        let table = super::super::font_table::parse_font_table_xml(
            r#"<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
                 <w:font w:name="&quot;Unheard Of&quot;, Other, serif"><w:altName w:val="Consolas"/></w:font>
               </w:fonts>"#,
        );
        let (face, step) =
            Fonts::new().resolve_in_step(r#""Unheard Of", Other, serif"#, false, false, &table);
        assert_eq!(face, FaceId::ConsolasRegular);
        assert_eq!(step, FontStep::AltName);
    }

    #[test]
    fn unknown_face_key_keeps_its_style() {
        for (bold, italic, expected) in [
            (false, false, FaceId::CambriaRegular),
            (true, false, FaceId::CambriaBold),
            (false, true, FaceId::CambriaItalic),
            (true, true, FaceId::CambriaBoldItalic),
        ] {
            let key = FaceKey {
                family: "Definitely Not A Font".into(),
                bold,
                italic,
            };
            assert_eq!(
                Fonts::id_from_key(&key),
                expected,
                "bold={bold} italic={italic}"
            );
        }
    }

    #[test]
    fn nested_font_table_scope_restores_outer_table() {
        // Three distinct answers: outer altName Arial, inner altName
        // Consolas, and the default table's unknown-family row (Cambria).
        let outer = super::super::font_table::parse_font_table_xml(
            r#"<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
                 <w:font w:name="SomeRare"><w:altName w:val="Arial"/></w:font>
               </w:fonts>"#,
        );
        let inner = super::super::font_table::parse_font_table_xml(
            r#"<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
                 <w:font w:name="SomeRare"><w:altName w:val="Consolas"/></w:font>
               </w:fonts>"#,
        );
        let fonts = Fonts::new();

        with_font_table(outer, || {
            assert_eq!(fonts.resolve("SomeRare", false, false), FaceId::SansRegular);
            with_font_table(inner, || {
                assert_eq!(
                    fonts.resolve("SomeRare", false, false),
                    FaceId::ConsolasRegular
                );
            });
            assert_eq!(
                fonts.resolve("SomeRare", false, false),
                FaceId::SansRegular,
                "leaving an inner conversion must restore the outer font table"
            );
        });
        assert_eq!(
            fonts.resolve("SomeRare", false, false),
            FaceId::CambriaRegular,
            "leaving all conversion scopes must restore the default table"
        );
    }
}

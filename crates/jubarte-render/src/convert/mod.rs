// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! Independent DOCX → PDF, PNG and page-text conversion (no LibreOffice
//! or Word process is involved).
//!
//! Layout aims at LibreOffice visual parity: the Carlito / Liberation
//! faces (the same metric-compatible substitutes soffice embeds),
//! Word's `docDefaults` (Calibri 11 / line 276 / after 200 twips while
//! the file states them, Word 16's built-ins — Aptos 12 / line 278 /
//! after 160 twips — when the styles part or its pPrDefault /
//! rPrDefault are absent) and `sectPr` page geometry. A page-by-page
//! visual diff against a Word PDF
//! is the `diff_render` family; per-page font resolutions and the page
//! text are in `RenderReport`.
//!
//! Entry points:
//!
//! - `docx_to_pdf`, `docx_to_pdf_with`, `docx_to_pdf_report` for the
//!   PDF;
//! - `docx_to_png`, `docx_render_report`, `render` for PNG pages
//!   and the page report;
//! - `pdf_page_count` to count pages of a PDF;
//! - `DiffOptions` / `RenderDiff` / `PageDiff` (re-exported from
//!   `diff_render`) for the visual diff;
//! - `FontStep` / `FontReportEntry` / `font_report_json` (re-exported
//!   from `font`) for the conversion's font-resolution report.
//!
//! `RevisionStyle` / `RevisionPalette` back the `--revisions` CLI flag
//! and the `RevisionStyle::Word` mode the fidelity benchmarks measure
//! against Word's own Save as PDF.

pub(crate) mod altchunk;
mod cell_measure;
mod chart;
mod chrome;
mod comments;
mod diff_render;
mod drawing;
mod fields;
mod font;
mod font_table;
mod hf_lines;
mod layout;
mod layout_driver;
mod lists;
mod math_font;
mod math_layout;
mod metafile;
mod model;
mod notes;
mod ops;
mod paragraph;
mod pdf;
mod preset_geom;
mod preset_geom_data;
mod preset_text_rect_data;
mod raster;
mod runs;
mod settings;
mod shape_paint;
mod shape_points;
mod styles;
mod table;
mod table_widths;
mod textbox;
mod walk;
mod word_subst;
mod wrap;

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;
use std::rc::Rc;

use crate::namespaces::{A, M, MC, R, W, W14, W15, WNE, WP};
use crate::opc::PartFs;
use crate::util::{narrow_finite_f32, parse_finite_f32, parse_finite_f64};
use crate::xmllinq::{Dom, NodeId, XName, XNamespace};

use cell_measure::*;
use chart::*;
use chrome::*;
use comments::*;
use drawing::*;
use fields::*;
use font::{Face, FaceId, FaceRef, Fonts, NO_BREAK_HYPHEN};
use hf_lines::*;
use layout::*;
use layout_driver::*;
use lists::*;
use model::*;
use notes::*;
use ops::*;
use paragraph::*;
use runs::*;
use settings::*;
use shape_paint::*;
use shape_points::*;
use styles::*;
use table::*;
use table_widths::*;
use textbox::*;
use walk::*;
use wrap::*;

pub use diff_render::{DiffOptions, PageDiff, RenderDiff, diff_render};
pub use font::{FontReportEntry, FontStep, font_report_json};

/// The bundled catalogue faces alone (metric twins of Word's: Carlito for
/// Calibri, Liberation for Arial and Times). Chrome parts load before the
/// document's own faces exist; their measures come from these.
fn fonts() -> &'static Fonts<'static> {
    use std::sync::LazyLock;
    static FONTS: LazyLock<Fonts<'static>> = LazyLock::new(Fonts::new);
    &FONTS
}
pub(crate) use crate::numbering::{NumFmt, format_num, names_deeper_level, parse_num_fmt};
#[cfg(test)]
use crate::numbering::{
    alpha_label, cardinal_label, chicago_label, chinese_counting_label, decimal_fullwidth_label,
    hebrew1_label, hindi_counting_label, ideograph_digital_label, ideograph_enclosed_circle_label,
    ideograph_legal_traditional_label, ideograph_zodiac_traditional_label, japanese_counting_label,
    korean_counting_label, korean_legal_label, ordinal_label, ordinal_text_label, roman_label,
    thai_counting_label,
};
pub use crate::render_options::{
    CommentPlacement, MarkLines, PdfOptions, RenderRequest, RevisionMark, RevisionPalette,
    RevisionStyle,
};
pub(crate) use crate::table_dom::{TblLook, cell_is_deleted, table_look, wrapped_children};
use pdf::{Op, Page, PdfComment};

/// Opening a DOCX, emitting a PDF, or selecting pages failed.
#[derive(Debug)]
pub enum ConvertError {
    /// The bytes were not a readable OPC package, or its strict-to-transitional
    /// normalization couldn't open.
    OpenPackage(String),
    /// The package has no main document part.
    MissingDocument,
    /// PDF object assembly failed.
    Emit(String),
    /// A PNG resolution outside `1..=MAX_PNG_DPI`, or a page too large to
    /// rasterize at the requested one.
    Raster(String),
    /// [`RenderRequest::pages`] named a page past the document's last.
    PageOutOfRange {
        /// The zero-based document page asked for.
        requested: usize,
        /// The document's pages: the layout's, and the listing pages of
        /// `CommentPlacement::End` after them.
        page_count: usize,
    },
    /// [`RenderRequest::pages`] named a document page that
    /// [`PdfOptions::changed_only`] dropped.
    PageNotKept {
        /// The zero-based document page asked for.
        requested: usize,
    },
}

impl fmt::Display for ConvertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OpenPackage(err) => write!(f, "opening DOCX: {err}"),
            Self::MissingDocument => write!(f, "DOCX has no main document part"),
            Self::Emit(err) => write!(f, "emitting PDF: {err}"),
            Self::Raster(err) => write!(f, "rasterizing PNG: {err}"),
            Self::PageOutOfRange {
                requested,
                page_count,
            } => write!(
                f,
                "page {} is out of range: the document has {page_count} page{}",
                requested + 1,
                if *page_count == 1 { "" } else { "s" }
            ),
            Self::PageNotKept { requested } => write!(
                f,
                "page {} is not among the kept pages (changed pages only)",
                requested + 1
            ),
        }
    }
}

impl std::error::Error for ConvertError {}

thread_local! {
    /// The conversion in progress's `PdfOptions::revisions`.
    static REVISIONS: std::cell::Cell<RevisionStyle> =
        const { std::cell::Cell::new(RevisionStyle::Conventional) };
    /// The conversion in progress's `PdfOptions::comments` and
    /// `PdfOptions::changed_only`.
    static PAGE_PLAN: std::cell::Cell<(CommentPlacement, bool)> =
        const { std::cell::Cell::new((CommentPlacement::Margin, false)) };
}

/// Run `f` with `options` as the conversion in progress's.
/// The options restored on drop, so a panicking conversion that a caller
/// catches (Python's `PanicException`) leaves none of its own behind.
fn with_options<T>(options: PdfOptions, f: impl FnOnce() -> T) -> T {
    struct Restore(RevisionStyle, (CommentPlacement, bool));
    impl Drop for Restore {
        fn drop(&mut self) {
            REVISIONS.with(|r| r.set(self.0));
            PAGE_PLAN.with(|p| p.set(self.1));
        }
    }
    let _restore = Restore(
        REVISIONS.with(|r| r.replace(options.revisions)),
        PAGE_PLAN.with(|p| p.replace((options.comments, options.changed_only))),
    );
    f()
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/with_options_tests.rs"]
mod with_options_tests;

/// `CommentPlacement::End` is in force.
fn comments_at_end() -> bool {
    PAGE_PLAN.with(|p| p.get().0 == CommentPlacement::End)
}

/// Rendered PDF plus the distinct font resolutions for this document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConvertedPdf {
    /// Rendered PDF (`%PDF` header).
    pub pdf: Vec<u8>,
    /// Per-document font report (plan Step 2f): one entry per distinct
    /// requested family + style the layout saw.
    pub font_report: Vec<FontReportEntry>,
}

/// Render a DOCX package to PDF with [`PdfOptions::default`]. Equivalent to
/// `docx_to_pdf_with(docx, PdfOptions::default())`.
///
/// # Errors
///
/// [`ConvertError`] from [`docx_to_pdf_with`].
pub fn docx_to_pdf(docx: &[u8]) -> Result<Vec<u8>, ConvertError> {
    docx_to_pdf_with(docx, PdfOptions::default())
}

/// Render a DOCX package to PDF, choosing how streams are written and
/// how marks are painted.
///
/// # Errors
///
/// - [`ConvertError::OpenPackage`] when `docx` is not a readable OPC
///   package;
/// - [`ConvertError::MissingDocument`] when the package has no main
///   document part;
/// - [`ConvertError::Emit`] when the PDF object assembly fails.
pub fn docx_to_pdf_with(docx: &[u8], options: PdfOptions) -> Result<Vec<u8>, ConvertError> {
    Ok(docx_to_pdf_report(docx, options)?.pdf)
}

/// Render a DOCX package to PDF and return the font-resolution report for
/// it. The PDF is the `pdf` field of the result; the font report is the
/// entries the conversion made for that document.
///
/// # Errors
///
/// As [`docx_to_pdf_with`].
pub fn docx_to_pdf_report(docx: &[u8], options: PdfOptions) -> Result<ConvertedPdf, ConvertError> {
    let (result, font_report) = font::with_font_report(|| docx_to_pdf_inner(docx, options));
    Ok(ConvertedPdf {
        pdf: result?,
        font_report,
    })
}

fn docx_to_pdf_inner(docx: &[u8], options: PdfOptions) -> Result<Vec<u8>, ConvertError> {
    with_options(options, || docx_to_pdf_body(docx, options))
}

fn docx_to_pdf_body(docx: &[u8], options: PdfOptions) -> Result<Vec<u8>, ConvertError> {
    with_pages(docx, |fonts, pages| pdf::emit(fonts, pages, options))
}

/// Text painted on one page, in paint order (header first when present).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageText {
    /// Zero-based index among the pages written by the run.
    pub index: usize,
    /// The page's number in the document, from 1. The comment listing's
    /// pages number on after the last document page when
    /// [`PdfOptions::comments`] is `End`.
    pub number: usize,
    /// One line per baseline, in painting order.
    pub text: String,
}

/// Page facts from one layout pass. `page_count` is the renderer's result,
/// not a saved `docProps/app.xml` figure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderReport {
    /// Pages written: every page laid out, less those
    /// `PdfOptions::changed_only` drops, plus a `CommentPlacement::End`
    /// listing, narrowed to [`RenderRequest::pages`] when set.
    pub page_count: usize,
    /// The document's pages: every page laid out, plus a
    /// `CommentPlacement::End` listing, whatever was written. The last
    /// [`PageText::number`] a full render can have; names padded to its
    /// width sort the same whichever pages are written.
    pub document_pages: usize,
    /// Per-page painted text.
    pub pages: Vec<PageText>,
    /// Distinct font resolutions (requested → physical face).
    pub fonts: Vec<FontReportEntry>,
}

impl RenderReport {
    /// Each written page's first line, at most 60 characters: the
    /// `page_starts` an edit bundle's `render` line lists.
    #[must_use]
    pub fn page_starts(&self) -> Vec<String> {
        self.pages
            .iter()
            .map(|page| {
                page.text
                    .lines()
                    .next()
                    .unwrap_or("")
                    .chars()
                    .take(60)
                    .collect()
            })
            .collect()
    }

    /// `{"page_count", "document_pages", "pages": [{"index", "number", "text"}],
    /// "fonts": [...]}`.
    #[must_use]
    pub fn to_json(&self) -> String {
        let pages: Vec<serde_json::Value> = self
            .pages
            .iter()
            .map(|p| serde_json::json!({"index": p.index, "number": p.number, "text": p.text}))
            .collect();
        let fonts: serde_json::Value =
            serde_json::from_str(&font::font_report_json(&self.fonts)).unwrap_or_default();
        serde_json::json!({
            "page_count": self.page_count,
            "document_pages": self.document_pages,
            "pages": pages,
            "fonts": fonts,
        })
        .to_string()
    }
}

/// Output of [`render`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rendered {
    /// PDF bytes when requested and at least one page is written.
    pub pdf: Option<Vec<u8>>,
    /// One PNG per page when requested (page order), or one per selected
    /// page in ascending order when [`RenderRequest::pages`] is set.
    pub pngs: Vec<Vec<u8>>,
    /// Page count, page text and font resolutions.
    pub report: RenderReport,
}

/// Highest PNG resolution [`render`] accepts (`1.0..=1200.0`).
pub const MAX_PNG_DPI: f32 = 1200.0;

/// Lay the document out once and emit any combination of PDF, PNG pages and
/// the page report.
/// Lay the document out once and emit any combination of PDF, PNG pages
/// and the page report. The default `RenderRequest` writes nothing.
///
/// One layout pass covers the whole document; `request.pages` then
/// narrows what is written. The font report always runs, so an empty
/// `request` still pages the document and counts fonts.
///
/// # Errors
///
/// - [`ConvertError::OpenPackage`] / [`ConvertError::MissingDocument`]
///   from the layout pass;
/// - [`ConvertError::Raster`] when `request.png_dpi` is `Some(d)` outside
///   `1.0..=MAX_PNG_DPI`, or any page is too large to rasterize at `d`;
/// - [`ConvertError::PageNotKept`] when `request.pages` names a page
///   [`PdfOptions::changed_only`] dropped;
/// - [`ConvertError::PageOutOfRange`] when `request.pages` names a page
///   past the document's last;
/// - [`ConvertError::Emit`] when PDF object assembly fails.
pub fn render(
    docx: &[u8],
    options: PdfOptions,
    request: RenderRequest,
) -> Result<Rendered, ConvertError> {
    let RenderRequest {
        pdf: want_pdf,
        png_dpi,
        pages: wanted,
    } = request;
    if let Some(dpi) = png_dpi
        && !(1.0..=MAX_PNG_DPI).contains(&dpi)
    {
        return Err(ConvertError::Raster(format!(
            "dpi {dpi} is outside 1..={MAX_PNG_DPI}"
        )));
    }
    let (result, font_report) = font::with_font_report(|| {
        with_options(options, || {
            with_layout(docx, |fonts, pages, facts| {
                let document_pages = document_pages(pages, facts.page_count);
                let written: Vec<&pdf::Page> = selected_pages(wanted, pages, facts.page_count)?
                    .into_iter()
                    .map(|i| &pages[i])
                    .collect();
                let pdf =
                    (want_pdf && !written.is_empty()).then(|| pdf::emit(fonts, &written, options));
                let pngs: Vec<Vec<u8>> = match png_dpi {
                    Some(dpi) => written
                        .iter()
                        .map(|page| {
                            raster::paint_page(fonts, page, dpi)
                                .map(|pixmap| raster::encode_png(&pixmap))
                                .ok_or_else(|| {
                                    ConvertError::Raster(format!(
                                        "page {} is too large to rasterize at {dpi} dpi",
                                        page.number
                                    ))
                                })
                        })
                        .collect::<Result<_, _>>()?,
                    None => Vec::new(),
                };
                let texts = written
                    .iter()
                    .enumerate()
                    .map(|(index, page)| PageText {
                        index,
                        number: page.number,
                        text: raster::page_text(fonts, page),
                    })
                    .collect();
                Ok::<_, ConvertError>((pdf, pngs, written.len(), document_pages, texts))
            })
        })
    });
    let (pdf, pngs, page_count, document_pages, pages) = result??;
    Ok(Rendered {
        pdf,
        pngs,
        report: RenderReport {
            page_count,
            document_pages,
            pages,
            fonts: font_report,
        },
    })
}

/// The document's page count: `laid` (the layout's pages) or, past it, the
/// last comment-listing page.
fn document_pages(pages: &[pdf::Page], laid: usize) -> usize {
    pages.iter().map(|p| p.number).max().unwrap_or(0).max(laid)
}

/// Where in `pages` (the pages kept) each wanted zero-based document page
/// is, ascending and without repeats: every kept page when `wanted` is
/// `None`. `laid` is the layout's page count; the comment listing's pages
/// number on after it.
fn selected_pages(
    wanted: Option<Vec<usize>>,
    pages: &[pdf::Page],
    laid: usize,
) -> Result<Vec<usize>, ConvertError> {
    let Some(mut wanted) = wanted else {
        return Ok((0..pages.len()).collect());
    };
    wanted.sort_unstable();
    wanted.dedup();
    let page_count = document_pages(pages, laid);
    if let Some(&requested) = wanted.last()
        && requested >= page_count
    {
        return Err(ConvertError::PageOutOfRange {
            requested,
            page_count,
        });
    }
    wanted
        .into_iter()
        .map(|requested| {
            pages
                .iter()
                .position(|p| p.number == requested + 1)
                .ok_or(ConvertError::PageNotKept { requested })
        })
        .collect()
}

/// Rasterize every page to PNG at `dpi` (96 is screen resolution, 150
/// reads comfortably, 300 is print).
///
/// # Errors
///
/// [`ConvertError::OpenPackage`] when `docx` is not a readable package,
/// [`ConvertError::MissingDocument`] when it has no main document part;
/// [`ConvertError::Raster`] when `dpi` is outside `1.0..=MAX_PNG_DPI` or
/// when a page is too large to rasterize at `dpi`.
pub fn docx_to_png(
    docx: &[u8],
    options: PdfOptions,
    dpi: f32,
) -> Result<Vec<Vec<u8>>, ConvertError> {
    Ok(render(
        docx,
        options,
        RenderRequest {
            pdf: false,
            png_dpi: Some(dpi),
            pages: None,
        },
    )?
    .pngs)
}

/// Page count, page text and font resolutions from one layout pass, with
/// no PDF or PNG written.
///
/// # Errors
///
/// As [`render`] with a default [`RenderRequest`]: an
/// [`ConvertError::OpenPackage`] or
/// [`ConvertError::MissingDocument`] from the layout pass.
pub fn docx_render_report(docx: &[u8], options: PdfOptions) -> Result<RenderReport, ConvertError> {
    Ok(render(docx, options, RenderRequest::default())?.report)
}

/// What one layout pass learned about pages, for writing field results back
/// into the package (`jubarte::fields`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[doc(hidden)]
pub struct LayoutFacts {
    /// Pages laid out (the `NUMPAGES` result).
    pub page_count: usize,
    /// Bookmark name to the page label a `PAGEREF` to it shows (the section's
    /// number format and chapter prefix applied, so "iii" or "2-1").
    pub bookmark_pages: BTreeMap<String, String>,
}

/// Lay `docx` out as the default convert does and report its page facts.
#[doc(hidden)]
pub fn layout_facts(docx: &[u8]) -> Result<LayoutFacts, ConvertError> {
    // Every page, whatever the caller's conversion asks for: the facts
    // are the whole document's.
    with_options(PdfOptions::default(), || {
        with_layout(docx, |_, _, facts| facts.clone())
    })
}

/// The shared layout pipeline: open, resolve fonts, lay out, then hand the
/// pages to `emit`.
fn with_pages<T>(
    docx: &[u8],
    emit: impl FnOnce(&Fonts, &[pdf::Page]) -> T,
) -> Result<T, ConvertError> {
    with_layout(docx, |fonts, pages, _| emit(fonts, pages))
}

/// [`with_pages`] that also hands `emit` the layout's page facts.
fn with_layout<T>(
    docx: &[u8],
    emit: impl FnOnce(&Fonts, &[pdf::Page], &LayoutFacts) -> T,
) -> Result<T, ConvertError> {
    crate::admission::admit_package(docx)
        .map_err(|err| ConvertError::OpenPackage(err.to_string()))?;
    let normalized = crate::strict_translation::strict_to_transitional_docx(docx)
        .map_err(|err| ConvertError::OpenPackage(err.to_string()))?;
    let mut pkg =
        PartFs::open(&normalized).map_err(|err| ConvertError::OpenPackage(format!("{err:?}")))?;
    let main = pkg
        .main_document_part()
        .or_else(|| {
            pkg.part_bytes("word/document.xml")
                .map(|_| "word/document.xml".to_string())
        })
        .ok_or(ConvertError::MissingDocument)?;
    resolve_text_alternates(&mut pkg, &main);
    let xml = pkg
        .part_string(&main)
        .ok_or(ConvertError::MissingDocument)?;
    // Word merges `w:altChunk` content (HTML / MHT) into the body on open.
    let xml = if xml.contains("<w:altChunk") {
        altchunk::expand(&xml, |rid| {
            rel_target_path(&pkg, &main, rid).and_then(|p| pkg.part_bytes(&p).map(<[u8]>::to_vec))
        })
    } else {
        xml
    };
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).ok_or(ConvertError::MissingDocument)?;
    let body = dom
        .descendants(root, Some(&W::body()))
        .into_iter()
        .next()
        .ok_or(ConvertError::MissingDocument)?;

    let table = font_table::load_font_table(&pkg);
    let mut embedded = font_table::load_embedded_fonts(&pkg, &table);
    let mut family_names = rfont_names(&xml);
    let mut run_faces = latin_font_names(&xml);
    for part in ["word/styles.xml", "word/theme/theme1.xml"] {
        if let Some(text) = pkg.part_string(part) {
            family_names.extend(rfont_names(&text));
            run_faces.extend(latin_font_names(&text));
        }
    }
    // Headers, footers, notes and comments paint too: their faces and
    // their scripts count (PR #247 review: Hangul only in a header lost its
    // glyphs without the Korean fallback).
    let stories = story_parts_xml(&pkg, &main);
    for text in &stories {
        family_names.extend(rfont_names(text));
        run_faces.extend(latin_font_names(text));
    }
    let any_text = |test: fn(char) -> bool| {
        std::iter::once(&xml)
            .chain(&stories)
            .any(|t| t.chars().chain(char_refs(t)).any(test))
    };
    // List and page numbers in an East Asian format paint characters the
    // text never holds (a footer's ideographEnclosedCircle PAGE label).
    let numbering = pkg.part_string("word/numbering.xml");
    let writes = |test: fn(char) -> bool| {
        any_text(test)
            || [Some(xml.as_str()), numbering.as_deref()]
                .into_iter()
                .flatten()
                .any(|part| number_formats_write(part, test))
    };
    let has_cjk = writes(takes_cjk_fallback);
    // Word sets OMML text in Cambria Math whether or not the font table
    // lists it (sd_2750_borderbox lists only Cambria).
    let has_math = std::iter::once(&xml)
        .chain(&stories)
        .any(|t| writes_omml(t));
    if has_math {
        family_names.push("Cambria Math".to_string());
        run_faces.push("Cambria Math".to_string());
    }
    // Balanced spaces take a run's eastAsia face's width, Latin text or
    // not (`balance_spaces`), so that face loads like a painted one.
    if settings_flag(&pkg, "balanceSingleByteDoubleByteWidth") {
        let styles = pkg.part_string("word/styles.xml");
        for text in std::iter::once(&xml).chain(&stories).chain(styles.as_ref()) {
            run_faces.extend(font_attr_names(text, &["w:eastAsia=\""]));
        }
    }
    font::add_installed_faces(&mut embedded, &table, &family_names, &run_faces, has_cjk);
    if has_cjk {
        font::add_cjk_fallbacks(&mut embedded);
    }
    if any_text(is_thaana) {
        font::add_thaana_fallback(&mut embedded);
    }
    if writes(is_thai) {
        font::add_script_fallback(
            &mut embedded,
            font::THAI_FALLBACK,
            &["Leelawadee UI", "Thonburi"],
        );
    }
    if writes(is_devanagari) {
        font::add_script_fallback(
            &mut embedded,
            font::DEVANAGARI_FALLBACK,
            &["Mangal", "Kohinoor Devanagari", "Devanagari MT"],
        );
    }
    let fonts = Fonts::for_document(&embedded);
    font::with_font_table(table, || {
        let _math = MathAlphanumerics::set(if has_math {
            let face = fonts.get(fonts.resolve("Cambria Math", false, false));
            MathFace::probe(|c| face.glyph(c) != 0)
        } else {
            MathFace::All
        });
        // The same face supplies glyph ids for the math boxes. A face
        // with no MATH table still lays out, on the TeX fallback. Without
        // an equation the face is never resolved: resolving it would list
        // Cambria Math in the font report of a document that asks for no
        // such font.
        let _math_font = math_font::MathFontGuard::set(if has_math {
            let math_face = fonts.get(fonts.resolve("Cambria Math", false, false));
            math_font::MathFont::from_bytes(math_face.bytes())
                .unwrap_or_else(|| math_font::MathFont::fallback(math_face.upem))
        } else {
            math_font::MathFont::fallback(1000.0)
        });
        let core = load_core_dates(&pkg);
        with_core_dates(core, || {
            let mut sheet = load_stylesheet(&pkg);
            if let Some(tab) = settings_default_tab_pt(&pkg) {
                sheet.defaults.page.default_tab = tab;
            }
            sheet.defaults.page.gutter_at_top = settings_flag(&pkg, "gutterAtTop");
            // Word Save-as-PDF draws the grey balloon pasteboard (and shrinks
            // the page) for comments and inserted or deleted table cells
            // only. Live Word 2026-09-25: 120 tracked insertions and
            // deletions with w:trackRevisions, with or without formatting
            // changes, keep the full page (English redline e1de10f3 was
            // shrunk on a 100/100 count).
            // Comments listed at the end bring no pane: the page keeps its
            // width (table cell marks still do).
            let balloons = if comments_at_end() {
                Some(HashSet::new())
            } else {
                word_balloon_comments(&pkg, &main, &dom, body)
            };
            if document_has_balloons(&pkg, &main, balloons.as_ref()) {
                sheet.defaults.page.balloon_gutter = 144.0;
            }
            let page = load_page_setup(&dom, body, &sheet.defaults.page);
            let hf = first_section_hf(&pkg, &main, &dom, body, &sheet);
            let mut blocks = collect_blocks(&pkg, &main, &dom, body, &sheet, &fonts);
            let compat_mode = settings_compat_mode(&pkg);
            at_least_off_grid(&mut blocks, compat_mode);
            mark_ideograph_words(&mut blocks, &fonts, compat_mode);
            if settings_flag(&pkg, "balanceSingleByteDoubleByteWidth")
                && east_asian_layout(&pkg, &sheet, &fonts)
            {
                balance_spaces(&mut blocks, &fonts, sheet.defaults.run.family_ea.as_deref());
            }
            squeeze_bracket_pairs(&mut blocks);
            if settings_flag(&pkg, "adjustLineHeightInTable") {
                grid_table_cells(&mut blocks, page.grid_pitch);
            }
            let display = number_footnote_refs(&mut blocks);
            resolve_cell_fields(&mut blocks);
            keep_opening_row_mark(&mut blocks);
            let (notes, separator) = load_footnotes(&pkg, &main, &sheet);
            let footnotes = FootnoteCatalog {
                notes,
                display,
                separator,
            };
            let (mut pages, facts) =
                layout_with_facts(&fonts, &page, &hf, &blocks, compat_mode, footnotes);
            for (i, page) in pages.iter_mut().enumerate() {
                page.number = i + 1;
            }
            let laid = pages.len();
            if PAGE_PLAN.with(|p| p.get().1) {
                keep_changed_pages(&mut pages);
            }
            if comments_at_end() {
                list_comments_at_end(&fonts, &sheet, &mut pages, laid);
            } else {
                paint_comment_balloons(&fonts, &sheet, &mut pages);
            }
            Ok(emit(&fonts, &pages, &facts))
        })
    })
}

/// Count page objects in `pdf`: `/Type /Page` (not `/Type /Pages`).
///
/// Counts by matching the substring; an invalid PDF that happens to
/// contain `/Type /Page` strings will over-count. The conservative use
/// is `page_count` from a [`RenderReport`].
pub fn pdf_page_count(pdf: &[u8]) -> usize {
    let text = String::from_utf8_lossy(pdf);
    let needle = "/Type /Page";
    text.match_indices(needle)
        .filter(|(idx, _)| {
            let rest = &text[idx + needle.len()..];
            !rest.starts_with('s')
        })
        .count()
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/default_tab_tests.rs"]
mod default_tab_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/pg_border_tests.rs"]
mod pg_border_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/settings_tests.rs"]
mod settings_tests;

pub use crate::util::datetime::utc_now_iso8601;

use crate::util::datetime::{CivilDateTime, parse_w3cdtf, utc_now};

#[cfg(test)]
use crate::util::datetime::{
    civil_from_unix_days, civil_from_unix_secs, iso8601, unix_days_from_civil,
};

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/page_num_fmt_labels.rs"]
mod page_num_fmt_labels;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/page_count_tests.rs"]
mod page_count_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/theme_slot_tests.rs"]
mod theme_slot_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/field_tests.rs"]
mod field_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/drawing_tests.rs"]
mod drawing_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/numbering_tests.rs"]
mod numbering_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/table_tests.rs"]
mod table_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/comments_spacing_tests.rs"]
mod comments_spacing_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/regression_tests.rs"]
mod regression_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/comment_listing_review_tests.rs"]
mod comment_listing_review_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/coverage_boundary_tests.rs"]
mod coverage_boundary_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/coverage_round_two_tests.rs"]
mod coverage_round_two_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/converter_semantic_boundary_tests.rs"]
mod converter_semantic_boundary_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/coverage_round_three_tests.rs"]
mod coverage_round_three_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/rendering_uncovered_contract_tests.rs"]
mod rendering_uncovered_contract_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/complete_coordinate_layout_contract_tests.rs"]
mod complete_coordinate_layout_contract_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/complete_coordinate_vml_owner_contract_tests.rs"]
mod complete_coordinate_vml_owner_contract_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/field_group_coordinate_contract_tests.rs"]
mod field_group_coordinate_contract_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/chart_cache_and_diagram_threshold_contract_tests.rs"]
mod chart_cache_and_diagram_threshold_contract_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/chart_series_cache_owner_regression_tests.rs"]
mod chart_series_cache_owner_regression_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/cell_picture_owned_room_contract_tests.rs"]
mod cell_picture_owned_room_contract_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/style_hit_and_following_line_contract_tests.rs"]
mod style_hit_and_following_line_contract_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/encoded_picture_effect_contract_tests.rs"]
mod encoded_picture_effect_contract_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/chart_admitted_series_owner_boundaries.rs"]
mod chart_admitted_series_owner_boundaries;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/automatic_column_source_matrix_tests.rs"]
mod automatic_column_source_matrix_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/chrome_collector_source_boundary_tests.rs"]
mod chrome_collector_source_boundary_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/flow_picture_branch_tests.rs"]
mod flow_picture_branch_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/layout_scenario_branch_tests.rs"]
mod layout_scenario_branch_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/layout_object_branch_tests.rs"]
mod layout_object_branch_tests;

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Visual page diff: render two documents at one resolution and compare the
//! pages pixel for pixel. The renderer is deterministic (the same input
//! writes the same bytes), so any changed pixel is a changed page.

use image::{ImageFormat, Rgba, RgbaImage};
use serde::Serialize;

use super::{ConvertError, PdfOptions, RenderReport, RenderRequest, render};

/// Ink of changed pixels and their box in an overlay.
const MAGENTA: Rgba<u8> = Rgba([255, 0, 255, 255]);
/// Pixels between the changed region and the box drawn around it.
const BOX_MARGIN: u32 = 2;

/// How [`diff_render`] renders and marks the two documents.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DiffOptions {
    /// Raster resolution of both sides.
    pub dpi: f32,
    /// Layout and revision style of both sides.
    pub pdf: PdfOptions,
    /// Paint changed pixels magenta over `b`'s page and box them.
    pub overlay: bool,
}

impl Default for DiffOptions {
    fn default() -> Self {
        Self {
            dpi: 100.0,
            pdf: PdfOptions::default(),
            overlay: true,
        }
    }
}

/// How one page differs between the two documents.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PageDiff {
    /// Zero-based page index.
    pub index: usize,
    /// Changed pixels over all pixels, 0.0 to 1.0. 1.0 when the page exists
    /// on one side only or the two pages differ in size.
    pub changed_ratio: f32,
    /// `[x0, y0, x1, y1]` in pixels around every changed pixel (`x1` and
    /// `y1` exclusive); `None` when equal.
    pub bbox: Option<[u32; 4]>,
    /// `"a"` or `"b"` when the page exists on one side only; never skipped.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub only_in: Option<&'static str>,
}

impl PageDiff {
    /// Whether this page differs at all.
    #[must_use]
    pub fn differs(&self) -> bool {
        self.changed_ratio > 0.0 || self.only_in.is_some()
    }
}

/// Output of [`diff_render`].
#[derive(Clone, Debug, PartialEq)]
pub struct RenderDiff {
    /// One entry per page of the longer document, in page order.
    pub pages: Vec<PageDiff>,
    /// `a`'s PNG pages.
    pub a: Vec<Vec<u8>>,
    /// `b`'s PNG pages.
    pub b: Vec<Vec<u8>>,
    /// Per entry of `pages`: `b`'s page with the change painted and boxed,
    /// or `None` when the page is equal, exists on one side only, differs
    /// in size, or overlays were not asked for.
    pub overlays: Vec<Option<Vec<u8>>>,
    /// `a`'s page count, page text and fonts.
    pub a_report: RenderReport,
    /// `b`'s page count, page text and fonts.
    pub b_report: RenderReport,
}

impl RenderDiff {
    /// Whether any page differs.
    #[must_use]
    pub fn differs(&self) -> bool {
        self.pages.iter().any(PageDiff::differs)
    }
}

/// Which pages of `a` and `b` differ, from one layout pass each at
/// `options.dpi`.
pub fn diff_render(a: &[u8], b: &[u8], options: &DiffOptions) -> Result<RenderDiff, ConvertError> {
    let request = RenderRequest {
        pdf: false,
        png_dpi: Some(options.dpi),
        pages: None,
    };
    let a = render(a, options.pdf, request.clone())?;
    let b = render(b, options.pdf, request)?;
    let count = a.pngs.len().max(b.pngs.len());
    let mut pages = Vec::with_capacity(count);
    let mut overlays = Vec::with_capacity(count);
    for index in 0..count {
        let (page, overlay) = match (a.pngs.get(index), b.pngs.get(index)) {
            (Some(pa), Some(pb)) => compare(index, pa, pb, options.overlay)?,
            (Some(only), None) => (one_side(index, only, "a")?, None),
            (None, Some(only)) => (one_side(index, only, "b")?, None),
            (None, None) => unreachable!("index is below the longer page list"),
        };
        pages.push(page);
        overlays.push(overlay);
    }
    Ok(RenderDiff {
        pages,
        a: a.pngs,
        b: b.pngs,
        overlays,
        a_report: a.report,
        b_report: b.report,
    })
}

fn decode(png: &[u8]) -> Result<RgbaImage, ConvertError> {
    image::load_from_memory_with_format(png, ImageFormat::Png)
        .map(|img| img.to_rgba8())
        .map_err(|err| ConvertError::Raster(format!("decoding page PNG: {err}")))
}

fn encode(img: &RgbaImage) -> Result<Vec<u8>, ConvertError> {
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, ImageFormat::Png)
        .map_err(|err| ConvertError::Raster(format!("encoding overlay PNG: {err}")))?;
    Ok(out.into_inner())
}

/// A page only one side has: all of it changed.
fn one_side(index: usize, png: &[u8], side: &'static str) -> Result<PageDiff, ConvertError> {
    let img = decode(png)?;
    Ok(PageDiff {
        index,
        changed_ratio: 1.0,
        bbox: Some([0, 0, img.width(), img.height()]),
        only_in: Some(side),
    })
}

fn compare(
    index: usize,
    a: &[u8],
    b: &[u8],
    overlay: bool,
) -> Result<(PageDiff, Option<Vec<u8>>), ConvertError> {
    let unchanged = PageDiff {
        index,
        changed_ratio: 0.0,
        bbox: None,
        only_in: None,
    };
    if a == b {
        return Ok((unchanged, None));
    }
    let (a, mut b) = (decode(a)?, decode(b)?);
    if a.dimensions() != b.dimensions() {
        let changed = PageDiff {
            changed_ratio: 1.0,
            bbox: Some([0, 0, b.width().max(a.width()), b.height().max(a.height())]),
            ..unchanged
        };
        return Ok((changed, None));
    }
    let mut changed = 0_u64;
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0, 0);
    for (x, y, pb) in b.enumerate_pixels_mut() {
        if a.get_pixel(x, y) != pb {
            changed += 1;
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x + 1);
            y1 = y1.max(y + 1);
            *pb = MAGENTA;
        }
    }
    if changed == 0 {
        // Same pixels in different PNG bytes.
        return Ok((unchanged, None));
    }
    let total = u64::from(b.width()) * u64::from(b.height());
    let page = PageDiff {
        changed_ratio: (changed as f64 / total as f64) as f32,
        bbox: Some([x0, y0, x1, y1]),
        ..unchanged
    };
    if !overlay {
        return Ok((page, None));
    }
    draw_box(&mut b, [x0, y0, x1, y1]);
    Ok((page, Some(encode(&b)?)))
}

/// A one-pixel magenta frame `BOX_MARGIN` pixels outside `bbox`, clipped to
/// the page.
fn draw_box(img: &mut RgbaImage, [x0, y0, x1, y1]: [u32; 4]) {
    let left = x0.saturating_sub(BOX_MARGIN);
    let top = y0.saturating_sub(BOX_MARGIN);
    let right = (x1 + BOX_MARGIN).min(img.width() - 1);
    let bottom = (y1 + BOX_MARGIN).min(img.height() - 1);
    for x in left..=right {
        img.put_pixel(x, top, MAGENTA);
        img.put_pixel(x, bottom, MAGENTA);
    }
    for y in top..=bottom {
        img.put_pixel(left, y, MAGENTA);
        img.put_pixel(right, y, MAGENTA);
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use image::codecs::png::{CompressionType, FilterType, PngEncoder};
    use image::{ImageEncoder, Rgba, RgbaImage};

    fn png(img: &RgbaImage, compression: CompressionType) -> Vec<u8> {
        let mut out = Vec::new();
        PngEncoder::new_with_quality(&mut out, compression, FilterType::Adaptive)
            .write_image(
                img,
                img.width(),
                img.height(),
                image::ExtendedColorType::Rgba8,
            )
            .unwrap();
        out
    }

    #[test]
    fn the_same_pixels_in_different_png_bytes_are_unchanged() {
        let mut img = RgbaImage::from_pixel(40, 30, Rgba([255, 255, 255, 255]));
        img.put_pixel(7, 9, Rgba([0, 0, 0, 255]));
        let (fast, best) = (
            png(&img, CompressionType::Fast),
            png(&img, CompressionType::Best),
        );
        assert_ne!(
            fast, best,
            "the two encodings must differ for this test to mean anything"
        );
        let (page, overlay) = super::compare(3, &fast, &best, true).unwrap();
        assert_eq!(page.index, 3);
        assert_eq!(page.changed_ratio, 0.0);
        assert_eq!(page.bbox, None);
        assert_eq!(overlay, None);
    }

    #[test]
    fn one_changed_pixel_is_boxed_exactly() {
        let white = RgbaImage::from_pixel(40, 30, Rgba([255, 255, 255, 255]));
        let mut dot = white.clone();
        dot.put_pixel(7, 9, Rgba([0, 0, 0, 255]));
        let a = png(&white, CompressionType::Fast);
        let b = png(&dot, CompressionType::Fast);
        let (page, overlay) = super::compare(0, &a, &b, true).unwrap();
        assert_eq!(page.bbox, Some([7, 9, 8, 10]), "x1 and y1 are exclusive");
        assert_eq!(page.changed_ratio, 1.0 / 1200.0);
        let overlay = image::load_from_memory(&overlay.unwrap())
            .unwrap()
            .to_rgba8();
        assert_eq!(
            *overlay.get_pixel(7, 9),
            super::MAGENTA,
            "the changed pixel"
        );
        assert_eq!(
            *overlay.get_pixel(5, 7),
            super::MAGENTA,
            "the box corner, 2 px out"
        );
        assert_eq!(
            *overlay.get_pixel(20, 20),
            Rgba([255, 255, 255, 255]),
            "untouched"
        );
        let garbage = super::compare(0, b"not a png", &b, true).unwrap_err();
        assert!(
            garbage.to_string().contains("decoding page PNG"),
            "{garbage}"
        );
    }
}

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-FileCopyrightText: 2024-2026 SylphxAI
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Pictures of a Word document written out beside the Markdown, as pandoc's
//! `--extract-media` does. Each raster picture is named by the first 16 hex
//! digits of its SHA-256, so a picture used twice is one file.

use std::collections::BTreeMap;

/// Pictures with more pixels than this stay alt text (checked on the header).
const MAX_PIXELS: u64 = 50_000_000;

/// Where pictures go and what has been collected.
#[derive(Debug, Default)]
pub(crate) struct Extracted {
    /// Directory the Markdown names pictures under, `/`-separated.
    dir: String,
    /// File name to bytes.
    pub(crate) files: BTreeMap<String, Vec<u8>>,
}

impl Extracted {
    pub(crate) fn new(dir: &str) -> Self {
        Self {
            dir: dir.trim_end_matches(['/', '\\']).to_string(),
            files: BTreeMap::new(),
        }
    }

    /// `![alt](dir/name.ext)` for a raster picture, collecting its bytes;
    /// `None` for anything else.
    pub(crate) fn add(&mut self, bytes: &[u8], alt: &str) -> Option<String> {
        use sha2::{Digest, Sha256};
        let ext = raster_extension(bytes)?;
        let hash: String = Sha256::digest(bytes)
            .iter()
            .take(8)
            .map(|byte| format!("{byte:02x}"))
            .collect();
        let name = format!("{hash}.{ext}");
        self.files
            .entry(name.clone())
            .or_insert_with(|| bytes.to_vec());
        let path = if self.dir.is_empty() {
            name
        } else {
            format!("{}/{name}", self.dir)
        };
        let alt = alt
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .replace('[', "\\[")
            .replace(']', "\\]");
        Some(format!("![{alt}]({})", destination(&path)))
    }
}

/// A link destination: `<...>` around paths with spaces or parentheses, and
/// `{`, `}` encoded so they never form CriticMarkup delimiters.
fn destination(path: &str) -> String {
    let path = path.replace('{', "%7B").replace('}', "%7D");
    if path
        .chars()
        .any(|c| c.is_whitespace() || matches!(c, '(' | ')' | '<' | '>'))
    {
        format!("<{}>", path.replace('<', "%3C").replace('>', "%3E"))
    } else {
        path
    }
}

/// The extension of a PNG, JPEG, GIF, BMP, TIFF or WebP picture of at most
/// [`MAX_PIXELS`], read from its header.
fn raster_extension(bytes: &[u8]) -> Option<&'static str> {
    use image::ImageFormat;
    let reader = image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .ok()?;
    let ext = match reader.format()? {
        ImageFormat::Png => "png",
        ImageFormat::Jpeg => "jpg",
        ImageFormat::Gif => "gif",
        ImageFormat::Bmp => "bmp",
        ImageFormat::Tiff => "tiff",
        ImageFormat::WebP => "webp",
        _ => return None,
    };
    let (width, height) = reader.into_dimensions().ok()?;
    (width > 0 && height > 0 && u64::from(width) * u64::from(height) <= MAX_PIXELS).then_some(ext)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    fn png(width: u32, height: u32) -> Vec<u8> {
        use image::ImageEncoder;
        let pixels = vec![0u8; (width * height) as usize];
        let mut out = Vec::new();
        image::codecs::png::PngEncoder::new(&mut out)
            .write_image(&pixels, width, height, image::ExtendedColorType::L8)
            .unwrap();
        out
    }

    #[test]
    fn names_by_content_and_collects_once() {
        let mut media = Extracted::new("media/");
        let bytes = png(4, 3);
        let first = media.add(&bytes, "A  [chart]").unwrap();
        let second = media.add(&bytes, "again").unwrap();
        assert!(first.starts_with("![A \\[chart\\]](media/"), "{first}");
        assert!(first.ends_with(".png)"), "{first}");
        assert_eq!(
            first[first.find('(').unwrap()..],
            second[second.find('(').unwrap()..]
        );
        assert_eq!(media.files.len(), 1);
        assert_eq!(media.files.keys().next().unwrap().len(), 16 + 4);
    }

    #[test]
    fn refuses_non_images_and_huge_images() {
        let mut media = Extracted::new("");
        assert_eq!(media.add(b"not an image", "x"), None);
        let mut header = png(64, 64);
        header[16..20].copy_from_slice(&10_000u32.to_be_bytes());
        header[20..24].copy_from_slice(&10_000u32.to_be_bytes());
        assert_eq!(media.add(&header, "x"), None);
        assert!(media.files.is_empty());
    }

    #[test]
    fn quotes_odd_paths() {
        assert_eq!(destination("my media/a.png"), "<my media/a.png>");
        assert_eq!(destination("a{b}.png"), "a%7Bb%7D.png");
        assert_eq!(destination("x/a.png"), "x/a.png");
        let mut media = Extracted::new("");
        assert!(media.add(&png(2, 2), "").unwrap().starts_with("![]("));
    }
}

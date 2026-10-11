// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `insert_image`: a new paragraph holding one inline picture, next to the
//! anchor paragraph. The plan carries the picture base64-encoded; the
//! picture is sized from its pixels at 96 dots per inch (at most 6.5 inches
//! wide) unless `width_emu` is given, and keeps its aspect ratio. The media
//! part, its content type and the image relationship are added to the copy,
//! and the `w:drawing` is the one the Markdown writer emits.

use std::collections::HashSet;

use crate::markdown::{Picture, drawing_xml, read_picture};
use crate::namespaces::{R, W};
use crate::opc::relative_rel_target;
use crate::xmllinq::{Dom, NodeId};

use super::{Transaction, check_text};

const IMAGE_REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image";
const WP_NS: &str = "http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing";

/// The largest picture Word lays out: 22 inches, in EMU.
const MAX_WIDTH_EMU: u64 = 20_116_800;

/// Content types the engine embeds.
const SUPPORTED: &[&str] = &[
    "image/png",
    "image/jpeg",
    "image/gif",
    "image/bmp",
    "image/tiff",
];

/// Decode standard base64 (RFC 4648, `+/`, optional `=` padding); ASCII
/// whitespace is skipped so wrapped encodings are accepted.
pub(super) fn decode_base64(text: &str) -> Result<Vec<u8>, String> {
    let mut out = Vec::with_capacity(text.len() / 4 * 3);
    let mut buffer: u32 = 0;
    let mut bits = 0u32;
    let mut padding = 0usize;
    for byte in text.bytes() {
        if byte.is_ascii_whitespace() {
            continue;
        }
        if byte == b'=' {
            padding += 1;
            continue;
        }
        if padding > 0 {
            return Err("base64 has data after its padding".into());
        }
        let value = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            other => {
                return Err(format!(
                    "image_base64 holds {:?}, which is not base64",
                    char::from(other)
                ));
            }
        };
        buffer = (buffer << 6) | u32::from(value);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buffer >> bits) as u8);
            buffer &= (1 << bits) - 1;
        }
    }
    // 6 leftover bits cannot hold a byte; more than two `=` is malformed.
    if bits >= 6 || padding > 2 {
        return Err("image_base64 is truncated".into());
    }
    if out.is_empty() {
        return Err("image_base64 is empty".into());
    }
    Ok(out)
}

/// Decode, sniff and size the picture; `(code, message)` on refusal.
pub(super) fn picture(
    base64: &str,
    content_type: Option<&str>,
    width_emu: Option<u64>,
    alt: &str,
) -> Result<Picture, (String, String)> {
    let invalid = |m: String| ("INVALID_EDIT".to_string(), m);
    if !alt.is_empty() {
        check_text(alt).map_err(|m| invalid(format!("alt: {m}")))?;
    }
    if let Some(declared) = content_type
        && !SUPPORTED.contains(&declared)
    {
        return Err((
            "UNSUPPORTED_IMAGE".into(),
            format!(
                "content_type {declared:?} is not one of {}",
                SUPPORTED.join(", ")
            ),
        ));
    }
    if let Some(width) = width_emu
        && !(1..=MAX_WIDTH_EMU).contains(&width)
    {
        return Err(invalid(format!(
            "width_emu {width} is outside 1..={MAX_WIDTH_EMU} (22 inches)"
        )));
    }
    let bytes = decode_base64(base64).map_err(invalid)?;
    let mut picture = read_picture(bytes, alt).ok_or_else(|| {
        (
            "UNSUPPORTED_IMAGE".to_string(),
            format!(
                "image_base64 is not a picture Word shows ({})",
                SUPPORTED.join(", ")
            ),
        )
    })?;
    if let Some(declared) = content_type
        && declared != picture.content_type
    {
        return Err(invalid(format!(
            "content_type {declared:?} does not match the picture, which is {}",
            picture.content_type
        )));
    }
    if let Some(width) = width_emu {
        picture.height = (picture.height * width / picture.width).max(1);
        picture.width = width;
    }
    Ok(picture)
}

impl Transaction<'_> {
    /// Add the picture's media part, content type and relationship to the
    /// copy and build its paragraph. `used` holds media parts this plan
    /// already added; `drawing_id` is the next free `wp:docPr` id.
    pub(super) fn image_paragraph(
        &mut self,
        picture: &Picture,
        used: &mut HashSet<String>,
        drawing_id: u32,
    ) -> NodeId {
        let main = self.opened.main.clone();
        let existing: HashSet<String> = self.opened.pkg.parts().into_iter().collect();
        let dir = main.rsplit_once('/').map_or("", |(dir, _)| dir);
        let part = (1..)
            .map(|n| {
                let name = format!("media/image{n}.{}", picture.extension);
                if dir.is_empty() {
                    name
                } else {
                    format!("{dir}/{name}")
                }
            })
            .find(|candidate| !existing.contains(candidate) && !used.contains(candidate))
            .expect("an unused media name");
        used.insert(part.clone());
        self.opened.pkg.set_part(&part, picture.bytes.clone());
        if self
            .opened
            .pkg
            .content_type_for(&format!("/{part}"))
            .is_none()
        {
            self.opened
                .pkg
                .add_content_type_default(picture.extension, picture.content_type);
        }
        let rel = self.opened.pkg.add_document_relationship(
            &main,
            IMAGE_REL,
            &relative_rel_target(&main, &part),
        );
        let mut drawing = String::new();
        drawing_xml(picture, &rel, drawing_id, &mut drawing);
        let xml = format!(
            r#"<w:p xmlns:w="{}" xmlns:r="{}" xmlns:wp="{WP_NS}"><w:r>{drawing}</w:r></w:p>"#,
            W::URI,
            R::URI
        );
        let dom: &mut Dom = &mut self.opened.dom;
        let document = dom.parse_xdocument(&xml);
        let p = dom.root(document).expect("drawing paragraph parses");
        dom.remove(p);
        p
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::decode_base64;

    #[test]
    fn decodes_padded_unpadded_and_wrapped_base64() {
        assert_eq!(decode_base64("TWFu").unwrap(), b"Man");
        assert_eq!(decode_base64("TWE=").unwrap(), b"Ma");
        assert_eq!(decode_base64("TQ==").unwrap(), b"M");
        assert_eq!(decode_base64("TWE").unwrap(), b"Ma");
        assert_eq!(decode_base64("TW\nFu\r\n").unwrap(), b"Man");
        assert_eq!(decode_base64("+/8=").unwrap(), [0xfb, 0xff]);
    }

    #[test]
    fn refuses_malformed_base64() {
        for bad in ["", "====", "T", "TWFu!", "TQ==TQ", "TQ==="] {
            assert!(decode_base64(bad).is_err(), "{bad:?}");
        }
    }
}

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Valid data-descriptor ZIP entries are admitted (#657, plan section
//! ZIP-18).
//!
//! A producer that streams a `.docx` cannot seek back to fill in the CRC and
//! sizes, so it writes each local header with general-purpose bit 3 set and a
//! zero CRC and sizes, then appends a `PK\x07\x08` data descriptor after the
//! entry data (APPNOTE 4.3.9.4). Such a package must be admitted like any
//! other: the reader bounds every entry by the central directory's sizes and
//! checks its CRC at the end of the read. A producer that died mid-descriptor
//! truncates the descriptor — and with it the central directory — and must
//! come back as a typed `INVALID_PACKAGE` refusal, never a panic, because the
//! Python and WASM hosts cannot catch one.
//!
//! The archives here are hand-built: the `zip` crate's writer seeks and never
//! emits descriptors, so no library writer can produce this shape.

use jubarte::WmlDocument;
use jubarte::admission::AdmissionErrorKind;
use jubarte::admission::InputLimits;
use jubarte::admission::admit;

const TYPES: &str = r#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#;
const RELS: &str = r#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;
const DOC: &str = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Hi</w:t></w:r></w:p></w:body></w:document>"#;

/// The CRC-32 the data descriptor and the central directory record carry.
fn crc32(data: &[u8]) -> u32 {
    let mut crc = flate2::Crc::new();
    crc.update(data);
    crc.sum()
}

fn u16le(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn u32le(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

/// A streaming ZIP of stored entries: every local header sets bit 3 and
/// leaves its CRC and sizes zero, the entry data is followed by a
/// `PK\x07\x08` descriptor holding the real values, and the central
/// directory at the end repeats them (stored entries compress to themselves,
/// so the two sizes are one number).
fn streaming_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut central = Vec::new();
    for (name, data) in entries {
        let name = name.as_bytes();
        let crc = crc32(data);
        let size = u32::try_from(data.len()).unwrap();
        let offset = u32::try_from(out.len()).unwrap();

        // Local file header: bit 3 set, CRC and sizes deferred to the
        // descriptor (APPNOTE 4.3.9.4).
        u32le(&mut out, 0x0403_4b50); // local file header signature
        u16le(&mut out, 20); // version needed to extract
        u16le(&mut out, 0x0008); // general purpose bit 3: descriptor follows
        u16le(&mut out, 0); // compression method: stored
        u16le(&mut out, 0); // last mod time
        u16le(&mut out, 0); // last mod date
        u32le(&mut out, 0); // crc-32: in the descriptor
        u32le(&mut out, 0); // compressed size: in the descriptor
        u32le(&mut out, 0); // uncompressed size: in the descriptor
        u16le(&mut out, u16::try_from(name.len()).unwrap());
        u16le(&mut out, 0); // extra field length
        out.extend_from_slice(name);
        out.extend_from_slice(data);

        // Data descriptor.
        u32le(&mut out, 0x0807_4b50); // data descriptor signature `PK\x07\x08`
        u32le(&mut out, crc);
        u32le(&mut out, size); // compressed size
        u32le(&mut out, size); // uncompressed size

        // Central directory record: what the reader actually trusts.
        u32le(&mut central, 0x0201_4b50); // central directory signature
        u16le(&mut central, 20); // version made by
        u16le(&mut central, 20); // version needed to extract
        u16le(&mut central, 0x0008); // bit 3 here too
        u16le(&mut central, 0); // compression method: stored
        u16le(&mut central, 0); // last mod time
        u16le(&mut central, 0); // last mod date
        u32le(&mut central, crc);
        u32le(&mut central, size);
        u32le(&mut central, size);
        u16le(&mut central, u16::try_from(name.len()).unwrap());
        u16le(&mut central, 0); // extra field length
        u16le(&mut central, 0); // file comment length
        u16le(&mut central, 0); // disk number start
        u16le(&mut central, 0); // internal file attributes
        u32le(&mut central, 0); // external file attributes
        u32le(&mut central, offset); // local header offset
        central.extend_from_slice(name);
    }
    let count = u16::try_from(entries.len()).unwrap();
    let cd_offset = u32::try_from(out.len()).unwrap();
    let cd_size = u32::try_from(central.len()).unwrap();
    out.extend_from_slice(&central);
    u32le(&mut out, 0x0605_4b50); // end of central directory signature
    u16le(&mut out, 0); // number of this disk
    u16le(&mut out, 0); // disk with the central directory
    u16le(&mut out, count); // entries on this disk
    u16le(&mut out, count); // entries in all
    u32le(&mut out, cd_size);
    u32le(&mut out, cd_offset);
    u16le(&mut out, 0); // comment length
    out
}

fn streaming_docx() -> Vec<u8> {
    streaming_zip(&[
        ("[Content_Types].xml", TYPES.as_bytes()),
        ("_rels/.rels", RELS.as_bytes()),
        ("word/document.xml", DOC.as_bytes()),
    ])
}

#[test]
fn a_streaming_data_descriptor_docx_is_admitted() {
    let bytes = streaming_docx();
    let admitted =
        admit(&bytes, InputLimits::default()).expect("a valid data descriptor is no refusal");
    assert_eq!(admitted.main_part, "word/document.xml");
    assert_eq!(admitted.entries, 3);
    assert_eq!(
        admitted.inflated_bytes,
        (TYPES.len() + RELS.len() + DOC.len()) as u64
    );
    // The document API opens it too, under its own compare budget.
    WmlDocument::from_bytes(&bytes).expect("the package opens end to end");
}

#[test]
fn a_truncated_data_descriptor_is_refused() {
    let bytes = streaming_docx();
    // The last entry's descriptor: cut right before its signature and inside
    // its CRC field — a producer that died while streaming the last part, so
    // the file ends before any central directory exists.
    let at = bytes
        .windows(4)
        .rposition(|w| w == [0x50, 0x4b, 0x07, 0x08])
        .expect("a trailing descriptor");
    for truncated in [&bytes[..at], &bytes[..at + 6]] {
        let err = admit(truncated, InputLimits::default())
            .expect_err("a truncated descriptor leaves no end-of-central-directory record");
        assert_eq!(err.kind, AdmissionErrorKind::InvalidPackage);
        assert_eq!(err.code(), "INVALID_PACKAGE");
    }
}

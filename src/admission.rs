// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Resource admission for untrusted DOCX input: bound the ZIP container
//! before anything inflates it without limits.
//!
//! [`admit`] reads the central directory, refuses names that are not safe
//! part names, duplicates, encryption and compression other than stored or
//! deflate, then inflates every entry through a counting reader capped at the
//! remaining budget (central-directory sizes are untrusted), checks each XML
//! part's nesting depth, and confirms the package is a WordprocessingML
//! document by its relationships and content types. Nothing is extracted to
//! disk, no macro runs and no external reference is resolved.
//!
//! The agent-facing entry points ([`crate::inspect`], [`crate::edit`] and the
//! bindings over them) admit with [`InputLimits::default`]. The redline
//! comparer keeps its historical tolerance.

use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::io::{Cursor, Read};

use quick_xml::Reader;
use quick_xml::events::Event;
use zip::{CompressionMethod, ZipArchive};

/// Resource budgets. The defaults are a product policy, not a proof against
/// every denial of service.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InputLimits {
    /// Size of the ZIP file itself.
    pub max_compressed_bytes: u64,
    /// ZIP entries, directories included.
    pub max_entries: usize,
    /// Inflated size of any one entry.
    pub max_part_bytes: u64,
    /// Inflated size of all entries together.
    pub max_uncompressed_bytes: u64,
    /// Element nesting in any XML part.
    pub max_xml_depth: usize,
}

impl Default for InputLimits {
    fn default() -> Self {
        const MIB: u64 = 1024 * 1024;
        Self {
            max_compressed_bytes: 64 * MIB,
            max_entries: 10_000,
            max_part_bytes: 64 * MIB,
            max_uncompressed_bytes: 256 * MIB,
            max_xml_depth: 256,
        }
    }
}

/// Why a package was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdmissionErrorKind {
    /// A budget in [`InputLimits`] was exceeded (`INPUT_LIMIT`).
    InputLimit,
    /// Two entries name the same part (`DUPLICATE_PART`).
    DuplicatePart,
    /// Not a WordprocessingML package, an unsafe part name, encryption or an
    /// unsupported compression method (`UNSUPPORTED_PACKAGE`).
    UnsupportedPackage,
    /// Not a readable ZIP, or an entry fails its CRC (`INVALID_PACKAGE`).
    InvalidPackage,
    /// An XML part is malformed or nests too deep (`INVALID_XML`).
    InvalidXml,
}

impl AdmissionErrorKind {
    /// The stable error code agents and bindings see.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::InputLimit => "INPUT_LIMIT",
            Self::DuplicatePart => "DUPLICATE_PART",
            Self::UnsupportedPackage => "UNSUPPORTED_PACKAGE",
            Self::InvalidPackage => "INVALID_PACKAGE",
            Self::InvalidXml => "INVALID_XML",
        }
    }
}

/// A refused package: the kind and what exactly was refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdmissionError {
    /// The error class.
    pub kind: AdmissionErrorKind,
    /// Human-readable detail, naming the entry when there is one.
    pub message: String,
}

impl AdmissionError {
    fn new(kind: AdmissionErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    /// The stable error code (`INPUT_LIMIT`, …).
    #[must_use]
    pub const fn code(&self) -> &'static str {
        self.kind.code()
    }
}

impl fmt::Display for AdmissionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code(), self.message)
    }
}

impl std::error::Error for AdmissionError {}

/// Facts about an admitted package.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdmittedPackage {
    /// ZIP entries, directories included.
    pub entries: usize,
    /// Inflated bytes of all entries.
    pub inflated_bytes: u64,
    /// The main document part (`word/document.xml` in most files).
    pub main_part: String,
}

const MAIN_CONTENT_TYPES: &[&str] = &[
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml",
    "application/vnd.openxmlformats-officedocument.wordprocessingml.template.main+xml",
    "application/vnd.ms-word.document.macroEnabled.main+xml",
    "application/vnd.ms-word.template.macroEnabledTemplate.main+xml",
];

const OFFICE_DOCUMENT_REL: &[&str] = &[
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument",
    "http://purl.oclc.org/ooxml/officeDocument/relationships/officeDocument",
];

/// Admit `bytes` as a DOCX within `limits`.
///
/// # Errors
///
/// An [`AdmissionError`] naming the first budget or rule the package breaks.
pub fn admit(bytes: &[u8], limits: InputLimits) -> Result<AdmittedPackage, AdmissionError> {
    use AdmissionErrorKind as K;

    if bytes.len() as u64 > limits.max_compressed_bytes {
        return Err(AdmissionError::new(
            K::InputLimit,
            format!(
                "input is {} bytes; the limit is {}",
                bytes.len(),
                limits.max_compressed_bytes
            ),
        ));
    }
    let declared = declared_entry_count(bytes)?;
    if declared > limits.max_entries as u64 {
        return Err(AdmissionError::new(
            K::InputLimit,
            format!(
                "{declared} ZIP entries; the limit is {}",
                limits.max_entries
            ),
        ));
    }
    let mut archive = ZipArchive::new(Cursor::new(bytes))
        .map_err(|e| AdmissionError::new(K::InvalidPackage, format!("not a ZIP file: {e}")))?;
    // The reader keys entries by name, so exact duplicates collapse into one.
    if (archive.len() as u64) < declared {
        return Err(AdmissionError::new(
            K::DuplicatePart,
            "two ZIP entries have the same name",
        ));
    }

    let mut canonical = HashSet::new();
    for i in 0..archive.len() {
        let entry = archive
            .by_index_raw(i)
            .map_err(|e| AdmissionError::new(K::InvalidPackage, e.to_string()))?;
        let name = entry.name();
        check_part_name(name)?;
        if !canonical.insert(name.to_ascii_lowercase()) {
            return Err(AdmissionError::new(
                K::DuplicatePart,
                format!("{name} appears twice (part names ignore case)"),
            ));
        }
        if entry.encrypted() {
            return Err(AdmissionError::new(
                K::UnsupportedPackage,
                format!("{name} is encrypted"),
            ));
        }
        if !matches!(
            entry.compression(),
            CompressionMethod::Stored | CompressionMethod::Deflated
        ) {
            return Err(AdmissionError::new(
                K::UnsupportedPackage,
                format!(
                    "{name} uses unsupported compression {}",
                    entry.compression()
                ),
            ));
        }
        if entry.size() > limits.max_part_bytes {
            return Err(AdmissionError::new(
                K::InputLimit,
                format!(
                    "{name} declares {} inflated bytes; the part limit is {}",
                    entry.size(),
                    limits.max_part_bytes
                ),
            ));
        }
    }

    let mut total = 0u64;
    let mut kept: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| AdmissionError::new(K::InvalidPackage, e.to_string()))?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name().to_string();
        let remaining = limits.max_uncompressed_bytes - total;
        let cap = limits.max_part_bytes.min(remaining);
        let lower = name.to_ascii_lowercase();
        let is_xml = lower.ends_with(".xml") || lower.ends_with(".rels");
        let mut limited = (&mut entry).take(cap + 1);
        let read = if is_xml {
            let mut buf = Vec::new();
            limited
                .read_to_end(&mut buf)
                .map_err(|e| AdmissionError::new(K::InvalidPackage, format!("{name}: {e}")))?;
            let n = buf.len() as u64;
            if n <= cap {
                check_xml_depth(&name, &buf, limits.max_xml_depth)?;
                if lower == "[content_types].xml" || lower == "_rels/.rels" {
                    kept.insert(lower, buf);
                }
            }
            n
        } else {
            std::io::copy(&mut limited, &mut std::io::sink())
                .map_err(|e| AdmissionError::new(K::InvalidPackage, format!("{name}: {e}")))?
        };
        if read > cap {
            let which = if cap == limits.max_part_bytes {
                format!("the part limit is {}", limits.max_part_bytes)
            } else {
                format!("the package limit is {}", limits.max_uncompressed_bytes)
            };
            return Err(AdmissionError::new(
                K::InputLimit,
                format!("{name} inflates past {cap} bytes; {which}"),
            ));
        }
        total += read;
    }

    let main_part = main_part(&archive, &kept)?;
    Ok(AdmittedPackage {
        entries: archive.len(),
        inflated_bytes: total,
        main_part,
    })
}

/// The entry count the end-of-central-directory record declares (ZIP64
/// aware), which the reader's name map can undercount.
fn declared_entry_count(bytes: &[u8]) -> Result<u64, AdmissionError> {
    const EOCD: [u8; 4] = [0x50, 0x4b, 0x05, 0x06];
    const LOCATOR: [u8; 4] = [0x50, 0x4b, 0x06, 0x07];
    const EOCD64: [u8; 4] = [0x50, 0x4b, 0x06, 0x06];
    let invalid = |what: &str| {
        AdmissionError::new(
            AdmissionErrorKind::InvalidPackage,
            format!("not a ZIP file: {what}"),
        )
    };
    let floor = bytes.len().saturating_sub(22 + usize::from(u16::MAX));
    let at = (floor..=bytes.len().saturating_sub(22))
        .rev()
        .find(|&i| bytes[i..].starts_with(&EOCD))
        .ok_or_else(|| invalid("no end-of-central-directory record"))?;
    let total = u16::from_le_bytes([bytes[at + 10], bytes[at + 11]]);
    if total != u16::MAX {
        return Ok(u64::from(total));
    }
    let locator = at
        .checked_sub(20)
        .filter(|&l| bytes[l..].starts_with(&LOCATOR))
        .ok_or_else(|| invalid("ZIP64 locator missing"))?;
    let mut offset = [0u8; 8];
    offset.copy_from_slice(&bytes[locator + 8..locator + 16]);
    let record = usize::try_from(u64::from_le_bytes(offset))
        .ok()
        .filter(|&r| r + 40 <= bytes.len() && bytes[r..].starts_with(&EOCD64))
        .ok_or_else(|| invalid("ZIP64 end record missing"))?;
    let mut count = [0u8; 8];
    count.copy_from_slice(&bytes[record + 32..record + 40]);
    Ok(u64::from_le_bytes(count))
}

/// A ZIP name must be a relative part name: no leading slash or drive, no
/// backslash, no empty, `.` or `..` segment.
fn check_part_name(name: &str) -> Result<(), AdmissionError> {
    let refuse = |why: &str| {
        Err(AdmissionError::new(
            AdmissionErrorKind::UnsupportedPackage,
            format!("unsafe entry name {name:?}: {why}"),
        ))
    };
    if name.is_empty() {
        return refuse("empty");
    }
    if name.starts_with('/') || name.as_bytes().get(1) == Some(&b':') {
        return refuse("absolute path");
    }
    if name.contains('\\') || name.contains('\0') {
        return refuse("backslash or NUL");
    }
    let segments = name.strip_suffix('/').unwrap_or(name);
    if segments
        .split('/')
        .any(|s| s.is_empty() || s == "." || s == "..")
    {
        return refuse("empty, . or .. segment");
    }
    Ok(())
}

/// Well-formed enough to scan, and no deeper than `max_depth` elements.
fn check_xml_depth(name: &str, xml: &[u8], max_depth: usize) -> Result<(), AdmissionError> {
    let invalid =
        |why: String| AdmissionError::new(AdmissionErrorKind::InvalidXml, format!("{name}: {why}"));
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().check_end_names = false;
    let mut buf = Vec::new();
    let mut depth = 0usize;
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(_)) => {
                depth += 1;
                if depth > max_depth {
                    return Err(invalid(format!("XML nests deeper than {max_depth}")));
                }
            }
            Ok(Event::End(_)) => depth = depth.saturating_sub(1),
            Ok(Event::Eof) => return Ok(()),
            Ok(_) => {}
            Err(e) => return Err(invalid(e.to_string())),
        }
        buf.clear();
    }
}

/// The main document part from the package relationships (falling back to
/// `word/document.xml`, as the reader does), checked against its declared
/// content type.
fn main_part<R: Read + std::io::Seek>(
    archive: &ZipArchive<R>,
    kept: &BTreeMap<String, Vec<u8>>,
) -> Result<String, AdmissionError> {
    use AdmissionErrorKind as K;
    let types = kept
        .get("[content_types].xml")
        .ok_or_else(|| AdmissionError::new(K::UnsupportedPackage, "no [Content_Types].xml"))?;
    let from_rels = kept.get("_rels/.rels").and_then(|rels| {
        elements(rels, b"Relationship")
            .into_iter()
            .find_map(|attrs| {
                let is_main = attrs
                    .get("Type")
                    .is_some_and(|t| OFFICE_DOCUMENT_REL.contains(&t.as_str()));
                let external = attrs.get("TargetMode").is_some_and(|m| m == "External");
                (is_main && !external)
                    .then(|| {
                        attrs
                            .get("Target")
                            .map(|t| t.trim_start_matches('/').to_string())
                    })
                    .flatten()
            })
    });
    let main = from_rels.unwrap_or_else(|| "word/document.xml".to_string());
    if archive.index_for_name(&main).is_none() {
        return Err(AdmissionError::new(
            K::UnsupportedPackage,
            format!("main document part {main} is missing"),
        ));
    }
    if let Some(content_type) = content_type_of(types, &main)
        && !MAIN_CONTENT_TYPES.contains(&content_type.as_str())
    {
        return Err(AdmissionError::new(
            K::UnsupportedPackage,
            format!("{main} is {content_type}, not a Word document"),
        ));
    }
    Ok(main)
}

/// The content type `[Content_Types].xml` declares for `part`: its Override,
/// else the Default for its extension.
fn content_type_of(types: &[u8], part: &str) -> Option<String> {
    let wanted = format!("/{part}");
    let overridden = elements(types, b"Override").into_iter().find_map(|attrs| {
        attrs
            .get("PartName")
            .filter(|p| p.eq_ignore_ascii_case(&wanted))
            .and_then(|_| attrs.get("ContentType").cloned())
    });
    overridden.or_else(|| {
        let ext = part.rsplit_once('.')?.1;
        elements(types, b"Default").into_iter().find_map(|attrs| {
            attrs
                .get("Extension")
                .filter(|e| e.eq_ignore_ascii_case(ext))
                .and_then(|_| attrs.get("ContentType").cloned())
        })
    })
}

/// Attributes of every element with local name `local` (namespace ignored).
fn elements(xml: &[u8], local: &[u8]) -> Vec<BTreeMap<String, String>> {
    let mut reader = Reader::from_reader(xml);
    let mut buf = Vec::new();
    let mut found = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e) | Event::Empty(e)) if e.local_name().as_ref() == local => {
                let attrs = e
                    .attributes()
                    .flatten()
                    .filter_map(|a| {
                        let key = String::from_utf8(a.key.local_name().as_ref().to_vec()).ok()?;
                        let value = a
                            .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                            .ok()?
                            .into_owned();
                        Some((key, value))
                    })
                    .collect();
                found.push(attrs);
            }
            Ok(Event::Eof) | Err(_) => return found,
            Ok(_) => {}
        }
        buf.clear();
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use zip::write::SimpleFileOptions;

    use super::*;

    const TYPES: &str = r#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#;
    const RELS: &str = r#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;
    const DOC: &str = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Hi</w:t></w:r></w:p></w:body></w:document>"#;

    fn zip_of(entries: &[(&str, &[u8])], method: CompressionMethod) -> Vec<u8> {
        let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default().compression_method(method);
        for (name, data) in entries {
            out.start_file(*name, options).unwrap();
            out.write_all(data).unwrap();
        }
        out.finish().unwrap().into_inner()
    }

    fn docx_with(extra: &[(&str, &[u8])]) -> Vec<u8> {
        let mut entries: Vec<(&str, &[u8])> = vec![
            ("[Content_Types].xml", TYPES.as_bytes()),
            ("_rels/.rels", RELS.as_bytes()),
            ("word/document.xml", DOC.as_bytes()),
        ];
        entries.extend_from_slice(extra);
        zip_of(&entries, CompressionMethod::Deflated)
    }

    fn kind(bytes: &[u8], limits: InputLimits) -> AdmissionErrorKind {
        admit(bytes, limits).expect_err("refused").kind
    }

    /// Overwrite the 2-byte field at `local_off` / `central_off` in every
    /// local and central header.
    fn patch_u16(bytes: &mut [u8], local_off: usize, central_off: usize, value: u16) {
        patch(bytes, local_off, central_off, &value.to_le_bytes());
    }

    fn patch(bytes: &mut [u8], local_off: usize, central_off: usize, value: &[u8]) {
        let mut i = 0;
        while i + 4 <= bytes.len() {
            let off = match &bytes[i..i + 4] {
                [0x50, 0x4b, 0x03, 0x04] => Some(local_off),
                [0x50, 0x4b, 0x01, 0x02] => Some(central_off),
                _ => None,
            };
            if let Some(off) = off {
                bytes[i + off..i + off + value.len()].copy_from_slice(value);
            }
            i += 1;
        }
    }

    #[test]
    fn a_plain_docx_is_admitted() {
        let bytes = docx_with(&[]);
        let admitted = admit(&bytes, InputLimits::default()).unwrap();
        assert_eq!(admitted.main_part, "word/document.xml");
        assert_eq!(admitted.entries, 3);
        assert_eq!(
            admitted.inflated_bytes,
            (TYPES.len() + RELS.len() + DOC.len()) as u64
        );
    }

    #[test]
    fn budgets_admit_their_exact_boundary() {
        let bytes = docx_with(&[]);
        let total = (TYPES.len() + RELS.len() + DOC.len()) as u64;
        let largest = TYPES.len().max(RELS.len()).max(DOC.len()) as u64;
        let exact = InputLimits {
            max_compressed_bytes: bytes.len() as u64,
            max_entries: 3,
            max_part_bytes: largest,
            max_uncompressed_bytes: total,
            max_xml_depth: 5,
        };
        assert!(admit(&bytes, exact).is_ok());
        for tighter in [
            InputLimits {
                max_compressed_bytes: bytes.len() as u64 - 1,
                ..exact
            },
            InputLimits {
                max_entries: 2,
                ..exact
            },
            InputLimits {
                max_part_bytes: largest - 1,
                ..exact
            },
            InputLimits {
                max_uncompressed_bytes: total - 1,
                ..exact
            },
        ] {
            assert_eq!(kind(&bytes, tighter), AdmissionErrorKind::InputLimit);
        }
        let shallow = InputLimits {
            max_xml_depth: 4,
            ..exact
        };
        assert_eq!(kind(&bytes, shallow), AdmissionErrorKind::InvalidXml);
    }

    #[test]
    fn a_highly_compressed_part_stops_at_the_budget() {
        // 32 MiB of zeros deflate to ~32 KiB; written in chunks, never held.
        let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        for (name, data) in [
            ("[Content_Types].xml", TYPES),
            ("_rels/.rels", RELS),
            ("word/document.xml", DOC),
        ] {
            out.start_file(name, options).unwrap();
            out.write_all(data.as_bytes()).unwrap();
        }
        out.start_file("word/media/zeros.bin", options).unwrap();
        let chunk = vec![0u8; 1 << 20];
        for _ in 0..32 {
            out.write_all(&chunk).unwrap();
        }
        let bytes = out.finish().unwrap().into_inner();
        assert!(bytes.len() < 1 << 20);
        let tiny = InputLimits {
            max_part_bytes: 1 << 20,
            ..InputLimits::default()
        };
        let refused = admit(&bytes, tiny).unwrap_err();
        assert_eq!(refused.kind, AdmissionErrorKind::InputLimit);
        assert!(refused.message.contains("zeros.bin"), "{refused}");
        let package = InputLimits {
            max_uncompressed_bytes: 1 << 20,
            ..InputLimits::default()
        };
        assert_eq!(kind(&bytes, package), AdmissionErrorKind::InputLimit);
    }

    #[test]
    fn an_understated_size_is_caught_while_inflating() {
        let big = vec![b'a'; 4096];
        let mut bytes = docx_with(&[("word/media/a.bin", &big)]);
        // Declare 16 inflated bytes for every entry (local +22, central +24).
        patch(&mut bytes, 22, 24, &16u32.to_le_bytes());
        let limits = InputLimits {
            max_part_bytes: 1024,
            ..InputLimits::default()
        };
        let refused = admit(&bytes, limits).unwrap_err();
        assert!(
            matches!(
                refused.kind,
                AdmissionErrorKind::InputLimit | AdmissionErrorKind::InvalidPackage
            ),
            "{refused}"
        );
    }

    #[test]
    fn duplicate_names_are_refused() {
        // Same length names, renamed in place: the reader would keep one.
        let mut bytes = docx_with(&[("word/aaaa.xml", b"<a/>"), ("word/bbbb.xml", b"<b/>")]);
        let at: Vec<usize> = bytes
            .windows(13)
            .enumerate()
            .filter(|(_, w)| *w == b"word/bbbb.xml")
            .map(|(i, _)| i)
            .collect();
        assert_eq!(at.len(), 2, "local and central name");
        for i in at {
            bytes[i..i + 13].copy_from_slice(b"word/aaaa.xml");
        }
        assert_eq!(
            kind(&bytes, InputLimits::default()),
            AdmissionErrorKind::DuplicatePart
        );
        // Part names ignore case.
        let cased = docx_with(&[("Word/Document.xml", DOC.as_bytes())]);
        assert_eq!(
            kind(&cased, InputLimits::default()),
            AdmissionErrorKind::DuplicatePart
        );
    }

    #[test]
    fn unsafe_names_are_refused() {
        for name in [
            "../evil.xml",
            "word/../../evil.xml",
            "/abs.xml",
            "C:/abs.xml",
            "word\\x.xml",
            "word//x.xml",
            "word/./x.xml",
        ] {
            let bytes = docx_with(&[(name, b"<x/>")]);
            assert_eq!(
                kind(&bytes, InputLimits::default()),
                AdmissionErrorKind::UnsupportedPackage,
                "{name}"
            );
        }
        // A directory entry is fine.
        let mut out = zip::ZipWriter::new_append(Cursor::new(docx_with(&[]))).unwrap();
        out.add_directory("word/media/", SimpleFileOptions::default())
            .unwrap();
        let bytes = out.finish().unwrap().into_inner();
        assert!(admit(&bytes, InputLimits::default()).is_ok());
    }

    #[test]
    fn encryption_and_other_compression_are_refused() {
        let mut encrypted = docx_with(&[]);
        // General-purpose flag bit 0 (local +6, central +8).
        patch_u16(&mut encrypted, 6, 8, 1);
        assert_eq!(
            kind(&encrypted, InputLimits::default()),
            AdmissionErrorKind::UnsupportedPackage
        );
        let mut lzma = docx_with(&[]);
        // Compression method 14 = LZMA (local +8, central +10).
        patch_u16(&mut lzma, 8, 10, 14);
        assert_eq!(
            kind(&lzma, InputLimits::default()),
            AdmissionErrorKind::UnsupportedPackage
        );
        let stored = zip_of(
            &[
                ("[Content_Types].xml", TYPES.as_bytes()),
                ("_rels/.rels", RELS.as_bytes()),
                ("word/document.xml", DOC.as_bytes()),
            ],
            CompressionMethod::Stored,
        );
        assert!(admit(&stored, InputLimits::default()).is_ok());
    }

    #[test]
    fn a_corrupt_entry_is_refused() {
        let mut bytes = zip_of(
            &[
                ("[Content_Types].xml", TYPES.as_bytes()),
                ("_rels/.rels", RELS.as_bytes()),
                ("word/document.xml", DOC.as_bytes()),
            ],
            CompressionMethod::Stored,
        );
        let at = bytes
            .windows(2)
            .position(|w| w == b"Hi")
            .expect("stored text");
        bytes[at] = b'X';
        assert_eq!(
            kind(&bytes, InputLimits::default()),
            AdmissionErrorKind::InvalidPackage
        );
        assert_eq!(
            kind(b"not a zip at all", InputLimits::default()),
            AdmissionErrorKind::InvalidPackage
        );
    }

    #[test]
    fn a_package_that_is_not_a_word_document_is_refused() {
        let no_main = zip_of(
            &[
                ("[Content_Types].xml", TYPES.as_bytes()),
                ("_rels/.rels", RELS.as_bytes()),
            ],
            CompressionMethod::Deflated,
        );
        assert_eq!(
            kind(&no_main, InputLimits::default()),
            AdmissionErrorKind::UnsupportedPackage
        );
        let no_types = zip_of(
            &[
                ("_rels/.rels", RELS.as_bytes()),
                ("word/document.xml", DOC.as_bytes()),
            ],
            CompressionMethod::Deflated,
        );
        assert_eq!(
            kind(&no_types, InputLimits::default()),
            AdmissionErrorKind::UnsupportedPackage
        );
        // A spreadsheet's main part, named like a Word one: the content type decides.
        let sheet = TYPES.replace(
            "wordprocessingml.document.main+xml",
            "spreadsheetml.sheet.main+xml",
        );
        let xlsx = zip_of(
            &[
                ("[Content_Types].xml", sheet.as_bytes()),
                ("_rels/.rels", RELS.as_bytes()),
                ("word/document.xml", DOC.as_bytes()),
            ],
            CompressionMethod::Deflated,
        );
        assert_eq!(
            kind(&xlsx, InputLimits::default()),
            AdmissionErrorKind::UnsupportedPackage
        );
    }

    #[test]
    fn the_main_part_follows_the_relationship() {
        let rels = RELS.replace("word/document.xml", "/content/main.xml");
        let types = TYPES.replace("/word/document.xml", "/content/main.xml");
        let bytes = zip_of(
            &[
                ("[Content_Types].xml", types.as_bytes()),
                ("_rels/.rels", rels.as_bytes()),
                ("content/main.xml", DOC.as_bytes()),
            ],
            CompressionMethod::Deflated,
        );
        assert_eq!(
            admit(&bytes, InputLimits::default()).unwrap().main_part,
            "content/main.xml"
        );
    }

    #[test]
    fn deep_or_malformed_xml_is_refused() {
        let deep = format!("{}{}", "<a>".repeat(300), "</a>".repeat(300));
        let bytes = docx_with(&[("word/deep.xml", deep.as_bytes())]);
        assert_eq!(
            kind(&bytes, InputLimits::default()),
            AdmissionErrorKind::InvalidXml
        );
        let broken = docx_with(&[("word/broken.xml", b"<a><b attr=\"x></a>")]);
        assert_eq!(
            kind(&broken, InputLimits::default()),
            AdmissionErrorKind::InvalidXml
        );
    }

    #[test]
    fn error_codes_are_stable() {
        use AdmissionErrorKind as K;
        let codes: Vec<_> = [
            K::InputLimit,
            K::DuplicatePart,
            K::UnsupportedPackage,
            K::InvalidPackage,
            K::InvalidXml,
        ]
        .map(K::code)
        .into();
        assert_eq!(
            codes,
            [
                "INPUT_LIMIT",
                "DUPLICATE_PART",
                "UNSUPPORTED_PACKAGE",
                "INVALID_PACKAGE",
                "INVALID_XML"
            ]
        );
        let e = AdmissionError::new(K::InputLimit, "x");
        assert_eq!(e.to_string(), "INPUT_LIMIT: x");
    }
}

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::io::{Read, Seek, SeekFrom};

use zip::ZipArchive;
use zip::read::{ArchiveOffset, Config};
use zip::result::{ZipError, ZipResult};

/// Open at offset zero first, retaining support for prepended data —
/// APPNOTE 4.3.6 lets a self-extracting archive carry a stub ahead of
/// the first local header, and jubarte admits one.
///
/// The unchecked opener: like [`ZipArchive::new`] it trusts the central
/// directory and never looks at local file headers. `jubarte debug` — the
/// maintainer triage tool — uses it so a package every other route
/// refuses can still be inspected, and `admit` uses it so its own
/// consistency check can refuse with a typed code.
pub(crate) fn open_archive_unchecked<R: Read + Seek + Clone>(
    reader: R,
) -> ZipResult<ZipArchive<R>> {
    // zip 9's offset detection scans the payload even for ordinary packages.
    let config = Config {
        archive_offset: ArchiveOffset::Known(0),
    };
    if let Ok(archive) = ZipArchive::with_config(config, reader.clone()) {
        Ok(archive)
    } else {
        ZipArchive::new(reader)
    }
}

/// The package open every reader shares: [`open_archive_unchecked`], then
/// [`zip_defects`]. A package whose two headers describe an entry
/// differently — a dual-header "zip cloaking" package,
/// GHSA-m665-v7pm-7grj — whose entries overlap or reach into the
/// central directory, or that records an entry as a non-regular type
/// fails the open. The error carries `admit`'s typed
/// [`crate::admission::AdmissionError`] as its source, so the refusal
/// code reads the same whichever route met the file.
pub(crate) fn open_archive<R: Read + Seek + Clone>(reader: R) -> ZipResult<ZipArchive<R>> {
    open_archive_within(reader, true)
}

/// [`open_archive`] without the duplicate-part refusal: [`PartFs`] refuses
/// the same identity itself, with the typed `OpcError::DuplicatePartName`
/// its callers match on (ZIP-15/#600); every other defect still refuses
/// here.
pub(crate) fn open_archive_within<R: Read + Seek + Clone>(
    reader: R,
    refuse_duplicates: bool,
) -> ZipResult<ZipArchive<R>> {
    let mut headers = reader.clone();
    let archive = open_archive_unchecked(reader)?;
    if let Some(defect) = zip_defects(&archive, &mut headers)
        .into_iter()
        .find(|defect| refuse_duplicates || defect.kind != ZipDefectKind::DuplicatePart)
    {
        return Err(ZipError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            defect.refusal(),
        )));
    }
    Ok(archive)
}

/// A header's signature and the offsets of the fields the structural
/// pass reads: general-purpose flags, compression method, CRC-32, the
/// two sizes, and the name and extra-field lengths. The central record
/// (`PK\1\2`, APPNOTE 4.3.12) and the local file header (`PK\3\4`,
/// APPNOTE 4.3.7) lay the same fields out at different offsets; the
/// central-only fields — `version_made_by` and the external attributes —
/// are `None` offsets on the local shape.
struct HeaderShape {
    signature: &'static [u8; 4],
    version_made_by: Option<usize>,
    flags: usize,
    method: usize,
    crc32: usize,
    compressed_size: usize,
    uncompressed_size: usize,
    name_length: usize,
    extra_length: usize,
    external_attributes: Option<usize>,
    /// The central record's file-comment length; a local header has none.
    comment_length: Option<usize>,
    name: usize,
}

const CENTRAL_HEADER: HeaderShape = HeaderShape {
    signature: b"PK\x01\x02",
    version_made_by: Some(4),
    flags: 8,
    method: 10,
    crc32: 16,
    compressed_size: 20,
    uncompressed_size: 24,
    name_length: 28,
    extra_length: 30,
    external_attributes: Some(38),
    comment_length: Some(32),
    name: 46,
};
const LOCAL_HEADER: HeaderShape = HeaderShape {
    signature: b"PK\x03\x04",
    version_made_by: None,
    flags: 6,
    method: 8,
    crc32: 14,
    compressed_size: 18,
    uncompressed_size: 22,
    name_length: 26,
    extra_length: 28,
    external_attributes: None,
    comment_length: None,
    name: 30,
};

/// The fields of one record, parsed in place: the bytes the structural
/// pass compares, plus the raw name and extra bytes.
struct HeaderFields {
    version_made_by: Option<u16>,
    flags: u16,
    method: u16,
    crc32: u32,
    compressed_size: u32,
    uncompressed_size: u32,
    name_length: u16,
    extra_length: u16,
    external_attributes: Option<u32>,
    comment_length: u16,
    name: Vec<u8>,
    extra: Vec<u8>,
}

fn read_u16(head: &[u8], at: usize) -> Option<u16> {
    head.get(at..at.checked_add(2)?)
        .and_then(|field| <[u8; 2]>::try_from(field).ok())
        .map(u16::from_le_bytes)
}

fn read_u32(head: &[u8], at: usize) -> Option<u32> {
    head.get(at..at.checked_add(4)?)
        .and_then(|field| <[u8; 4]>::try_from(field).ok())
        .map(u32::from_le_bytes)
}

/// The fields of the header at `at`, or `None` when no header of that
/// shape is there. A length that overruns the reader truncates instead
/// of failing, so a malformed variable part is still positioned.
fn header_fields<R: Read + Seek>(
    reader: &mut R,
    at: u64,
    shape: &HeaderShape,
) -> Option<HeaderFields> {
    let mut head = vec![0u8; shape.name];
    reader.seek(SeekFrom::Start(at)).ok()?;
    reader.read_exact(&mut head).ok()?;
    if head.get(..4)? != shape.signature.as_slice() {
        return None;
    }
    let mut variable = |length_at: usize| -> Vec<u8> {
        let Some(length) = read_u16(&head, length_at) else {
            return Vec::new();
        };
        let mut bytes = vec![0; usize::from(length)];
        let mut filled = 0usize;
        while let Some(tail) = bytes.get_mut(filled..)
            && !tail.is_empty()
        {
            match reader.read(tail) {
                Ok(0) | Err(_) => break,
                Ok(n) => filled = filled.saturating_add(n),
            }
        }
        bytes.truncate(filled);
        bytes
    };
    let name = variable(shape.name_length);
    let extra = variable(shape.extra_length);
    Some(HeaderFields {
        version_made_by: shape.version_made_by.and_then(|at| read_u16(&head, at)),
        flags: read_u16(&head, shape.flags)?,
        method: read_u16(&head, shape.method)?,
        crc32: read_u32(&head, shape.crc32)?,
        compressed_size: read_u32(&head, shape.compressed_size)?,
        uncompressed_size: read_u32(&head, shape.uncompressed_size)?,
        name_length: read_u16(&head, shape.name_length)?,
        extra_length: read_u16(&head, shape.extra_length)?,
        external_attributes: shape.external_attributes.and_then(|at| read_u32(&head, at)),
        comment_length: shape
            .comment_length
            .and_then(|at| read_u16(&head, at))
            .unwrap_or(0),
        name,
        extra,
    })
}

/// The data of the first 0x0001 (ZIP64 extended information) field in
/// `extra`, of any length. `None` when no field carries the tag or an
/// earlier field's declared length runs past the block.
fn zip64_field(extra: &[u8]) -> Option<&[u8]> {
    let mut at = 0usize;
    while at.checked_add(4).is_some_and(|end| end <= extra.len()) {
        let tag = read_u16(extra, at)?;
        let length = usize::from(read_u16(extra, at.checked_add(2)?)?);
        let data = at.checked_add(4)?;
        let end = data.checked_add(length)?;
        if end > extra.len() {
            return None;
        }
        if tag == 0x0001 {
            return extra.get(data..end);
        }
        at = end;
    }
    None
}

/// The `(uncompressed, compressed)` sizes a local header's 0x0001
/// (ZIP64 extended information) extra field carries — always both, in
/// that order (APPNOTE 4.5.3). A malformed or short field yields `None`.
fn zip64_sizes(extra: &[u8]) -> Option<(u64, u64)> {
    let field = zip64_field(extra)?;
    let size = |at: usize| {
        field
            .get(at..at.checked_add(8)?)
            .and_then(|f| <[u8; 8]>::try_from(f).ok())
            .map(u64::from_le_bytes)
    };
    Some((size(0)?, size(8)?))
}

/// The size a local header's `field` stands for: itself, or the local
/// ZIP64 extra field's value when it is the `0xFFFFFFFF` placeholder
/// (`which` 0 = uncompressed, 1 = compressed). `None` when the
/// placeholder has no resolvable field.
fn resolved_local_size(field: u32, extra: &[u8], which: usize) -> Option<u64> {
    if field != u32::MAX {
        return Some(u64::from(field));
    }
    let (uncompressed, compressed) = zip64_sizes(extra)?;
    Some(if which == 0 { uncompressed } else { compressed })
}

/// The length of the data descriptor at `at` — the bytes a bit-3 entry
/// trails its payload with — when exactly one interpretation matches the
/// central record. APPNOTE 4.3.9.3 permits the optional signature, so a
/// first word equal to `0x08074b50` is tried as both signature and CRC;
/// any other first word can only be the CRC. `zip64` prescribes 8-byte
/// sizes (APPNOTE 4.3.9.2), but some writers pick the other width, so
/// both are tried — the CRC and both sizes must still match, and the
/// prescribed width wins when both do. An unreadable, non-matching or
/// ambiguous triple is a defect.
fn descriptor_extent<R: Read + Seek>(
    reader: &mut R,
    at: u64,
    zip64: bool,
    crc32: u32,
    compressed: u64,
    uncompressed: u64,
) -> Result<u64, &'static str> {
    const SIGNATURE: u32 = 0x0807_4b50;
    let mut buf = [0u8; 28];
    reader
        .seek(SeekFrom::Start(at))
        .map_err(|_| "has no readable data descriptor")?;
    let mut got = 0usize;
    while let Some(tail) = buf.get_mut(got..24) {
        match reader.read(tail) {
            Ok(0) => break,
            Ok(n) => got = got.saturating_add(n),
            Err(_) => return Err("has no readable data descriptor"),
        }
    }
    if got < 12 {
        return Err("has a truncated data descriptor");
    }
    let filled = buf.get(..got).unwrap_or_default();
    let word = |at: usize| -> Option<u32> { read_u32(filled, at) };
    let qword = |at: usize| -> Option<u64> {
        filled
            .get(at..at.checked_add(8)?)
            .and_then(|field| <[u8; 8]>::try_from(field).ok())
            .map(u64::from_le_bytes)
    };
    // The extents each size width admits, prescribed width first. Both
    // widths matching picks the prescribed one; neither refuses.
    let mut extents = Vec::with_capacity(2);
    for wide in [zip64, !zip64] {
        let need = if wide { 24 } else { 16 };
        let triple = |at: usize| -> bool {
            let Some(crc) = word(at) else {
                return false;
            };
            let (c, u) = if wide {
                (qword(at.saturating_add(4)), qword(at.saturating_add(12)))
            } else {
                (
                    word(at.saturating_add(4)).map(u64::from),
                    word(at.saturating_add(8)).map(u64::from),
                )
            };
            crc == crc32 && c == Some(compressed) && u == Some(uncompressed)
        };
        let unsigned = triple(0);
        let signed = word(0) == Some(SIGNATURE) && got >= need && triple(4);
        match (unsigned, signed) {
            (true, false) => extents.push(if wide { 20 } else { 12 }),
            (false, true) => extents.push(if wide { 24 } else { 16 }),
            (false, false) => {}
            (true, true) => return Err("has an ambiguous data descriptor"),
        }
    }
    match extents.first() {
        Some(&extent) => Ok(extent),
        None => Err("has no data descriptor matching its central record"),
    }
}

/// A structural defect in a package: the class that decides the refusal
/// code, plus the detail naming the entry and what was wrong.
#[derive(Debug)]
pub(crate) struct ZipDefect {
    /// The defect class.
    pub kind: ZipDefectKind,
    /// What disagreed, naming the entry and both values.
    pub detail: String,
}

/// The class of a [`ZipDefect`]. `HeaderMismatch` is the dual-header
/// class (`ZIP_HEADER_MISMATCH`, GHSA-m665-v7pm-7grj and the #644 field
/// comparison); `Extent` and `Descriptor` are the structural classes
/// (`INVALID_PACKAGE`); `SpecialEntry` is a non-regular entry type
/// (`UNSUPPORTED_PACKAGE`, ZIP-16/#639); `DirectoryPayload` and
/// `EntryRead` are unreadable payloads and unopenable entries, which
/// only `debug` constructs — every reading route refuses them through
/// its own CRC stream or open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ZipDefectKind {
    /// A local file header field disagrees with the central record.
    HeaderMismatch,
    /// An entry's extent overlaps another or reaches the central directory.
    Extent,
    /// A bit-3 entry's data descriptor is missing, truncated or wrong.
    Descriptor,
    /// An entry is recorded as a non-regular type — a link, a device, a
    /// pipe — or a directory on a name without a trailing `/`.
    SpecialEntry,
    /// A directory entry's payload cannot be read (`debug` findings only).
    DirectoryPayload,
    /// An entry cannot be opened or its payload read (`debug` findings
    /// only): a `debug` report keeps going where a reading route
    /// refuses, so the failure is named for the entry instead (#644).
    EntryRead,
    /// Two central records name one OPC part — byte-equal or under
    /// ASCII case (ZIP-15/#600) — which the reader's name index would
    /// collapse, silently keeping the second copy.
    DuplicatePart,
}

impl ZipDefect {
    /// The typed refusal this defect produces on a reading route.
    pub(crate) fn refusal(&self) -> crate::admission::AdmissionError {
        use crate::admission::{AdmissionError, AdmissionErrorKind as K};
        match self.kind {
            ZipDefectKind::HeaderMismatch => AdmissionError::new(
                K::ZipHeaderMismatch,
                format!("local and central headers disagree: {}", self.detail),
            ),
            ZipDefectKind::Extent
            | ZipDefectKind::Descriptor
            | ZipDefectKind::DirectoryPayload
            | ZipDefectKind::EntryRead => {
                AdmissionError::new(K::InvalidPackage, self.detail.clone())
            }
            ZipDefectKind::SpecialEntry => {
                AdmissionError::new(K::UnsupportedPackage, self.detail.clone())
            }
            ZipDefectKind::DuplicatePart => {
                AdmissionError::new(K::DuplicatePart, self.detail.clone())
            }
        }
    }

    /// The `debug` finding kind this defect reports as.
    pub(crate) fn finding_kind(&self) -> &'static str {
        match self.kind {
            ZipDefectKind::HeaderMismatch => "zip-header-mismatch",
            ZipDefectKind::Extent => "zip-extent",
            ZipDefectKind::Descriptor => "zip-descriptor",
            ZipDefectKind::SpecialEntry => "zip-special-entry",
            ZipDefectKind::DirectoryPayload => "zip-directory-payload",
            ZipDefectKind::EntryRead => "zip-entry-read",
            ZipDefectKind::DuplicatePart => "zip-duplicate-part",
        }
    }
}

/// The general-purpose flag bits the two records must agree on: bit 0
/// (encrypted) and bit 3 (data descriptor), always, plus bit 11 (UTF-8
/// names) only when a raw name byte is 0x80 or above — for an ASCII
/// name the bit changes no decoding, so the one-sided flag real
/// Word-opened files carry is not a mismatch. The deflate option bits
/// and the reserved bits are not compared — writers set them in only
/// one header — and neither are the modification time, the date nor any
/// extra field, which real Word-opened files carry differing (#594, zip
/// 9.0.5's stricter check).
const COMPARED_FLAGS: u16 = 0x0001 | 0x0008 | 0x0800;

/// Bit 11, the UTF-8-name flag: compared only when either raw name
/// holds a byte at or above 0x80 (see [`COMPARED_FLAGS`]).
const UTF8_FLAG: u16 = 0x0800;

/// The entry type recorded in the central record (ZIP-16/#639). When
/// the version-made-by host byte is 3 (Unix) or 19 (OS X), the Unix
/// mode is the high 16 bits of the external attributes and its type is
/// `mode & 0o170000`: type 0 (none recorded) and `0o100000` (regular)
/// admit, `0o040000` admits only on a name ending in `/`, and every
/// other type — link, device, pipe, socket — is refused. On any host,
/// the DOS directory attribute `0x10` on a non-slash name is refused.
/// Permission and owner bits never matter. The raw fields are read
/// rather than zip's `is_symlink`, which is a bit-mask test that reads
/// a mode only for the Unix host.
fn special_entry(index: usize, name: &[u8], central: &HeaderFields) -> Option<ZipDefect> {
    let shown = String::from_utf8_lossy(name).into_owned();
    let slash = name.ends_with(b"/");
    let defect = |detail: String| ZipDefect {
        kind: ZipDefectKind::SpecialEntry,
        detail,
    };
    let external = central.external_attributes.unwrap_or(0);
    if matches!(central.version_made_by.unwrap_or(0) >> 8, 3 | 19) {
        // The entry type is the mode's file-type bits; the permission,
        // setuid, setgid and sticky bits below it never matter.
        let kind = external >> 16 & 0o170000;
        match kind {
            0 | 0o100000 => {}
            0o040000 if slash => {}
            0o040000 => {
                return Some(defect(format!(
                    "entry {index} ('{shown}') is a directory on a name without a trailing '/'"
                )));
            }
            _ => {
                let what = match kind {
                    0o010000 => "FIFO".to_string(),
                    0o020000 => "character device".to_string(),
                    0o060000 => "block device".to_string(),
                    0o120000 => "symlink".to_string(),
                    0o140000 => "socket".to_string(),
                    other => format!("Unix type 0o{other:o}"),
                };
                return Some(defect(format!("entry {index} ('{shown}') is a {what}")));
            }
        }
    }
    if external & 0x10 != 0 && !slash {
        return Some(defect(format!(
            "entry {index} ('{shown}') is a directory on a name without a trailing '/'"
        )));
    }
    None
}

/// Every structural disagreement between what the central directory
/// declares and what the package's bytes carry, in archive order
/// (GHSA-m665-v7pm-7grj, ZIP-17/#644):
///
/// - the type the central record's version-made-by host byte and
///   external attributes record: only a regular file, or a directory on
///   a `/`-terminated name, is a part payload (ZIP-16/#639);
///
/// - the raw entry name plus the fields a scanner trusting local headers
///   would read — compression method, the [`COMPARED_FLAGS`] bits, and,
///   with bit 3 clear, the CRC-32 and the resolved (ZIP64-aware) sizes.
///   With bit 3 set, zero placeholders or matching final values are
///   allowed, since the descriptor carries them;
/// - the entry's full extent — local header, name, extras, compressed
///   payload and any data descriptor — which must end before the central
///   directory and intersect no other entry's;
/// - the descriptor itself, whose CRC and sizes must match the central
///   record, signed or unsigned, in the width the local header's ZIP64
///   form prescribes — a 0x0001 extra field alone prescribes 8-byte
///   sizes (APPNOTE 4.3.9.2) — or the other width when only it matches.
///
/// The raw bytes are read from `reader`, the source [`ZipArchive`]
/// indexed, because zip resolves an entry's name through a central
/// Unicode Path field. A local time or extra field disagreement is not
/// reported: it hides no content and Word opens such packages. An entry
/// zip cannot index is left to the read that will refuse it.
pub(crate) fn zip_defects<R: Read + Seek>(
    archive: &ZipArchive<R>,
    reader: &mut R,
) -> Vec<ZipDefect> {
    let mut defects = Vec::new();
    let mut extents: Vec<(usize, String, u64, u64)> = Vec::with_capacity(archive.len());
    for i in 0..archive.len() {
        // The central metadata only: `by_index_raw` would read the local
        // header itself and fail on the very entry this check must name.
        let Ok(entry) = archive.by_index_data(i) else {
            continue;
        };
        let (central_at, local_at) = (entry.central_header_start(), entry.header_start());
        let Some(central) = header_fields(reader, central_at, &CENTRAL_HEADER) else {
            defects.push(ZipDefect {
                kind: ZipDefectKind::HeaderMismatch,
                detail: format!("entry {i} has no central record at offset {central_at}"),
            });
            continue;
        };
        let shown = String::from_utf8_lossy(&central.name).into_owned();
        if let Some(defect) = special_entry(i, &central.name, &central) {
            defects.push(defect);
        }
        let mismatch = |detail: String| ZipDefect {
            kind: ZipDefectKind::HeaderMismatch,
            detail,
        };
        let Some(local) = header_fields(reader, local_at, &LOCAL_HEADER) else {
            defects.push(mismatch(format!(
                "entry {i} ('{shown}') has no local header at offset {local_at}"
            )));
            continue;
        };
        if local.name != central.name {
            defects.push(mismatch(format!(
                "entry {i} is '{shown}' in the central directory but '{}' in its local header",
                String::from_utf8_lossy(&local.name)
            )));
        }
        if local.method != central.method {
            defects.push(mismatch(format!(
                "entry {i} ('{shown}') has compression method {} in the central directory but {} in its local header",
                central.method, local.method
            )));
        }
        let compared = COMPARED_FLAGS
            & if central.name.iter().chain(&local.name).any(|&b| b >= 0x80) {
                u16::MAX
            } else {
                !UTF8_FLAG
            };
        let (central_flags, local_flags) = (central.flags & compared, local.flags & compared);
        if central_flags != local_flags {
            defects.push(mismatch(format!(
                "entry {i} ('{shown}') has general-purpose flags 0x{central_flags:04x} in the central directory but 0x{local_flags:04x} in its local header",
            )));
        }
        // A zero CRC or size in a header whose own bit 3 is set is the
        // "unknown until the descriptor" placeholder (APPNOTE 4.3.9),
        // not a disagreement; any other value must match the record the
        // reader trusts. The flag comparison above already reports the
        // two bits disagreeing.
        let local_descriptor = local.flags & 0x0008 != 0;
        if !(local_descriptor && local.crc32 == 0) && local.crc32 != central.crc32 {
            defects.push(mismatch(format!(
                "entry {i} ('{shown}') has CRC-32 0x{:08x} in the central directory but 0x{:08x} in its local header",
                central.crc32, local.crc32
            )));
        }
        for (what, local_field, central_value, which) in [
            (
                "uncompressed size",
                local.uncompressed_size,
                entry.size(),
                0,
            ),
            (
                "compressed size",
                local.compressed_size,
                entry.compressed_size(),
                1,
            ),
        ] {
            let local_value = resolved_local_size(local_field, &local.extra, which);
            let allowed = local_value == Some(central_value)
                || (local_descriptor && (local_field == 0 || local_field == u32::MAX));
            if !allowed {
                let local_shown = local_value.map_or_else(
                    || "an unresolved ZIP64 placeholder".to_string(),
                    |value| format!("{value}"),
                );
                defects.push(mismatch(format!(
                    "entry {i} ('{shown}') has {what} {central_value} in the central directory but {local_shown} in its local header",
                )));
            }
        }
        // The entry's extent: its local header, variable fields and
        // compressed payload, plus the data descriptor bit 3 announces.
        let descriptor = central_flags & 0x0008 != 0;
        let data_start = local_at
            .checked_add(LOCAL_HEADER.name as u64)
            .and_then(|at| {
                at.checked_add(u64::from(local.name_length))
                    .and_then(|at| at.checked_add(u64::from(local.extra_length)))
            });
        let Some(data_start) = data_start else {
            defects.push(ZipDefect {
                kind: ZipDefectKind::Extent,
                detail: format!("entry {i} ('{shown}') has an extent that overflows u64"),
            });
            continue;
        };
        let Some(payload_end) = data_start.checked_add(entry.compressed_size()) else {
            defects.push(ZipDefect {
                kind: ZipDefectKind::Extent,
                detail: format!("entry {i} ('{shown}') has an extent that overflows u64"),
            });
            continue;
        };
        // APPNOTE 4.3.9.2: the local header's 0x0001 field alone
        // prescribes the descriptor's 8-byte sizes, whatever it or the
        // fixed fields hold.
        let zip64 = local.compressed_size == u32::MAX
            || local.uncompressed_size == u32::MAX
            || entry.compressed_size() > u64::from(u32::MAX)
            || entry.size() > u64::from(u32::MAX)
            || zip64_field(&local.extra).is_some();
        let end = if descriptor {
            match descriptor_extent(
                reader,
                payload_end,
                zip64,
                central.crc32,
                entry.compressed_size(),
                entry.size(),
            ) {
                Ok(length) => match payload_end.checked_add(length) {
                    Some(end) => end,
                    None => {
                        defects.push(ZipDefect {
                            kind: ZipDefectKind::Extent,
                            detail: format!(
                                "entry {i} ('{shown}') has an extent that overflows u64"
                            ),
                        });
                        continue;
                    }
                },
                Err(why) => {
                    defects.push(ZipDefect {
                        kind: ZipDefectKind::Descriptor,
                        detail: format!("entry {i} ('{shown}') {why}"),
                    });
                    continue;
                }
            }
        } else {
            payload_end
        };
        extents.push((i, shown.clone(), local_at, end));
    }
    let central_directory = archive.central_directory_start();
    extents.sort_unstable_by_key(|&(_, _, start, _)| start);
    for &(index, ref name, start, end) in &extents {
        if end > central_directory {
            defects.push(ZipDefect {
                kind: ZipDefectKind::Extent,
                detail: format!(
                    "entry {index} ('{name}') at offset {start} reaches past the central directory at offset {central_directory}"
                ),
            });
        }
    }
    for pair in extents.windows(2) {
        let [(index, name, _, end), (_, other, other_start, _)] = pair else {
            continue;
        };
        if end > other_start {
            defects.push(ZipDefect {
                kind: ZipDefectKind::Extent,
                detail: format!(
                    "entry {index} ('{name}') ends at {end}, inside entry '{other}' starting at {other_start}"
                ),
            });
        }
    }
    defects.extend(
        duplicate_parts(reader, archive.central_directory_start())
            .into_iter()
            .map(|(_, defect)| defect),
    );
    defects
}

/// Two central records naming one OPC part (`DUPLICATE_PART`, ZIP-15/#600,
/// ECMA-376 Part 2 §6.2.2.3 M1.12). The reader's name index collapses a
/// byte-equal duplicate (and Word refuses the file either way — probes
/// p1/p2/p6/p7/p9), so the check walks the raw central records instead of
/// the index: each record's name decodes exactly as `zip_entry_name`
/// resolves it for the readers, and two decoded names equal under ASCII
/// case are one part. A directory entry (`word/`) is not a part and never
/// collides with a file. The walk stops at the first record that is not
/// a central header, so a comment or trailing data cannot desynchronize
/// it, and it allocates one entry per central record — the same bound
/// the index itself would reach. Each defect pairs with the second
/// entry's decoded name — the spelling `PartFs`'s typed refusal names.
pub(crate) fn duplicate_parts<R: Read + Seek>(
    reader: &mut R,
    mut at: u64,
) -> Vec<(String, ZipDefect)> {
    let mut defects = Vec::new();
    let mut seen: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for index in 0usize.. {
        let Some(central) = header_fields(reader, at, &CENTRAL_HEADER) else {
            break;
        };
        let step = u64::from(central.name_length)
            .saturating_add(u64::from(central.extra_length))
            .saturating_add(u64::from(central.comment_length));
        let Some(next) = at
            .checked_add(CENTRAL_HEADER.name as u64)
            .and_then(|at| at.checked_add(step))
        else {
            break;
        };
        at = next;
        if central.name.ends_with(b"/") {
            continue;
        }
        // The per-entry read refuses an archive whose name fails to decode
        // with the same error; it needs no second finding here.
        let Ok(name) =
            crate::admission::zip_entry_name_raw(&central.name, central.flags, &central.extra)
        else {
            continue;
        };
        let identity = name.to_ascii_lowercase();
        match seen.get(&identity) {
            None => {
                seen.insert(identity, name);
            }
            Some(first) if *first == name => {
                defects.push((
                    name.clone(),
                    ZipDefect {
                        kind: ZipDefectKind::DuplicatePart,
                        detail: format!(
                            "two ZIP entries have the same name '{name}' (entry {index})"
                        ),
                    },
                ));
            }
            Some(first) => {
                defects.push((
                    name.clone(),
                    ZipDefect {
                        kind: ZipDefectKind::DuplicatePart,
                        detail: format!(
                            "'{first}' and '{name}' name the same part (entry {index}; part names ignore case)"
                        ),
                    },
                ));
            }
        }
    }
    defects
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::io::{Cursor, SeekFrom, Write};
    use std::rc::Rc;

    use zip::write::SimpleFileOptions;
    use zip::{CompressionMethod, ZipWriter};

    #[derive(Clone)]
    struct CountingReader<'a> {
        inner: Cursor<&'a [u8]>,
        bytes_read: Rc<Cell<usize>>,
    }

    impl Read for CountingReader<'_> {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            let read = self.inner.read(buf)?;
            self.bytes_read.set(self.bytes_read.get() + read);
            Ok(read)
        }
    }

    impl Seek for CountingReader<'_> {
        fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
            self.inner.seek(pos)
        }
    }

    fn stored_package(payload: &[u8]) -> Vec<u8> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        writer
            .start_file(
                "payload.bin",
                SimpleFileOptions::default().compression_method(CompressionMethod::Stored),
            )
            .unwrap();
        writer.write_all(payload).unwrap();
        writer.finish().unwrap().into_inner()
    }

    #[test]
    fn open_archive_does_not_scan_payload() {
        let payload = vec![0x61; 1024 * 1024];
        let package = stored_package(&payload);
        let bytes_read = Rc::new(Cell::new(0));
        let reader = CountingReader {
            inner: Cursor::new(&package),
            bytes_read: Rc::clone(&bytes_read),
        };

        let mut archive = open_archive(reader).unwrap();

        assert!(
            bytes_read.get() < payload.len() / 2,
            "opening read {} bytes for a {}-byte payload",
            bytes_read.get(),
            payload.len()
        );
        assert_eq!(archive.len(), 1);
        let mut actual = Vec::new();
        archive
            .by_name("payload.bin")
            .unwrap()
            .read_to_end(&mut actual)
            .unwrap();
        assert_eq!(actual, payload);
    }

    #[test]
    fn open_archive_reads_prepended_data() {
        let payload = b"entry contents";
        let prefix = b"prepended data";
        let mut package = prefix.to_vec();
        package.extend_from_slice(&stored_package(payload));
        assert!(
            ZipArchive::with_config(
                Config {
                    archive_offset: ArchiveOffset::Known(0),
                },
                Cursor::new(&package),
            )
            .is_err()
        );

        let mut archive = open_archive(Cursor::new(&package)).unwrap();

        assert_eq!(archive.offset(), prefix.len() as u64);
        assert_eq!(archive.len(), 1);
        let mut actual = Vec::new();
        archive
            .by_name("payload.bin")
            .unwrap()
            .read_to_end(&mut actual)
            .unwrap();
        assert_eq!(actual, payload);
    }

    #[test]
    fn open_archive_refuses_non_zip_input() {
        assert!(open_archive(Cursor::new(b"not a zip")).is_err());
    }

    /// Change only the local name, locating the header through the
    /// central record rather than searching payload bytes.
    fn rename_local_header(bytes: &mut [u8], from: &[u8], to: &[u8]) {
        assert_eq!(from.len(), to.len());
        let at = {
            let mut archive = ZipArchive::new(Cursor::new(&*bytes)).unwrap();
            archive
                .by_name(std::str::from_utf8(from).unwrap())
                .unwrap()
                .header_start() as usize
        };
        assert_eq!(&bytes[at..at + 4], b"PK\x03\x04");
        assert_eq!(&bytes[at + 30..at + 30 + from.len()], from);
        bytes[at + 30..at + 30 + from.len()].copy_from_slice(to);
    }

    /// GHSA-m665-v7pm-7grj: the shared open refuses a package whose local
    /// header names an entry differently than the central directory; the
    /// unchecked open accepts it so `admit` and `debug` can see it.
    #[test]
    fn open_archive_refuses_a_dual_header_package() {
        let mut package = stored_package(b"payload");
        rename_local_header(&mut package, b"payload.bin", b"payload.bun");
        let err = open_archive(Cursor::new(&package)).expect_err("refused");
        assert!(err.to_string().contains("ZIP_HEADER_MISMATCH"), "{err}");
        assert!(open_archive_unchecked(Cursor::new(&package)).is_ok());
    }

    /// Change only the local header's last-modified time (`+10`), as a
    /// tool that rewrites the central record can leave it.
    fn retime_local_header(bytes: &mut [u8], name: &str) {
        let at = {
            let mut archive = ZipArchive::new(Cursor::new(&*bytes)).unwrap();
            archive.by_name(name).unwrap().header_start() as usize
        };
        assert_eq!(&bytes[at..at + 4], b"PK\x03\x04");
        bytes[at + 10] ^= 0x5a;
        bytes[at + 11] ^= 0x21;
    }

    /// A local time that disagrees with the central record hides no
    /// content, and Word opens such files (three real inputs carry one on
    /// `word/document.xml`). Only the names must agree (#594); the
    /// content fields are #644's.
    #[test]
    fn open_archive_accepts_a_local_time_mismatch() {
        let mut package = stored_package(b"payload");
        retime_local_header(&mut package, "payload.bin");
        let mut archive =
            open_archive(Cursor::new(&package)).expect("a time mismatch is not cloaking");
        let mut actual = Vec::new();
        archive
            .by_name("payload.bin")
            .unwrap()
            .read_to_end(&mut actual)
            .unwrap();
        assert_eq!(actual, b"payload");
    }

    /// A central record pointing at bytes that are no local file header is
    /// refused too: a scanner walking local headers would not find it.
    #[test]
    fn open_archive_refuses_a_central_record_without_its_local_header() {
        let mut package = stored_package(b"payload");
        let at = {
            let mut archive = ZipArchive::new(Cursor::new(&package)).unwrap();
            archive.by_name("payload.bin").unwrap().header_start() as usize
        };
        package[at + 2] = b'X';
        let err = open_archive(Cursor::new(&package)).expect_err("refused");
        assert!(
            err.to_string()
                .contains("ZIP_HEADER_MISMATCH: local and central headers disagree: entry 0 ('payload.bin') has no local header at offset 0"),
            "{err}"
        );
    }

    /// ZIP-17 (#644): the open refuses a compression method the local
    /// header records differently than the central directory — the same
    /// dual-header class as a name disagreement.
    #[test]
    fn open_archive_refuses_a_local_method_mismatch() {
        let mut package = stored_package(b"payload");
        let at = {
            let mut archive = ZipArchive::new(Cursor::new(&*package)).unwrap();
            archive.by_name("payload.bin").unwrap().header_start() as usize
        };
        assert_eq!(&package[at..at + 4], b"PK\x03\x04");
        package[at + 8] = 8; // deflate where the record says stored
        let err = open_archive(Cursor::new(&package)).expect_err("refused");
        assert!(err.to_string().contains("ZIP_HEADER_MISMATCH"), "{err}");
        assert!(
            err.to_string()
                .contains("compression method 0 in the central directory but 8"),
            "{err}"
        );
    }

    /// #644 fix round: bit 11 is compared only when a raw name byte is
    /// 0x80 or above — an ASCII name decodes identically whether the bit
    /// is set, so the one-sided flag a real Word-opened file carries is
    /// not a mismatch.
    #[test]
    fn open_archive_compares_bit_11_only_for_non_ascii_names() {
        use crate::admission::zip_names;

        let patch = |bytes: &mut Vec<u8>, name: &str, bit: u16, local: bool, set: bool| {
            let at = {
                let mut archive = ZipArchive::new(Cursor::new(&*bytes)).unwrap();
                let entry = archive.by_name(name).unwrap();
                if local {
                    entry.header_start() as usize + 6
                } else {
                    entry.central_header_start() as usize + 8
                }
            };
            let flags = u16::from_le_bytes([bytes[at], bytes[at + 1]]);
            let flags = if set { flags | bit } else { flags & !bit };
            bytes[at..at + 2].copy_from_slice(&flags.to_le_bytes());
        };
        let mut ascii = zip_names::docx_package("", &[zip_names::RawEntry::raw(b"x.bin", b"x")]);
        patch(&mut ascii, "x.bin", 0x0800, true, true);
        open_archive(Cursor::new(&ascii)).expect("a one-sided UTF-8 flag on ASCII admits");
        for local_only in [true, false] {
            let mut bytes = zip_names::docx_package(
                "",
                &[zip_names::RawEntry::raw("word/m\u{e9}dia.bin".as_bytes(), b"x").utf8()],
            );
            patch(&mut bytes, "word/m\u{e9}dia.bin", 0x0800, local_only, false);
            let err = open_archive(Cursor::new(&bytes))
                .expect_err("a one-sided UTF-8 flag on a non-ASCII name refuses");
            assert!(
                err.to_string().contains("ZIP_HEADER_MISMATCH"),
                "local_only={local_only}: {err}"
            );
        }
    }

    /// #644 fix round: a bit-3 entry whose local header carries a 0x0001
    /// ZIP64 extra field takes a 24-byte descriptor (APPNOTE 4.3.9.2),
    /// whatever its sizes say — Word opens the shape.
    #[test]
    fn open_archive_admits_a_zip64_local_extra_wide_descriptor() {
        use crate::admission::zip_names;

        let extra = zip_names::zip64_extra(0, 0);
        let package = zip_names::assembled(
            b"",
            &[zip_names::AssembledEntry {
                local_extra: extra.as_slice(),
                ..zip_names::AssembledEntry::plain(b"payload.bin", b"payload")
                    .with_descriptor(zip_names::Descriptor::SignedZip64)
            }],
            None,
            b"",
        );
        let mut archive = open_archive(Cursor::new(&package)).expect("the wide descriptor admits");
        let mut actual = Vec::new();
        archive
            .by_name("payload.bin")
            .unwrap()
            .read_to_end(&mut actual)
            .unwrap();
        assert_eq!(actual, b"payload");
    }

    /// The header check refuses only headers that disagree: every `.docx`
    /// the repository commits under `tests/` (fixtures, corpus, goldens)
    /// must still open. A refusal here means a conforming writer produced
    /// headers the check cannot tolerate — loosen nothing, investigate.
    #[test]
    fn every_committed_docx_opens_under_header_consistency() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
        let mut stack = vec![root];
        let mut files = Vec::new();
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().is_some_and(|e| e == "docx") {
                    files.push(path);
                }
            }
        }
        files.sort();
        assert!(
            files.len() > 500,
            "the sweep should cover the corpus, found {}",
            files.len()
        );
        let mut not_zip = Vec::new();
        let mut refused = Vec::new();
        for path in &files {
            let bytes = std::fs::read(path).unwrap();
            match open_archive_unchecked(Cursor::new(&bytes)) {
                Err(e) => not_zip.push(format!("{}: {e}", path.display())),
                Ok(_) => {
                    if let Err(e) = open_archive(Cursor::new(&bytes)) {
                        refused.push(format!("{}: {e}", path.display()));
                    }
                }
            }
        }
        assert!(
            refused.is_empty(),
            "{} of {} committed docx fail header consistency:\n{}",
            refused.len(),
            files.len(),
            refused.join("\n")
        );
        assert!(
            not_zip.is_empty(),
            "{} committed docx are not readable zips:\n{}",
            not_zip.len(),
            not_zip.join("\n")
        );
    }
}

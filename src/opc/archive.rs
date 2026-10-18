// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::io::{Read, Seek};

use zip::ZipArchive;
use zip::read::{ArchiveOffset, Config};
use zip::result::{ZipError, ZipResult};

/// Open at offset zero first, retaining support for prepended data.
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
/// zip 9.0.5's [`ZipArchive::validate_header_consistency`]. A local file
/// header that disagrees with the central directory — a dual-header "zip
/// cloaking" package, GHSA-m665-v7pm-7grj — fails the open. The error
/// text carries `admit`'s `ZIP_HEADER_MISMATCH` code so an agent sees the
/// same refusal whichever route met the file.
pub(crate) fn open_archive<R: Read + Seek + Clone>(reader: R) -> ZipResult<ZipArchive<R>> {
    let mut archive = open_archive_unchecked(reader)?;
    archive.validate_header_consistency().map_err(|e| {
        ZipError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "{}: local and central headers disagree: {e}",
                crate::admission::AdmissionErrorKind::ZipHeaderMismatch.code()
            ),
        ))
    })?;
    Ok(archive)
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

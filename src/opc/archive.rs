// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::io::{Read, Seek};

use zip::ZipArchive;
use zip::read::{ArchiveOffset, Config};
use zip::result::ZipResult;

/// Open at offset zero first, retaining support for prepended data.
pub(crate) fn open_archive<R: Read + Seek + Clone>(reader: R) -> ZipResult<ZipArchive<R>> {
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
}

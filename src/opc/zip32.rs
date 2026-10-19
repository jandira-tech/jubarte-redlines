// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The ZIP32 boundary check every package write plans against (ZIP-10,
//! #604). jubarte does not write ZIP64: Word parity for it is unproven,
//! and no admitted input produces a part or archive near the boundary
//! (`InputLimits` caps parts at 64 MiB, `compare` at 512 MiB). Instead of
//! learning mid-write that an entry is too large — zip's writer reports
//! `Large file option has not been set` only after the bytes are written
//! — a writer runs [`zip32_plan`] on the names and uncompressed lengths
//! it is about to write, before `ZipWriter::new`, and refuses up front.

use std::fmt;

/// The `0xFFFFFFFF` value a ZIP32 field writes as the "see the ZIP64
/// extra field" marker: a size or offset that reaches it needs ZIP64
/// (zip's `ZIP64_BYTES_THR`, APPNOTE 4.5.3).
const ZIP32_END: u64 = u32::MAX as u64;

/// `0xFFFF`, the value a ZIP32 end record's entry-count fields write as
/// the ZIP64 marker: a package with that many entries is ambiguous to a
/// reader checking for the marker (zip's `ZIP64_ENTRY_THR`).
const ZIP32_MAX_ENTRIES: usize = u16::MAX as usize;

/// The largest deflate stream `n` uncompressed bytes can produce: zlib's
/// `compressBound` formula (`n + n/4096 + n/16384 + n/33554432 + 13`),
/// which covers the RFC 1951 §3.2.4 stored-block worst case — a block
/// header, a two-byte LEN and a two-byte NLEN per 65,535-byte block,
/// five bytes of overhead each — with margin. A Stored entry's payload
/// is `n`, below the same bound, so every writer shares it.
fn deflate_bound(n: u64) -> u64 {
    n.saturating_add(n / 4096)
        .saturating_add(n / 16384)
        .saturating_add(n / 33_554_432)
        .saturating_add(13)
}

/// A planned write that cannot stay inside ZIP32: the package would need
/// ZIP64, which jubarte does not write. `Display` leads with the stable
/// `OUTPUT_LIMIT` code, so the refusal reads the same on the CLI and
/// through the bindings, whatever wrapper carries it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Zip32Limit(pub String);

impl fmt::Display for Zip32Limit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OUTPUT_LIMIT: {}", self.0)
    }
}

impl std::error::Error for Zip32Limit {}

/// Refuse the write `entries` describes — `(name, uncompressed length)`
/// per ZIP entry — when ZIP32 cannot hold it:
///
/// - more than 65,534 entries (65,535 is the ZIP64 count marker);
/// - an entry whose uncompressed size reaches `0xFFFFFFFF`, or whose
///   worst-case compressed size ([`deflate_bound`]) does;
/// - a local-header offset, central-directory start or central-directory
///   size that reaches `0xFFFFFFFF`.
///
/// Offsets are worst-case: local headers count 30 bytes plus the name,
/// central records 46 plus the name, the end record 22 (APPNOTE 4.3.7,
/// 4.3.12, 4.3.16); real headers also carry extra fields, so the plan
/// refuses early rather than late.
///
/// # Errors
///
/// A [`Zip32Limit`] naming the entry and the boundary.
pub fn zip32_plan(entries: &[(&str, u64)]) -> Result<(), Zip32Limit> {
    if entries.len() >= ZIP32_MAX_ENTRIES {
        return Err(Zip32Limit(format!(
            "{} entries; ZIP32 writes at most 65,534 (65,535 is the ZIP64 marker)",
            entries.len()
        )));
    }
    let mut at = 0u64;
    let mut central_size = 0u64;
    for (name, size) in entries {
        if *size >= ZIP32_END || deflate_bound(*size) >= ZIP32_END {
            return Err(Zip32Limit(format!(
                "'{name}' would need ZIP64: {size} uncompressed bytes; ZIP32 ends at 4,294,967,295"
            )));
        }
        let local_end = at
            .saturating_add(30)
            .saturating_add(name.len() as u64)
            .saturating_add(deflate_bound(*size));
        if local_end >= ZIP32_END {
            return Err(Zip32Limit(format!(
                "'{name}' would need ZIP64: its local header lands at offset {at} and the archive reaches {local_end}; ZIP32 offsets end at 4,294,967,295"
            )));
        }
        at = local_end;
        central_size = central_size
            .saturating_add(46)
            .saturating_add(name.len() as u64);
    }
    if central_size >= ZIP32_END {
        return Err(Zip32Limit(format!(
            "the central directory ({central_size} bytes at offset {at}) would need ZIP64; ZIP32 ends at 4,294,967,295"
        )));
    }
    Ok(())
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn a_small_package_plans_zip32() {
        zip32_plan(&[
            ("[Content_Types].xml", 500),
            ("_rels/.rels", 300),
            ("word/document.xml", 10_000),
        ])
        .unwrap();
        zip32_plan(&[]).unwrap();
    }

    #[test]
    fn the_refusal_leads_with_its_stable_code() {
        let err = zip32_plan(&[("word/media/big.bin", ZIP32_END)])
            .expect_err("a part of 0xFFFFFFFF bytes needs ZIP64");
        assert!(err.to_string().starts_with("OUTPUT_LIMIT:"), "{err}");
    }

    #[test]
    fn the_entry_count_ends_below_the_zip64_marker() {
        let at_marker = vec![("a.bin", 0); ZIP32_MAX_ENTRIES];
        let err = zip32_plan(&at_marker).expect_err("65,535 is the ZIP64 marker");
        assert!(err.0.contains("65,534"), "{err}");
        let under = vec![("a.bin", 0); ZIP32_MAX_ENTRIES - 1];
        zip32_plan(&under).unwrap();
    }

    #[test]
    fn a_part_at_the_size_boundary_is_refused() {
        let err = zip32_plan(&[("word/media/big.bin", ZIP32_END)])
            .expect_err("a part of 0xFFFFFFFF bytes needs ZIP64");
        assert!(err.0.contains("big.bin"), "{err}");
        assert!(err.0.contains("4,294,967,295"), "{err}");
        // Just under the marker still needs ZIP64: the worst-case
        // compressed size reaches the boundary first.
        let err = zip32_plan(&[("word/media/big.bin", ZIP32_END - 1_000_000)])
            .expect_err("the compressed-size bound reaches the boundary");
        assert!(err.0.contains("big.bin"), "{err}");
        zip32_plan(&[("word/media/big.bin", ZIP32_END - 2_000_000)]).unwrap();
    }

    #[test]
    fn an_offset_at_the_boundary_is_refused() {
        // Two parts just under the compressed-size boundary: the second
        // entry's local header lands past the ZIP32 offset boundary.
        let err = zip32_plan(&[
            ("a.bin", ZIP32_END - 2_000_000),
            ("b.bin", ZIP32_END - 2_000_000),
        ])
        .expect_err("an offset past the boundary needs ZIP64");
        assert!(err.0.contains("b.bin"), "{err}");
        assert!(err.0.contains("offset"), "{err}");
        // The first entry alone plans ZIP32.
        zip32_plan(&[("a.bin", ZIP32_END - 2_000_000)]).unwrap();
    }

    #[test]
    fn a_central_directory_at_the_boundary_is_refused() {
        // 65,534 entries of a 65,493-byte name: each local record stays
        // inside ZIP32 (30 + name + 13 ≦ the boundary), but the central
        // records (46 + name each) total 4,295,032,826 bytes — past it.
        let name = "n".repeat(65_493);
        let entries: Vec<(&str, u64)> = (0..65_534).map(|_| (name.as_str(), 0)).collect();
        let err = zip32_plan(&entries).expect_err("the central directory needs ZIP64");
        assert!(err.0.contains("central directory"), "{err}");
    }

    #[test]
    fn deflate_bound_covers_stored_blocks() {
        // RFC 1951 §3.2.4: five bytes of overhead per 65,535-byte stored
        // block; compressBound stays above that for every size.
        for n in [0u64, 1, 65_534, 65_535, 65_536, 1 << 20, ZIP32_END - 1] {
            let stored = n + 5 * (n / 65_535 + 1);
            assert!(
                deflate_bound(n) >= stored,
                "{n}: {} < {stored}",
                deflate_bound(n)
            );
        }
    }
}

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! SHA-1 hex helpers, FNV-1a fingerprints, and hex encode / decode helpers.
//!
//! Ports of `SHA1HashStringForUTF8String` / `SHA1HashStringForByteArray` /
//! `HexStringFromBytes` from `PtUtil.ts`, plus the FNV-1a 64-bit / 128-bit
//! hashes ([`fnv1a_64`], [`fnv1a_128`]) the LCS derives from those SHA-1 hex
//! strings as content-bucketing keys.
//!
//! Used by the comparer's LCS hot path (atom hashes), the inline-atom-hash
//! path (`AtomHash`), the comparison-unit word hash, and the JSON block
//! hash. CLI: every `jubarte` subcommand that hashes a block of text uses
//! [`sha1_hex`].

use sha1::{Digest, Sha1};

/// `HexStringFromBytes` — lowercase hex.
const HEX_LOWER: [u8; 16] = *b"0123456789abcdef";

pub(crate) fn hex_string_from_bytes(bytes: &[u8]) -> String {
    // HASH-01c: write ASCII nibbles into a byte buffer (no per-nibble `char` push).
    let mut out = vec![0u8; bytes.len() * 2];
    for (i, &b) in bytes.iter().enumerate() {
        out[i * 2] = HEX_LOWER[(b >> 4) as usize];
        out[i * 2 + 1] = HEX_LOWER[(b & 0x0f) as usize];
    }
    // HEX_LOWER is pure ASCII; from_utf8 cannot fail.
    String::from_utf8(out).expect("hex digits are ASCII")
}

/// Lowercase-hex SHA-1 of `s`'s UTF-8 bytes. Port of
/// `SHA1HashStringForUTF8String`.
pub fn sha1_hex(s: &str) -> String {
    sha1_hex_bytes(s.as_bytes())
}

/// Raw 20-byte SHA-1 digest of `bytes`. Equal to
/// `hex_decode_20(sha1_hex_bytes(bytes))` once the hex is decoded. The
/// inline-atom-hash path (`AtomHash`) stores this directly instead of the
/// 40-char hex `String`.
pub fn sha1_digest(bytes: &[u8]) -> [u8; 20] {
    let mut hasher = Sha1::new();
    hasher.update(bytes);
    hasher.finalize().into()
}

/// Lowercase-hex-encode 20 digest bytes into a fixed 40-byte ASCII buffer
/// (no heap allocation). Byte-identical to
/// `hex_string_from_bytes(digest).into_bytes()`. Used by
/// [`sha1_hex_of_digest_hexes`] and the inline-atom-hash `to_hex` path.
pub fn hex_encode_20(digest: &[u8; 20]) -> [u8; 40] {
    let mut out = [0u8; 40];
    for (i, &b) in digest.iter().enumerate() {
        out[i * 2] = HEX_LOWER[(b >> 4) as usize];
        out[i * 2 + 1] = HEX_LOWER[(b & 0x0f) as usize];
    }
    out
}

#[inline]
fn hex_val(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

/// Decode exactly 40 hex characters into the 20-byte digest they encode;
/// `None` for any other length or any non-hex character. Inverse of
/// [`hex_encode_20`] on well-formed input, so
/// `hex_decode_20(sha1_hex(x)) == Some(sha1_digest(x))` for any `x`.
/// Case-insensitive: `0-9a-fA-F` are all accepted.
pub fn hex_decode_20(s: &str) -> Option<[u8; 20]> {
    let bytes = s.as_bytes();
    if bytes.len() != 40 {
        return None;
    }
    let mut out = [0u8; 20];
    for (i, slot) in out.iter_mut().enumerate() {
        let hi = hex_val(bytes[i * 2])?;
        let lo = hex_val(bytes[i * 2 + 1])?;
        *slot = (hi << 4) | lo;
    }
    Some(out)
}

/// SHA-1 of the concatenation of the 40-char lowercase hex encodings of
/// each digest. Byte-identical to
/// `sha1_hex_parts(digests.map(|d| hex(d)))`. Used by
/// [`crate::comparer::atoms::ComparisonUnitWord::new`] to hash a word
/// from its atoms' 20-byte digests without allocating a per-atom heap
/// `String`.
pub fn sha1_hex_of_digest_hexes<'a, I>(digests: I) -> String
where
    I: IntoIterator<Item = &'a [u8; 20]>,
{
    let mut hasher = Sha1::new();
    for d in digests {
        hasher.update(hex_encode_20(d));
    }
    hex_string_from_bytes(&hasher.finalize())
}

/// Lowercase-hex SHA-1 of `bytes`. Port of
/// `SHA1HashStringForByteArray`.
pub fn sha1_hex_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(bytes);
    hex_string_from_bytes(&hasher.finalize())
}

/// SHA-1 of the concatenation of `parts`, without allocating the
/// concatenated string. Byte-identical to `sha1_hex(&parts.concat())`
/// for any iterator of UTF-8 pieces. Used by `ComparisonUnitWord::new`
/// to hash atom digests streamed straight into SHA-1.
pub fn sha1_hex_parts<'a, I>(parts: I) -> String
where
    I: IntoIterator<Item = &'a str>,
{
    let mut hasher = Sha1::new();
    for p in parts {
        hasher.update(p.as_bytes());
    }
    hex_string_from_bytes(&hasher.finalize())
}

/// 64-bit FNV-1a of the string's bytes (offset basis
/// `0xcbf29ce484222325`, prime `0x100000001b3`). Not SHA-1 and not
/// cryptographic.
///
/// Used as the `u64` bucket key of the comparison-unit index in the LCS
/// hot path (`longest_common_run`): units whose keys differ are never
/// compared, and units sharing a bucket go on to the 128-bit key,
/// [`fnv1a_128`]. It is a pure deterministic function of the string, so
/// equal strings always share a key and the pre-filter never drops a
/// real match; a `u64` collision only costs an extra probe. The input
/// is already a SHA-1 hex digest of document content, so a caller
/// cannot choose the FNV input bytes directly.
pub fn fnv1a_64(s: &str) -> u64 {
    const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut hash = OFFSET_BASIS;
    for &b in s.as_bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

/// 128-bit FNV-1a of the string's bytes (offset basis
/// `0x6c62272e07bb014262b821756295c58d`, prime
/// `0x1000000000000000000013b`). Not SHA-1 and not cryptographic.
///
/// In the LCS hot path (`extend_common_run` and the interior-run skip in
/// `longest_common_run`) this key alone decides comparison-unit
/// equality: two units correlate when their 128-bit keys are equal,
/// with no further comparison of the 40-byte SHA-1 hex strings. Equal
/// strings always share a key; distinct strings that collide would
/// wrongly correlate. That is accepted because the input is already a
/// SHA-1 hex digest of the content, so an attacker cannot choose the
/// FNV input bytes directly and a generic collision costs about 2^64
/// work, far beyond any document pair. Same construction as
/// [`fnv1a_64`], widened to 128 bits.
pub fn fnv1a_128(s: &str) -> u128 {
    const OFFSET_BASIS: u128 = 0x6c62_272e_07bb_0142_62b8_2175_6295_c58d;
    const PRIME: u128 = 0x0000_0000_0100_0000_0000_0000_0000_013b;
    let mut hash = OFFSET_BASIS;
    for &b in s.as_bytes() {
        hash ^= b as u128;
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_string_from_bytes() {
        assert_eq!(hex_string_from_bytes(b"hello"), "68656c6c6f");
        assert_eq!(hex_string_from_bytes(b""), "");
    }

    #[test]
    fn test_sha1_hex() {
        // sha1("hello") = aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d
        assert_eq!(
            sha1_hex("hello"),
            "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d"
        );
        assert_eq!(sha1_hex(""), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
    }

    #[test]
    fn test_sha1_digest() {
        let digest_hello = sha1_digest(b"hello");
        assert_eq!(
            hex_encode_20(&digest_hello).as_slice(),
            b"aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d"
        );

        let digest_empty = sha1_digest(b"");
        assert_eq!(
            hex_encode_20(&digest_empty).as_slice(),
            b"da39a3ee5e6b4b0d3255bfef95601890afd80709"
        );
    }

    #[test]
    fn test_hex_encode_20() {
        let digest = sha1_digest(b"hello");
        let encoded = hex_encode_20(&digest);
        assert_eq!(encoded, *b"aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d");
    }

    #[test]
    fn test_hex_decode_20() {
        let valid_hex = "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d";
        let digest = hex_decode_20(valid_hex).unwrap();
        assert_eq!(digest, sha1_digest(b"hello"));

        let upper_hex = "AAF4C61DDCC5E8A2DABEDE0F3B482CD9AEA9434D";
        let digest_upper = hex_decode_20(upper_hex).unwrap();
        assert_eq!(digest_upper, sha1_digest(b"hello"));

        // Invalid cases
        assert!(hex_decode_20("").is_none());
        assert!(hex_decode_20("aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434").is_none()); // 39 chars
        assert!(hex_decode_20("aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434dg").is_none()); // 41 chars
        assert!(hex_decode_20("aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434z").is_none()); // non-hex
    }

    #[test]
    fn test_sha1_hex_of_digest_hexes() {
        let d1 = sha1_digest(b"hello");
        let d2 = sha1_digest(b"world");
        let hexes = [d1, d2];

        let expected = sha1_hex(
            "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d7c211433f02071597741e6ff5a8ea34789abbf43",
        );
        assert_eq!(sha1_hex_of_digest_hexes(&hexes), expected);

        assert_eq!(
            sha1_hex_of_digest_hexes(std::iter::empty::<&[u8; 20]>()),
            sha1_hex("")
        );
    }

    #[test]
    fn test_sha1_hex_bytes() {
        assert_eq!(
            sha1_hex_bytes(b"hello"),
            "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d"
        );
        assert_eq!(
            sha1_hex_bytes(b""),
            "da39a3ee5e6b4b0d3255bfef95601890afd80709"
        );
    }

    #[test]
    fn test_sha1_hex_parts() {
        let parts = ["he", "ll", "o"];
        assert_eq!(
            sha1_hex_parts(parts),
            "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d"
        );

        let empty_parts: [&str; 0] = [];
        assert_eq!(
            sha1_hex_parts(empty_parts),
            "da39a3ee5e6b4b0d3255bfef95601890afd80709"
        );
    }

    #[test]
    fn test_fnv1a_64() {
        // Published FNV-1a 64-bit vectors (offset basis, "a", "foobar").
        assert_eq!(fnv1a_64(""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a_64("a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(fnv1a_64("foobar"), 0x8594_4171_f739_67e8);
    }

    #[test]
    fn test_fnv1a_128() {
        // Published FNV-1a 128-bit vectors (offset basis, "a", "foobar").
        assert_eq!(fnv1a_128(""), 0x6c62_272e_07bb_0142_62b8_2175_6295_c58d);
        assert_eq!(fnv1a_128("a"), 0xd228_cb69_6f1a_8caf_7891_2b70_4e4a_8964);
        assert_eq!(
            fnv1a_128("foobar"),
            0x343e_1662_793c_64bf_6f0d_3597_ba44_6f18
        );
    }

    #[test]
    fn sha1_of_multibyte_text_hashes_its_utf8_bytes() {
        assert_eq!(
            sha1_hex("\u{fc}n\u{ef}c\u{f6}d\u{e9}"),
            "bf5f8ac304f8a364c02e108151c3a18c4c6eb4ce"
        );
        assert_eq!(
            hex_encode_20(&sha1_digest(b"hello world")),
            *b"2aae6c35c94fcfb415dbe95f408b9ce91ee846ed"
        );
    }

    #[test]
    fn hex_encode_20_writes_every_nibble() {
        let digest: [u8; 20] = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54,
            0x32, 0x10, 0x0f, 0xf0, 0x5a, 0xa5,
        ];
        assert_eq!(
            hex_encode_20(&digest),
            *b"0123456789abcdeffedcba98765432100ff05aa5"
        );
        assert_eq!(hex_encode_20(&[0xff; 20]), [b'f'; 40]);
    }
}

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Port of `SHA1HashStringForUTF8String` / `SHA1HashStringForByteArray` /
//! `HexStringFromBytes` from `PtUtil.ts`.

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

/// `SHA1HashStringForUTF8String(s)` — lowercase hex SHA-1 of the UTF-8 bytes.
pub fn sha1_hex(s: &str) -> String {
    sha1_hex_bytes(s.as_bytes())
}

/// Raw 20-byte SHA-1 digest of `bytes` (the binary digest, not hex). Equal to
/// `hex_decode(sha1_hex_bytes(bytes))`. The inline-atom-hash path (`AtomHash`)
/// stores this directly instead of the 40-char hex `String`.
pub fn sha1_digest(bytes: &[u8]) -> [u8; 20] {
    let mut hasher = Sha1::new();
    hasher.update(bytes);
    hasher.finalize().into()
}

/// Lowercase-hex-encode 20 digest bytes into a fixed 40-byte ASCII buffer
/// (no heap allocation). Byte-identical to `hex_string_from_bytes(digest)`.
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

/// Decode exactly 40 hex characters into the 20-byte digest they encode; `None`
/// for any other length or a non-hex character. Inverse of [`hex_encode_20`] on
/// well-formed input, so `hex_decode_20(sha1_hex(x)) == Some(sha1_digest(x))`.
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

/// SHA-1 of the concatenation of the 40-char lowercase-hex encodings of each
/// digest. Byte-identical to `sha1_hex_parts(digests.map(|d| hex(d)))` — used by
/// [`crate::comparer::atoms::ComparisonUnitWord::new`] to hash a word from its
/// atoms' inline digests without a per-atom heap `String`.
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

/// `SHA1HashStringForByteArray(bytes)`.
pub fn sha1_hex_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(bytes);
    hex_string_from_bytes(&hasher.finalize())
}

/// SHA-1 of the concatenation of `parts`, without allocating the concatenated
/// string. Byte-identical to `sha1_hex(&parts.concat())` for any sequence of
/// UTF-8 pieces (used by `ComparisonUnitWord::new` to hash atom digests).
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

/// A fixed-width `u64` fingerprint of a hash string, used as a cheap pre-filter
/// for comparison-unit equality in the LCS hot path (`longest_common_run`). It
/// is a *pure deterministic function* of the string, so equal strings always
/// map to equal keys — the pre-filter never wrongly rejects a real match, and
/// the full hash string stays the source of truth (`key_eq && str_eq`). Not
/// lossless (u64 can collide), which is why the string confirmation remains.
///
/// Implemented as 64-bit FNV-1a over the string's bytes: deterministic,
/// dependency-free, and well-distributed for the (already uniformly random)
/// SHA-1 hex inputs it fingerprints.
pub fn sha1_fingerprint(s: &str) -> u64 {
    const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut hash = OFFSET_BASIS;
    for &b in s.as_bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

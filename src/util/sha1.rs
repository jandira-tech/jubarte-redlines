// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Port of `SHA1HashStringForUTF8String` / `SHA1HashStringForByteArray` /
//! `HexStringFromBytes` from `PtUtil.ts`, plus the FNV-1a fingerprints
//! ([`fnv1a_64`], [`fnv1a_128`]) the LCS derives from those SHA-1 hex strings.

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

/// 64-bit FNV-1a of the string's bytes (offset basis `0xcbf29ce484222325`,
/// prime `0x100000001b3`). Not SHA-1 and not cryptographic.
///
/// Used as the `u64` bucket key of the comparison-unit index in the LCS hot
/// path (`longest_common_run`): units whose keys differ are never compared,
/// and units sharing a bucket go on to the 128-bit key, [`fnv1a_128`]. It is a
/// pure deterministic function of the string, so equal strings always share a
/// key and the pre-filter never drops a real match; a `u64` collision only
/// costs an extra probe. The input is already a SHA-1 hex digest of document
/// content, so a caller cannot choose the FNV input bytes directly.
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
/// `0x6c62272e07bb014262b821756295c58d`, prime `0x1000000000000000000013b`).
/// Not SHA-1 and not cryptographic.
///
/// In the LCS hot path (`extend_common_run` and the interior-run skip in
/// `longest_common_run`) this key *alone* decides comparison-unit equality:
/// two units correlate when their 128-bit keys are equal, with no further
/// comparison of the 40-byte SHA-1 hex strings. Equal strings always share a
/// key; distinct strings that collide would wrongly correlate. That is
/// accepted because the input is already a SHA-1 hex digest of the content,
/// so an attacker cannot choose the FNV input bytes directly and a generic
/// collision costs about 2^64 work, far beyond any document pair. Same
/// construction as [`fnv1a_64`], widened to 128 bits.
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

/// Former name of [`fnv1a_64`]. It never computed SHA-1.
#[deprecated(since = "0.10.2", note = "renamed to `fnv1a_64`; this was never SHA-1")]
pub fn sha1_fingerprint(s: &str) -> u64 {
    fnv1a_64(s)
}

/// Former name of [`fnv1a_128`]. It never computed SHA-1.
#[deprecated(
    since = "0.10.2",
    note = "renamed to `fnv1a_128`; this was never SHA-1"
)]
pub fn sha1_fingerprint128(s: &str) -> u128 {
    fnv1a_128(s)
}

//! Port of `SHA1HashStringForUTF8String` / `SHA1HashStringForByteArray` /
//! `HexStringFromBytes` from `PtUtil.ts`.

use sha1::{Digest, Sha1};

/// `HexStringFromBytes` — lowercase hex.
const HEX_LOWER: [u8; 16] = *b"0123456789abcdef";

fn hex_string_from_bytes(bytes: &[u8]) -> String {
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

//! Port of `SHA1HashStringForUTF8String` / `SHA1HashStringForByteArray` /
//! `HexStringFromBytes` from `PtUtil.ts`.

use sha1::{Digest, Sha1};

/// `HexStringFromBytes` — lowercase hex.
fn hex_string_from_bytes(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
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

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-FileCopyrightText: 2023-2025 René Kijewski
//
// SPDX-License-Identifier: AGPL-3.0-only

//! zipsign signature check for downloaded release archives (#1705).
//!
//! Adapted from zipsign-api 0.2.2 (`verify/`, `verify_unsign_tar.rs` and
//! `constants.rs`), Copyright (c) 2023-2025 René Kijewski, licensed
//! `MIT OR Apache-2.0 OR Apache-2.0 WITH LLVM-exception` and used here under
//! MIT; the preserved notice is `LICENSES/LicenseRef-zipsign-MIT.txt`.
//! zipsign-api pins ed25519-dalek 2, whose digest 0.10 brings block-buffer
//! 0.10.4 (GHSA-qwgh-2vcv-g2f7); this port checks the same bytes with
//! ed25519-dalek 3.
//!
//! `zipsign sign` writes two layouts:
//!
//! - `zipsign sign zip`: a 16-byte header ([`MAGIC`], then the signature
//!   count as a little-endian `u16`), the 64-byte signatures, then the
//!   archive. The signed message is everything after the signatures.
//! - `zipsign sign tar`: the archive, then one more gzip member whose
//!   comment holds the same header and signatures in base64, closed by that
//!   member's start offset in 16 hex digits and [`GZIP_END`]. The signed
//!   message is everything before that member. Gzip readers see an empty
//!   last member, so the archive still extracts.
//!
//! Either way a signature is ed25519ph over SHA-512, with the archive's file
//! name as its context. The archive's name picks the layout, as `self_update`'s
//! own archive detection does: `.tar.gz` and `.tgz` are tar, `.zip` is zip.
//! The check reads a `.zip` on every platform, though only a build with
//! `self_update`'s `archive-zip` (Windows) can then extract it.

use std::fs::File;
use std::io::{ErrorKind, Read, Seek, SeekFrom};
use std::path::Path;

use base64::Engine;
use ed25519_dalek::{Signature, VerifyingKey};
use sha2::{Digest, Sha512};

/// An ed25519 public key trusted to sign release archives: the 32-byte file
/// `zipsign gen-key` writes.
pub type ReleaseKey = [u8; 32];

/// The signature block's magic: format 1, `ed25519ph`.
const MAGIC: &[u8; 14] = b"\x0c\x04\x01ed25519ph\x00\x00";
/// [`MAGIC`] and the little-endian `u16` signature count.
const HEADER_SIZE: usize = 16;
const SIGNATURE_LENGTH: usize = 64;
/// The most signature bytes zipsign reads or writes.
const BUF_LIMIT: usize = 1 << 16;

/// The gzip member header zipsign opens its signature member with: deflate,
/// a comment, mtime 2001-01-01T00:00:00Z, unknown OS.
const GZIP_START: [u8; 10] = {
    let [m1, m2, m3, m4] = 978_307_200_u32.to_le_bytes();
    [0x1f, 0x8b, 0x08, 0x10, m1, m2, m3, m4, 0x00, 0xff]
};
/// The comment's NUL, an empty final stored deflate block, and a zero
/// CRC-32 and size.
const GZIP_END: [u8; 14] = [0x00, 0x01, 0x00, 0x00, 0xff, 0xff, 0, 0, 0, 0, 0, 0, 0, 0];
/// The 16 hex digits of the signature member's offset.
const OFFSET_DIGITS: usize = 16;

/// Check that one of `keys` signed the archive at `archive`, as
/// `zipsign sign tar` or `zipsign sign zip` signs it: the context is the
/// archive's file name, so the download must keep its release asset name.
/// With no key there is nothing to check.
///
/// # Errors
///
/// A message naming why the archive is refused: its name is not UTF-8 or
/// not `.tar.gz`/`.tgz`/`.zip`, it cannot be read, it carries no well-formed
/// signature block, or no key verifies any of its signatures.
pub fn verify_signature(archive: &Path, keys: &[ReleaseKey]) -> Result<(), String> {
    if keys.is_empty() {
        return Ok(());
    }
    let name = archive
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("the archive's file name is not UTF-8, so it cannot be a signing context")?;
    let keys = keys
        .iter()
        .enumerate()
        .map(|(i, key)| {
            VerifyingKey::from_bytes(key)
                .map_err(|e| format!("release key #{i} is not an ed25519 public key: {e}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut file = File::open(archive).map_err(|e| format!("reading {name}: {e}"))?;

    // zipsign signs the exact archive name as context, so the extension case
    // is cryptographically bound.
    #[allow(clippy::case_sensitive_file_extension_comparisons)]
    let (message, signatures) = if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        tar_message_and_signatures(&mut file)
    } else if name.ends_with(".zip") {
        zip_message_and_signatures(&mut file)
    } else {
        return Err(format!(
            "{name}: zipsign signs only .tar.gz, .tgz and .zip archives"
        ));
    }
    .map_err(|e| format!("{name}: {e}"))?;
    let signed = keys.iter().any(|key| {
        signatures.iter().any(|signature| {
            key.verify_prehashed_strict(message.clone(), Some(name.as_bytes()), signature)
                .is_ok()
        })
    });
    if signed {
        Ok(())
    } else {
        Err(format!(
            "{name}: no trusted release key signed this archive"
        ))
    }
}

/// The SHA-512 of the archive bytes before the signature member, and the
/// signatures that member carries.
fn tar_message_and_signatures(file: &mut File) -> Result<(Sha512, Vec<Signature>), String> {
    let mut tail = [0; OFFSET_DIGITS + GZIP_END.len()];
    let tail_len = i64::try_from(tail.len()).map_err(|e| e.to_string())?;
    let data_end = file
        .seek(SeekFrom::End(-tail_len))
        .map_err(|_| "no signature: the archive is too short")?;
    file.read_exact(&mut tail).map_err(|e| e.to_string())?;
    if tail[OFFSET_DIGITS..] != GZIP_END {
        return Err("no signature: the last gzip member is not zipsign's".into());
    }
    let member_start = std::str::from_utf8(&tail[..OFFSET_DIGITS])
        .ok()
        .and_then(|digits| u64::from_str_radix(digits, 16).ok())
        .ok_or("no signature: the signature member's offset is malformed")?;
    let encoded_start = member_start
        .checked_add(GZIP_START.len() as u64)
        .ok_or("no signature: the signature member's offset is out of range")?;
    let encoded_len = data_end
        .checked_sub(encoded_start)
        .and_then(|len| usize::try_from(len).ok())
        .filter(|&len| len <= BUF_LIMIT)
        .ok_or("no signature: the signature member's offset is out of range")?;

    file.seek(SeekFrom::Start(member_start))
        .map_err(|e| e.to_string())?;
    let mut member = vec![0; GZIP_START.len() + encoded_len];
    file.read_exact(&mut member).map_err(|e| e.to_string())?;
    if member[..GZIP_START.len()] != GZIP_START {
        return Err("no signature: the signature member's gzip header is not zipsign's".into());
    }
    let block = base64::engine::general_purpose::STANDARD
        .decode(&member[GZIP_START.len()..])
        .map_err(|_| "the signature block is not base64")?;
    let (count, rest) = signature_header(&block)?;
    if rest.len() != count * SIGNATURE_LENGTH {
        return Err("the signature block's length does not match its signature count".into());
    }
    let signatures = signatures(rest);

    file.rewind().map_err(|e| e.to_string())?;
    let message = prehash(&mut file.take(member_start)).map_err(|e| e.to_string())?;
    Ok((message, signatures))
}

/// The signatures at the head of a signed zip, and the SHA-512 of the
/// archive after them.
fn zip_message_and_signatures(file: &mut File) -> Result<(Sha512, Vec<Signature>), String> {
    let mut header = [0; HEADER_SIZE];
    file.read_exact(&mut header)
        .map_err(|_| "no signature: the archive is too short")?;
    let (count, _) = signature_header(&header)?;
    let mut block = vec![0; count * SIGNATURE_LENGTH];
    file.read_exact(&mut block)
        .map_err(|_| "the archive ends inside its signatures")?;
    let signatures = signatures(&block);
    let message = prehash(file).map_err(|e| e.to_string())?;
    Ok((message, signatures))
}

/// The signature count a block's header announces, and the bytes after the
/// header.
fn signature_header(block: &[u8]) -> Result<(usize, &[u8]), String> {
    let Some((header, rest)) = block.split_first_chunk::<HEADER_SIZE>() else {
        return Err("no signature: the signature block is too short".into());
    };
    let Some((magic, count)) = header.split_first_chunk::<{ MAGIC.len() }>() else {
        return Err("no signature: the signature block is too short".into());
    };
    if magic != MAGIC {
        return Err("no signature: the signature block's magic is not zipsign's".into());
    }
    let count = usize::from(u16::from_le_bytes([count[0], count[1]]));
    if count == 0 {
        return Err("the signature block holds no signature".into());
    }
    if count * SIGNATURE_LENGTH > BUF_LIMIT {
        return Err("the signature block announces too many signatures".into());
    }
    Ok((count, rest))
}

/// The 64-byte signatures in `bytes`, whose length the caller has checked.
fn signatures(bytes: &[u8]) -> Vec<Signature> {
    let (signatures, _) = bytes.as_chunks::<SIGNATURE_LENGTH>();
    signatures.iter().map(Signature::from_bytes).collect()
}

fn prehash(input: &mut impl Read) -> std::io::Result<Sha512> {
    let mut hash = Sha512::new();
    let mut buf = [0; 1 << 14];
    loop {
        match input.read(&mut buf) {
            Ok(0) => return Ok(hash),
            Ok(n) => hash.update(&buf[..n]),
            Err(e) if e.kind() == ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
}

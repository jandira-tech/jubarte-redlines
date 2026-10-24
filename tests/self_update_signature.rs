// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The signer and the verifier agree. release.yml signs each archive with
//! `zipsign sign tar <archive> <key>` (`zipsign sign zip` for Windows), whose
//! context defaults to the archive's file name; `jubarte self-update` saves
//! the download under its release asset name and checks it with
//! [`jubarte::update::verify_signature`], whose context is that file name.
//!
//! The archives in `tests/data/self_update/` were signed by the zipsign 0.2.2
//! CLI itself (`signed-a.*` with key A, `signed-b.tar.gz` with key B,
//! `signed-ab.tar.gz` with both), under the asset names below. Only the two
//! public keys are committed. Neither key signs or ever signed a release, and
//! their private halves were never kept.

#![cfg(feature = "self-update")]

use std::path::{Path, PathBuf};

use jubarte::update::{verify_signature, ReleaseKey};
use self_update::Extract;

/// Named as release.yml stages a Linux archive.
const ASSET: &str = "jubarte-9.9.9-linux-x86_64.tar.gz";
/// Named as release.yml stages a Windows archive.
const ZIP_ASSET: &str = "jubarte-9.9.9-windows-x86_64.zip";
const BIN_IN_ARCHIVE: &str = "jubarte-9.9.9-linux-x86_64/jubarte";
const BIN_BYTES: &[u8] = b"test-only stand-in for the jubarte binary\n";

const KEY_A: ReleaseKey = *include_bytes!("data/self_update/a.pub");
const KEY_B: ReleaseKey = *include_bytes!("data/self_update/b.pub");

const UNSIGNED: &[u8] = include_bytes!("data/self_update/unsigned.tar.gz");
const SIGNED_A: &[u8] = include_bytes!("data/self_update/signed-a.tar.gz");
const SIGNED_B: &[u8] = include_bytes!("data/self_update/signed-b.tar.gz");
const SIGNED_AB: &[u8] = include_bytes!("data/self_update/signed-ab.tar.gz");
const UNSIGNED_ZIP: &[u8] = include_bytes!("data/self_update/unsigned.zip");
const SIGNED_A_ZIP: &[u8] = include_bytes!("data/self_update/signed-a.zip");

/// Write `bytes` as `dir/name`, the way the updater stores a download under
/// its asset name before checking it.
fn download(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, bytes).expect("write");
    path
}

fn flipped(bytes: &[u8], at: usize) -> Vec<u8> {
    let mut bytes = bytes.to_vec();
    bytes[at] ^= 0x01;
    bytes
}

#[test]
fn a_signed_archive_verifies_and_still_extracts() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = download(dir.path(), ASSET, SIGNED_A);

    verify_signature(&path, &[KEY_A]).expect("signature accepted");

    // The signature rides in a trailing empty gzip member; the updater's
    // extractor still finds the binary, byte for byte.
    let out = tempfile::tempdir().expect("tempdir");
    Extract::from_source(&path)
        .extract_file(out.path(), BIN_IN_ARCHIVE)
        .expect("extract");
    assert_eq!(
        std::fs::read(out.path().join(BIN_IN_ARCHIVE)).expect("read"),
        BIN_BYTES
    );
}

#[test]
fn a_signed_windows_zip_verifies() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = download(dir.path(), ZIP_ASSET, SIGNED_A_ZIP);
    verify_signature(&path, &[KEY_A]).expect("signature accepted");
}

#[test]
fn any_trusted_key_may_have_signed() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = download(dir.path(), ASSET, SIGNED_A);
    verify_signature(&path, &[KEY_B, KEY_A]).expect("the second key signed");
}

#[test]
fn a_tampered_archive_is_refused() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = download(dir.path(), ASSET, &flipped(SIGNED_A, 20));
    assert!(verify_signature(&path, &[KEY_A]).is_err());
    // A zip's signature block is its first 80 bytes; byte 100 is archive data.
    let path = download(dir.path(), ZIP_ASSET, &flipped(SIGNED_A_ZIP, 100));
    assert!(verify_signature(&path, &[KEY_A]).is_err());
}

#[test]
fn a_tampered_signature_is_refused() {
    let dir = tempfile::tempdir().expect("tempdir");
    // The tar signature is base64 text just before the 30-byte gzip tail.
    let at = SIGNED_A.len() - 40;
    let path = download(dir.path(), ASSET, &flipped(SIGNED_A, at));
    assert!(verify_signature(&path, &[KEY_A]).is_err());
    // The zip signature follows the 16-byte header.
    let path = download(dir.path(), ZIP_ASSET, &flipped(SIGNED_A_ZIP, 30));
    assert!(verify_signature(&path, &[KEY_A]).is_err());
}

#[test]
fn an_unsigned_archive_is_refused_once_a_key_is_trusted() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = download(dir.path(), ASSET, UNSIGNED);
    assert!(verify_signature(&path, &[KEY_A]).is_err());
    let path = download(dir.path(), ZIP_ASSET, UNSIGNED_ZIP);
    assert!(verify_signature(&path, &[KEY_A]).is_err());
}

#[test]
fn a_truncated_or_empty_download_is_refused() {
    let dir = tempfile::tempdir().expect("tempdir");
    for bytes in [&b""[..], &SIGNED_A[..10], &SIGNED_A[SIGNED_A.len() - 30..]] {
        let path = download(dir.path(), ASSET, bytes);
        assert!(verify_signature(&path, &[KEY_A]).is_err());
    }
    for bytes in [&b""[..], &SIGNED_A_ZIP[..40]] {
        let path = download(dir.path(), ZIP_ASSET, bytes);
        assert!(verify_signature(&path, &[KEY_A]).is_err());
    }
}

#[test]
fn a_signature_header_that_counts_no_signatures_is_refused() {
    let dir = tempfile::tempdir().expect("tempdir");
    // Bytes 14..16 of a signed zip count its signatures (little endian).
    let mut bytes = SIGNED_A_ZIP.to_vec();
    bytes[14] = 0;
    bytes[15] = 0;
    let path = download(dir.path(), ZIP_ASSET, &bytes);
    assert!(verify_signature(&path, &[KEY_A]).is_err());
}

#[test]
fn with_no_trusted_key_nothing_is_checked() {
    // Today's state: RELEASE_KEYS is empty, and SHA256SUMS.txt alone guards
    // the download.
    let dir = tempfile::tempdir().expect("tempdir");
    let path = download(dir.path(), ASSET, UNSIGNED);
    verify_signature(&path, &[]).expect("no keys, no check");
}

#[test]
fn an_untrusted_signer_is_refused() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = download(dir.path(), ASSET, SIGNED_B);
    assert!(verify_signature(&path, &[KEY_A]).is_err());
}

#[test]
fn the_signature_is_bound_to_the_asset_name() {
    // The context is the file name, so one platform's genuinely signed
    // archive cannot stand in for another's, and an archive signed under one
    // name fails once uploaded under another.
    let dir = tempfile::tempdir().expect("tempdir");
    let path = download(dir.path(), "jubarte-9.9.9-linux-aarch64.tar.gz", SIGNED_A);
    assert!(verify_signature(&path, &[KEY_A]).is_err());
    let path = download(
        dir.path(),
        "jubarte-9.9.9-windows-aarch64.zip",
        SIGNED_A_ZIP,
    );
    assert!(verify_signature(&path, &[KEY_A]).is_err());
}

#[test]
fn an_archive_zipsign_cannot_sign_is_refused_once_a_key_is_trusted() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = download(dir.path(), "jubarte-9.9.9-linux-x86_64.tar.xz", SIGNED_A);
    let error = verify_signature(&path, &[KEY_A]).expect_err("no signature format");
    assert!(error.contains(".tar.gz"), "{error}");
}

#[test]
fn during_a_rotation_either_key_verifies_a_doubly_signed_archive() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = download(dir.path(), ASSET, SIGNED_AB);
    verify_signature(&path, &[KEY_A]).expect("old key");
    verify_signature(&path, &[KEY_B]).expect("new key");
}

#[test]
fn a_header_announcing_more_signatures_than_zipsign_writes_is_refused() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut bytes = SIGNED_A_ZIP.to_vec();
    bytes[14] = 0xff;
    bytes[15] = 0xff;
    let path = download(dir.path(), ZIP_ASSET, &bytes);
    let error = verify_signature(&path, &[KEY_A]).expect_err("too many signatures");
    assert!(error.contains("too many"), "{error}");
}

#[test]
fn a_malformed_or_out_of_range_signature_offset_is_refused() {
    let dir = tempfile::tempdir().expect("tempdir");
    // The 16 hex digits before the 14-byte gzip end give the signature
    // member's offset.
    let digits = SIGNED_A.len() - 30;
    for replacement in [
        &b"zzzzzzzzzzzzzzzz"[..],
        b"ffffffffffffffff",
        b"0000000000000000",
    ] {
        let mut bytes = SIGNED_A.to_vec();
        bytes[digits..digits + 16].copy_from_slice(replacement);
        let path = download(dir.path(), ASSET, &bytes);
        assert!(verify_signature(&path, &[KEY_A]).is_err());
    }
}

// ---- Adversarial review additions for 9ea0da3e (#1705); do not commit ----
//
// Fixture archives live outside the checkout under ~/temp/T/fix013w/rev1705/w,
// all produced by the zipsign 0.2.2 CLI (rel.tar.gz / rel.tgz / rel.zip
// signed under their own names with keys/a.key) or derived from them.

const CLI_TGZ: &[u8] = include_bytes!("../../../fix013w/rev1705/w/rel.tgz");
const CLI_TAR_GZ: &[u8] = include_bytes!("../../../fix013w/rev1705/w/rel.tar.gz");
const CLI_ZIP: &[u8] = include_bytes!("../../../fix013w/rev1705/w/rel.zip");
const UPPER_HEX: &[u8] = include_bytes!("../../../fix013w/rev1705/w/attack-upperhex.tar.gz");
const PLUS_HEX: &[u8] = include_bytes!("../../../fix013w/rev1705/w/attack-offplus.tar.gz");
const TRAILING: &[u8] = include_bytes!("../../../fix013w/rev1705/w/attack-trailing.tar.gz");
const TWO_MEMBERS: &[u8] = include_bytes!("../../../fix013w/rev1705/w/attack-twomem.tar.gz");
const OFF_ZERO: &[u8] = include_bytes!("../../../fix013w/rev1705/w/attack-off0.tar.gz");
const OFF_MAX: &[u8] = include_bytes!("../../../fix013w/rev1705/w/attack-offmax.tar.gz");
const B64_SPACE: &[u8] = include_bytes!("../../../fix013w/rev1705/w/attack-b64space.tar.gz");
const NO_PAD: &[u8] = include_bytes!("../../../fix013w/rev1705/w/attack-nopad.tar.gz");
const EMPTY_MESSAGE: &[u8] =
    include_bytes!("../../../fix013w/rev1705/w/attack-emptymessage.tar.gz");
const OFF_IN_DATA: &[u8] = include_bytes!("../../../fix013w/rev1705/w/attack-offindata.tar.gz");
const SIGS_767: &[u8] = include_bytes!("../../../fix013w/rev1705/w/attack-767sigs.tar.gz");
const SIGS_768: &[u8] = include_bytes!("../../../fix013w/rev1705/w/attack-768sigs.tar.gz");
const ZDUP: &[u8] = include_bytes!("../../../fix013w/rev1705/w/dup/rel.zip");
const ZCOUNT2: &[u8] = include_bytes!("../../../fix013w/rev1705/w/c2/rel.zip");
const ZCOUNT1024: &[u8] = include_bytes!("../../../fix013w/rev1705/w/c1k/rel.zip");
const ZCOUNT1025: &[u8] = include_bytes!("../../../fix013w/rev1705/w/c1k5/rel.zip");
const ZCOUNT0: &[u8] = include_bytes!("../../../fix013w/rev1705/w/c0/rel.zip");

/// The zipsign CLI signs and verifies `.tgz` (verified:
/// `zipsign verify tar rel.tgz a.pub` -> OK). self_update's old `signatures`
/// path also verified it: detect_archive() maps `.tgz` to
/// `ArchiveKind::Tar(Some(Gz))`. The port's `name.ends_with(".tar.gz")`
/// refuses it.
#[test]
fn a_cli_signed_tgz_is_refused_by_the_port() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = download(dir.path(), "rel.tgz", CLI_TGZ);
    let error = verify_signature(&path, &[KEY_A]).expect_err("port refuses .tgz");
    assert!(error.contains(".tar.gz"), "{error}");
    // Control: the archive signed under rel.tar.gz passes under that name.
    let path = download(dir.path(), "rel.tar.gz", CLI_TAR_GZ);
    verify_signature(&path, &[KEY_A]).expect("the bytes themselves verify");
}

/// The CLI accepts an upper-case hex offset and a leading '+'
/// (`u64::from_str_radix` semantics); confirm the port matches.
#[test]
fn hex_offset_case_and_sign_match_the_cli() {
    let dir = tempfile::tempdir().expect("tempdir");
    // Both were signed under the name rel.tar.gz, so keep the context.
    let path = download(dir.path(), "rel.tar.gz", UPPER_HEX);
    verify_signature(&path, &[KEY_A]).expect("upper-case hex offset, CLI says OK");
    let path = download(dir.path(), "rel.tar.gz", PLUS_HEX);
    verify_signature(&path, &[KEY_A]).expect("leading + hex offset, CLI says OK");
}

/// Everything the CLI refused, the port must refuse too.
#[test]
fn cli_refusals_are_also_refused() {
    let dir = tempfile::tempdir().expect("tempdir");
    for bytes in [
        TRAILING,
        TWO_MEMBERS,
        OFF_ZERO,
        OFF_MAX,
        B64_SPACE,
        NO_PAD,
        EMPTY_MESSAGE,
        OFF_IN_DATA,
        SIGS_767,
        SIGS_768,
    ] {
        let path = download(dir.path(), "rel.tar.gz", bytes);
        assert!(verify_signature(&path, &[KEY_A]).is_err());
    }
    for bytes in [ZCOUNT2, ZCOUNT1024, ZCOUNT1025, ZCOUNT0] {
        let path = download(dir.path(), "rel.zip", bytes);
        assert!(verify_signature(&path, &[KEY_A]).is_err());
    }
}

/// Two copies of one signature, count = 2: the CLI accepts (any-of).
#[test]
fn a_zip_with_a_duplicated_signature_member_is_accepted() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = download(dir.path(), "rel.zip", ZDUP);
    verify_signature(&path, &[KEY_A]).expect("CLI accepts a duplicated signature");
}

/// A pathological but honest key: all-zero is a valid compressed point
/// (VerifyingKey::from_bytes succeeds) and strict verification must refuse it.
#[test]
fn an_all_zero_key_is_refused() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = download(dir.path(), "rel.tar.gz", CLI_TAR_GZ);
    assert!(verify_signature(&path, &[[0; 32]]).is_err());
    let path = download(dir.path(), "rel.zip", CLI_ZIP);
    assert!(verify_signature(&path, &[[0; 32]]).is_err());
}

#[test]
fn a_key_listed_twice_still_verifies() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = download(dir.path(), "rel.tar.gz", CLI_TAR_GZ);
    verify_signature(&path, &[KEY_A, KEY_A]).expect("duplicate key");
}

#[cfg(unix)]
#[test]
fn a_file_name_that_is_not_utf8_cannot_be_a_signing_context() {
    use std::os::unix::ffi::OsStrExt;
    // The name is checked before the file is opened, so the path need not
    // exist (APFS refuses to create it).
    let path = Path::new(std::ffi::OsStr::from_bytes(b"jubarte-\xff.tar.gz"));
    let error = verify_signature(path, &[KEY_A]).expect_err("not UTF-8");
    assert!(error.contains("UTF-8"), "{error}");
}

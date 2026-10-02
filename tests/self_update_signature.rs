// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The signer and the verifier agree. release.yml signs each archive with
//! `zipsign sign tar <archive> <key>`, whose context defaults to the archive's
//! file name; `jubarte self-update` downloads the asset under its release name
//! and checks it with `self_update::verify_signature`, whose context is the
//! downloaded file's name. These tests sign the way the CLI does and check
//! with the updater's own verifier, offline.
//!
//! Every key here is derived from a fixed, test-only seed. None of them signs
//! or ever signed a release.

#![cfg(feature = "self-update")]

use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};

use self_update::{Extract, VerifyingKey, verify_signature};
use zipsign_api::SigningKey;
use zipsign_api::sign::copy_and_sign_tar;

/// TEST ONLY: a throwaway seed so the test is deterministic. Not a release key.
const TEST_ONLY_SEED: [u8; 32] = [0x5a; 32];
/// TEST ONLY: a second throwaway seed, for an untrusted signer and rotation.
const OTHER_TEST_ONLY_SEED: [u8; 32] = [0xa5; 32];

/// Named as release.yml stages a Linux archive.
const ASSET: &str = "jubarte-9.9.9-linux-x86_64.tar.gz";
const BIN_IN_ARCHIVE: &str = "jubarte-9.9.9-linux-x86_64/jubarte";
const BIN_BYTES: &[u8] = b"test-only stand-in for the jubarte binary\n";

fn tar_gz() -> Vec<u8> {
    let gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    let mut tar = tar::Builder::new(gz);
    let mut header = tar::Header::new_gnu();
    header.set_size(BIN_BYTES.len() as u64);
    header.set_mode(0o755);
    header.set_mtime(0);
    header.set_cksum();
    tar.append_data(&mut header, BIN_IN_ARCHIVE, BIN_BYTES)
        .expect("append");
    let mut gz = tar.into_inner().expect("finish tar");
    gz.flush().expect("flush");
    gz.finish().expect("finish gzip")
}

/// Sign as `zipsign sign tar <file> <keys>...` does without `--context`: the
/// context is the file name (zipsign 0.2.1, `get_context` in src/main.rs).
fn sign(unsigned: &[u8], file_name: &str, seeds: &[[u8; 32]]) -> Vec<u8> {
    let keys: Vec<SigningKey> = seeds.iter().map(SigningKey::from_bytes).collect();
    let mut signed = Cursor::new(Vec::new());
    copy_and_sign_tar(
        &mut Cursor::new(unsigned),
        &mut signed,
        &keys,
        Some(file_name.as_bytes()),
    )
    .expect("sign");
    signed.into_inner()
}

fn public(seed: &[u8; 32]) -> VerifyingKey {
    SigningKey::from_bytes(seed).verifying_key().to_bytes()
}

/// Write `bytes` as `dir/name`, the way the updater stores a download under
/// its asset name before checking it.
fn download(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, bytes).expect("write");
    path
}

#[test]
fn a_signed_archive_verifies_and_still_extracts() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = download(
        dir.path(),
        ASSET,
        &sign(&tar_gz(), ASSET, &[TEST_ONLY_SEED]),
    );

    verify_signature(&path, &[public(&TEST_ONLY_SEED)]).expect("signature accepted");

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
fn a_tampered_archive_is_refused() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut bytes = sign(&tar_gz(), ASSET, &[TEST_ONLY_SEED]);
    bytes[20] ^= 0x01;
    let path = download(dir.path(), ASSET, &bytes);
    assert!(verify_signature(&path, &[public(&TEST_ONLY_SEED)]).is_err());
}

#[test]
fn an_unsigned_archive_is_refused_once_a_key_is_trusted() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = download(dir.path(), ASSET, &tar_gz());
    assert!(verify_signature(&path, &[public(&TEST_ONLY_SEED)]).is_err());
}

#[test]
fn with_no_trusted_key_nothing_is_checked() {
    // Today's state: RELEASE_KEYS is empty, and SHA256SUMS.txt alone guards
    // the download.
    let dir = tempfile::tempdir().expect("tempdir");
    let path = download(dir.path(), ASSET, &tar_gz());
    verify_signature(&path, &[]).expect("no keys, no check");
}

#[test]
fn an_untrusted_signer_is_refused() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = download(
        dir.path(),
        ASSET,
        &sign(&tar_gz(), ASSET, &[OTHER_TEST_ONLY_SEED]),
    );
    assert!(verify_signature(&path, &[public(&TEST_ONLY_SEED)]).is_err());
}

#[test]
fn the_signature_is_bound_to_the_asset_name() {
    // The context is the file name, so one platform's genuinely signed
    // archive cannot stand in for another's, and an archive signed under one
    // name fails once uploaded under another.
    let dir = tempfile::tempdir().expect("tempdir");
    let other = "jubarte-9.9.9-linux-aarch64.tar.gz";
    let path = download(
        dir.path(),
        other,
        &sign(&tar_gz(), ASSET, &[TEST_ONLY_SEED]),
    );
    assert!(verify_signature(&path, &[public(&TEST_ONLY_SEED)]).is_err());
}

#[test]
fn during_a_rotation_either_key_verifies_a_doubly_signed_archive() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = download(
        dir.path(),
        ASSET,
        &sign(&tar_gz(), ASSET, &[TEST_ONLY_SEED, OTHER_TEST_ONLY_SEED]),
    );
    verify_signature(&path, &[public(&TEST_ONLY_SEED)]).expect("old key");
    verify_signature(&path, &[public(&OTHER_TEST_ONLY_SEED)]).expect("new key");
}

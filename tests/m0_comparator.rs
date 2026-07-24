// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

mod common;

/// A docx is always structurally equal to itself.
#[test]
fn identity_is_structurally_equal() {
    let bytes = include_bytes!("goldens/redline.redline.docx");
    common::assert_docx_structurally_eq(bytes, bytes);
}

/// Two different goldens must NOT be structurally equal (guards against a
/// comparator that trivially passes everything).
#[test]
#[should_panic]
fn different_docx_are_not_equal() {
    let a = include_bytes!("goldens/redline.redline.docx");
    let b = include_bytes!("goldens/inpi.redline.docx");
    common::assert_docx_structurally_eq(a, b);
}

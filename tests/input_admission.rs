// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The redline comparer admits its inputs before anything inflates them.
//!
//! Every package here is assembled with the `zip` crate's ordinary writer:
//! the archives are well-formed, they are simply larger than the budget the
//! test hands the comparer. A refusal must come back as a typed `Err`, never
//! as an allocation abort or a panic, because the Python wheel and the WASM
//! build cannot catch either.

use std::io::{Cursor, Write};

use jubarte::WmlDocument;
use jubarte::admission::InputLimits;
use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::{compare_documents, compare_documents_with_settings};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

const ORIGINAL: &[u8] = include_bytes!("fixtures/redline/original.docx");
const MODIFIED: &[u8] = include_bytes!("fixtures/redline/modified.docx");

const CONTENT_TYPES: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
<Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
<Default Extension=\"xml\" ContentType=\"application/xml\"/>\
<Default Extension=\"bin\" ContentType=\"application/octet-stream\"/>\
<Override PartName=\"/word/document.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/>\
</Types>";

const PACKAGE_RELS: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
<Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"word/document.xml\"/>\
</Relationships>";

const DOCUMENT: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
<w:body><w:p><w:r><w:t>hello</w:t></w:r></w:p><w:sectPr/></w:body></w:document>";

/// A minimal WordprocessingML package plus `extra` entries appended after
/// the three mandatory parts.
fn package(extra: impl FnOnce(&mut ZipWriter<Cursor<Vec<u8>>>, SimpleFileOptions)) -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    for (name, body) in [
        ("[Content_Types].xml", CONTENT_TYPES),
        ("_rels/.rels", PACKAGE_RELS),
        ("word/document.xml", DOCUMENT),
    ] {
        writer.start_file(name, options).unwrap();
        writer.write_all(body.as_bytes()).unwrap();
    }
    extra(&mut writer, options);
    writer.finish().unwrap().into_inner()
}

/// A package whose one media part inflates to `size` zero bytes.
fn package_with_blob(size: usize) -> Vec<u8> {
    package(|writer, options| {
        writer.start_file("word/media/blob.bin", options).unwrap();
        let chunk = vec![0u8; 64 * 1024];
        let mut left = size;
        while left > 0 {
            let n = left.min(chunk.len());
            writer.write_all(&chunk[..n]).unwrap();
            left -= n;
        }
    })
}

/// A package with `count` extra one-byte parts.
fn package_with_entries(count: usize) -> Vec<u8> {
    package(|writer, options| {
        for i in 0..count {
            writer
                .start_file(format!("word/media/p{i}.bin"), options)
                .unwrap();
            writer.write_all(b"x").unwrap();
        }
    })
}

fn small_limits() -> InputLimits {
    InputLimits {
        max_part_bytes: 1024 * 1024,
        max_uncompressed_bytes: 4 * 1024 * 1024,
        max_entries: 32,
        ..InputLimits::compare()
    }
}

#[test]
fn compare_limits_are_a_generous_superset_of_the_agent_defaults() {
    let agent = InputLimits::default();
    let compare = InputLimits::compare();
    assert!(compare.max_compressed_bytes >= agent.max_compressed_bytes);
    assert!(compare.max_part_bytes >= agent.max_part_bytes);
    assert!(compare.max_uncompressed_bytes >= agent.max_uncompressed_bytes);
    assert_eq!(compare.max_entries, agent.max_entries);
    assert_eq!(compare.max_xml_depth, agent.max_xml_depth);
}

#[test]
fn settings_default_to_the_compare_limits() {
    assert_eq!(
        WmlComparerSettings::default().input_limits,
        InputLimits::compare()
    );
}

#[test]
fn compare_refuses_a_part_that_inflates_past_the_part_budget() {
    let settings = WmlComparerSettings {
        input_limits: small_limits(),
        ..WmlComparerSettings::default()
    };
    let bomb = package_with_blob(3 * 1024 * 1024);
    let err = compare_documents_with_settings(ORIGINAL, &bomb, &settings)
        .expect_err("a 3 MiB part must not pass a 1 MiB part budget");
    let text = err.to_string();
    assert!(text.contains("INPUT_LIMIT"), "{text}");
    assert!(text.contains("blob.bin"), "{text}");
}

#[test]
fn compare_refuses_the_original_side_too() {
    let settings = WmlComparerSettings {
        input_limits: small_limits(),
        ..WmlComparerSettings::default()
    };
    let bomb = package_with_blob(3 * 1024 * 1024);
    let err = compare_documents_with_settings(&bomb, MODIFIED, &settings)
        .expect_err("the original side is admitted as well");
    assert!(err.to_string().contains("INPUT_LIMIT"), "{err}");
}

#[test]
fn compare_refuses_more_entries_than_the_budget() {
    let settings = WmlComparerSettings {
        input_limits: small_limits(),
        ..WmlComparerSettings::default()
    };
    let crowded = package_with_entries(40);
    let err = compare_documents_with_settings(ORIGINAL, &crowded, &settings)
        .expect_err("43 entries must not pass a 32-entry budget");
    assert!(err.to_string().contains("INPUT_LIMIT"), "{err}");
}

#[test]
fn identical_inputs_are_admitted_before_the_fast_path() {
    let settings = WmlComparerSettings {
        input_limits: small_limits(),
        ..WmlComparerSettings::default()
    };
    let bomb = package_with_blob(3 * 1024 * 1024);
    let err = compare_documents_with_settings(&bomb, &bomb, &settings)
        .expect_err("the identical-input shortcut must not skip admission");
    assert!(err.to_string().contains("INPUT_LIMIT"), "{err}");
}

#[test]
fn the_default_entry_point_refuses_a_crowd_of_entries() {
    // InputLimits::compare() keeps the 10_000 entry budget, and the entry
    // count is read from the central directory before anything inflates.
    let crowded = package_with_entries(10_010);
    let err = compare_documents(ORIGINAL, &crowded, "Reviewer")
        .expect_err("10_013 entries must not pass the default budget");
    assert!(err.to_string().contains("INPUT_LIMIT"), "{err}");
}

#[test]
fn a_refusal_keeps_the_typed_admission_error_as_its_source() {
    let settings = WmlComparerSettings {
        input_limits: small_limits(),
        ..WmlComparerSettings::default()
    };
    let bomb = package_with_blob(3 * 1024 * 1024);
    let err = compare_documents_with_settings(ORIGINAL, &bomb, &settings).expect_err("refused");
    let jubarte::opc::OpcError::Io(io) = &err else {
        panic!("refusals travel as OpcError::Io(InvalidData): {err:?}");
    };
    assert_eq!(io.kind(), std::io::ErrorKind::InvalidData);
    let refused = io
        .get_ref()
        .and_then(|inner| inner.downcast_ref::<jubarte::admission::AdmissionError>())
        .expect("the AdmissionError rides along as the io::Error source");
    assert_eq!(refused.code(), "INPUT_LIMIT");
}

#[test]
fn a_small_well_formed_package_still_compares() {
    let settings = WmlComparerSettings {
        input_limits: small_limits(),
        ..WmlComparerSettings::default()
    };
    let small = package_with_blob(16 * 1024);
    compare_documents_with_settings(ORIGINAL, &small, &settings)
        .expect("a 16 KiB blob is well inside every budget");
    compare_documents(ORIGINAL, MODIFIED, "Reviewer").expect("the fixture pair still compares");
}

#[test]
fn wml_document_from_bytes_admits_its_input() {
    let crowded = package_with_entries(10_010);
    let Err(err) = WmlDocument::from_bytes(&crowded) else {
        panic!("10_013 entries exceed the budget");
    };
    assert!(err.to_string().contains("INPUT_LIMIT"), "{err}");
    assert!(
        WmlDocument::from_bytes(ORIGINAL).is_ok(),
        "the fixture still opens"
    );
}

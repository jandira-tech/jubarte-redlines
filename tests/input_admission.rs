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

mod common;

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

/// A minimal WordprocessingML package with `document_xml` as its
/// `word/document.xml` (`package` always writes the fixture document).
fn package_with_document(document_xml: &str) -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    for (name, body) in [
        ("[Content_Types].xml", CONTENT_TYPES),
        ("_rels/.rels", PACKAGE_RELS),
        ("word/document.xml", document_xml),
    ] {
        writer.start_file(name, options).unwrap();
        writer.write_all(body.as_bytes()).unwrap();
    }
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

/// APPNOTE 4.3.6 lets a self-extracting archive carry a stub ahead of
/// the first local header; jubarte admits a prefixed package.
#[test]
fn admission_accepts_a_package_with_prepended_data() {
    let mut bytes = b"prepended data".to_vec();
    bytes.extend_from_slice(&package(|_, _| {}));

    let admitted = jubarte::admission::admit(&bytes, small_limits()).unwrap();

    assert_eq!(admitted.entries, 3);
    assert_eq!(admitted.main_part, "word/document.xml");
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

/// Change only the local name, locating the header through the writer's
/// central record rather than searching payload bytes for a signature.
fn rename_local_header(bytes: &mut [u8], from: &[u8], to: &[u8]) {
    assert_eq!(from.len(), to.len(), "the local name must keep its length");
    let at = {
        let mut archive = zip::ZipArchive::new(Cursor::new(&*bytes)).unwrap();
        archive
            .by_name(std::str::from_utf8(from).unwrap())
            .unwrap()
            .header_start() as usize
    };
    assert_eq!(&bytes[at..at + 4], b"PK\x03\x04");
    assert_eq!(&bytes[at + 30..at + 30 + from.len()], from);
    bytes[at + 30..at + 30 + from.len()].copy_from_slice(to);
}

fn dual_header_package() -> Vec<u8> {
    let mut bytes = package(|_, _| {});
    jubarte::admission::admit(&bytes, small_limits()).expect("unmodified control is valid");
    rename_local_header(&mut bytes, b"word/document.xml", b"w0rd/document.xml");
    bytes
}

/// Rename an entry in its local AND central header (the shape a hostile
/// writer produces): the two records agree, so the only defect is the
/// duplicate part identity the new spelling creates (#600). `stand_in` is
/// the name the writer emitted, `alias` the name both headers adopt.
fn rename_entry(bytes: &mut [u8], stand_in: &[u8], alias: &[u8]) {
    assert_eq!(stand_in.len(), alias.len(), "names must keep the length");
    let (lfh, central) = header_offsets(bytes, std::str::from_utf8(stand_in).unwrap());
    bytes[lfh + 30..lfh + 30 + alias.len()].copy_from_slice(alias);
    bytes[central + 46..central + 46 + alias.len()].copy_from_slice(alias);
}

/// A valid package carrying a second entry that names the same part —
/// exactly or under ASCII case (#600 probes p1/p2/p6/p7/p9, which Word
/// refuses). `stand_in` is the unique name the writer emits, `alias` the
/// spelling it is renamed to.
fn duplicate_probe(stand_in: &str, alias: &str) -> Vec<u8> {
    let mut bytes = package(|writer, options| {
        writer
            .start_file(
                stand_in,
                options.compression_method(CompressionMethod::Stored),
            )
            .unwrap();
        writer.write_all(b"second").unwrap();
    });
    rename_entry(&mut bytes, stand_in.as_bytes(), alias.as_bytes());
    bytes
}

/// A package with exactly `entries` parts in order (a name ending in `/`
/// becomes a directory entry). Unlike `package`, the caller chooses every
/// part name, so case-variant spellings of the reserved names are
/// expressible (#600 probes p3/p4/p5).
fn package_named(entries: &[(&str, &str)]) -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    for (name, body) in entries {
        if name.ends_with('/') {
            writer.add_directory(*name, options).unwrap();
        } else {
            writer.start_file(*name, options).unwrap();
            writer.write_all(body.as_bytes()).unwrap();
        }
    }
    writer.finish().unwrap().into_inner()
}

/// A package valid in every other way whose `word/document.xml` local
/// header names the entry differently than the central directory: a tool
/// that trusts the local header and a reader that trusts the central
/// directory see two different packages, so a security review of one is
/// not a review of the other. Admission must refuse the package with a
/// typed code rather than pick a side.
#[test]
fn a_local_name_that_differs_from_the_central_directory_is_refused() {
    let bytes = dual_header_package();
    let err = jubarte::admission::admit(&bytes, small_limits())
        .expect_err("a dual-header package must not be admitted");
    assert_eq!(err.code(), "ZIP_HEADER_MISMATCH", "{err}");
}

#[test]
fn document_loading_refuses_a_dual_header_package() {
    let bytes = dual_header_package();
    let Err(err) = WmlDocument::from_bytes(&bytes) else {
        panic!("document loading must refuse a dual-header package through admission");
    };
    assert!(err.to_string().contains("ZIP_HEADER_MISMATCH"), "{err}");
}

#[test]
fn markdown_conversion_refuses_a_dual_header_package() {
    let bytes = dual_header_package();
    let Err(err) = jubarte::markdown::docx_to_markdown(&bytes, &Default::default()) else {
        panic!("Markdown conversion must refuse a dual-header package");
    };
    assert!(err.to_string().contains("ZIP_HEADER_MISMATCH"), "{err}");
}

#[test]
fn reading_without_layout_refuses_a_dual_header_package() {
    let bytes = dual_header_package();
    let options = jubarte::markdown::ReadOptions {
        page_markers: false,
        ..Default::default()
    };
    let Err(err) = jubarte::markdown::read(&bytes, &options) else {
        panic!("reading without layout must refuse a dual-header package");
    };
    assert!(err.to_string().contains("ZIP_HEADER_MISMATCH"), "{err}");
}

/// A part nested past the depth budget is refused with an error, never a
/// stack overflow (#1353 review): the view lists its changes with the
/// permissive parser, so only a package the tag policy admitted reaches
/// it. Release 0.12.0 refused this file the same way.
#[test]
fn reading_refuses_xml_nested_past_the_depth_budget() {
    let open = "<w:sdt><w:sdtContent>".repeat(60_000);
    let close = "</w:sdtContent></w:sdt>".repeat(60_000);
    let document = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{open}<w:p><w:r><w:t>x</w:t></w:r></w:p>{close}</w:body></w:document>"#
    );
    let bytes = package_with_document(&document);
    let options = jubarte::markdown::ReadOptions {
        page_markers: false,
        ..Default::default()
    };
    let Err(err) = jubarte::markdown::read(&bytes, &options) else {
        panic!("reading must refuse XML nested past the depth budget");
    };
    assert!(err.to_string().contains("nesting too deep"), "{err}");
}

/// A header or a footnote nested as deep is refused the same way: the
/// view's change listing reads every story (release 0.12.0 overflowed
/// its stack on these).
#[test]
fn reading_refuses_a_story_nested_past_the_depth_budget() {
    use common::docx::{Part, W_NS, docx_with, docx_with_sect};
    let open = "<w:sdt><w:sdtContent>".repeat(60_000);
    let close = "</w:sdtContent></w:sdt>".repeat(60_000);
    let deep = format!("{open}<w:p><w:r><w:t>x</w:t></w:r></w:p>{close}");
    let header = format!(r#"<w:hdr xmlns:w="{W_NS}">{deep}</w:hdr>"#);
    let footnotes = format!(
        r#"<w:footnotes xmlns:w="{W_NS}"><w:footnote w:id="1">{deep}</w:footnote></w:footnotes>"#
    );
    let with_header = docx_with_sect(
        "<w:p><w:r><w:t>Body</w:t></w:r></w:p>",
        &[Part {
            name: "word/header1.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
            xml: &header,
        }],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    );
    let with_note = docx_with(
        r#"<w:p><w:r><w:t>See</w:t></w:r><w:r><w:footnoteReference w:id="1"/></w:r></w:p>"#,
        &[Part {
            name: "word/footnotes.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes",
            xml: &footnotes,
        }],
    );
    let options = jubarte::markdown::ReadOptions {
        page_markers: false,
        ..Default::default()
    };
    for (part, bytes) in [("header1", with_header), ("footnotes", with_note)] {
        let Err(err) = jubarte::markdown::read(&bytes, &options) else {
            panic!("reading must refuse a {part} nested past the depth budget");
        };
        let err = err.to_string();
        assert!(
            err.contains(&format!("word/{part}.xml: XML nesting too deep")),
            "{err}"
        );
    }
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

/// Every public entry point that opens a package refuses it past the budget,
/// not only the compare path. The package constructor itself is bounded
/// (#645), so an entry point that forgets to admit still refuses; the
/// callers here admit on their own and their refusal is unchanged.
#[test]
fn every_package_entry_point_refuses_a_crowd_of_entries() {
    let crowded = package_with_entries(10_010);
    let refused = |what: &str, text: String| {
        assert!(text.contains("INPUT_LIMIT"), "{what}: {text}");
    };
    refused(
        "accept_revisions",
        jubarte::document_comparer::accept_revisions(&crowded)
            .expect_err("refused")
            .to_string(),
    );
    refused(
        "reject_revisions",
        jubarte::document_comparer::reject_revisions(&crowded)
            .expect_err("refused")
            .to_string(),
    );
    refused(
        "get_revisions",
        jubarte::document_comparer::get_revisions(&crowded, &WmlComparerSettings::default())
            .expect_err("refused")
            .to_string(),
    );
    refused(
        "list_changes",
        jubarte::changes::list_changes(&crowded)
            .expect_err("refused")
            .to_string(),
    );
    refused(
        "docx_to_pdf",
        jubarte::convert::docx_to_pdf(&crowded)
            .expect_err("refused")
            .to_string(),
    );
}

#[test]
fn the_entry_points_still_open_a_well_formed_package() {
    jubarte::document_comparer::accept_revisions(ORIGINAL).expect("accept");
    jubarte::document_comparer::reject_revisions(ORIGINAL).expect("reject");
    jubarte::changes::list_changes(ORIGINAL).expect("list");
}

/// `get_revisions` takes settings, so it admits under the caller's budget
/// rather than a fixed one.
#[test]
fn get_revisions_admits_under_the_settings_budget() {
    let tight = WmlComparerSettings::default().with_input_limits(InputLimits {
        max_entries: 2,
        ..InputLimits::compare()
    });
    let err = jubarte::document_comparer::get_revisions(ORIGINAL, &tight).expect_err("refused");
    assert!(err.to_string().contains("INPUT_LIMIT"), "{err}");
    jubarte::document_comparer::get_revisions(ORIGINAL, &WmlComparerSettings::default())
        .expect("the default budget admits the fixture");
}

/// The overrides the Python and WASM bindings accept: every key optional,
/// the rest taken from the base budget, unknown keys refused.
#[test]
fn limit_overrides_replace_only_the_keys_given() {
    use jubarte::admission::InputLimitOverrides;
    let base = InputLimits::compare();
    let none = InputLimitOverrides::from_json("{}").expect("empty object");
    assert_eq!(none.apply(base), base);
    let some = InputLimitOverrides::from_json(r#"{"max_entries": 7, "max_part_bytes": 1024}"#)
        .expect("two keys");
    assert_eq!(
        some.apply(base),
        InputLimits {
            max_entries: 7,
            max_part_bytes: 1024,
            ..base
        }
    );
    let typo = InputLimitOverrides::from_json(r#"{"max_entrys": 7}"#).expect_err("unknown key");
    assert!(typo.contains("max_entrys"), "{typo}");
    let negative =
        InputLimitOverrides::from_json(r#"{"max_entries": -1}"#).expect_err("negative count");
    assert!(negative.starts_with("invalid input limits"), "{negative}");
}

#[test]
fn wml_document_exposes_its_input_bytes_read_only() {
    let wml = WmlDocument::from_bytes(ORIGINAL).expect("open");
    assert_eq!(wml.bytes(), ORIGINAL);
}

/// The public document API must reject oversized OPC attributes before the
/// old dependency's quadratic duplicate check, while admitting the boundary.
#[test]
fn document_open_bounds_opc_metadata_attributes() {
    for count in [256, 257] {
        let attributes = (3..count)
            .map(|index| format!(" a{index}=\"x\""))
            .collect::<String>();
        let xml = format!(
            "<Relationships><Relationship Id=\"rId9\" Type=\"urn:test\" Target=\"document.xml\"{attributes}/></Relationships>"
        );
        let bytes = package(|writer, options| {
            writer
                .start_file("word/_rels/document.xml.rels", options)
                .unwrap();
            writer.write_all(xml.as_bytes()).unwrap();
        });
        let result = WmlDocument::from_bytes(&bytes);
        if count == 256 {
            assert!(result.is_ok());
        } else {
            let Err(err) = result else {
                panic!("oversized attributes admitted")
            };
            assert!(err.to_string().contains("256 attributes"), "{err}");
        }
    }
}

/// Package admission admitted unquoted and valueless attributes before it
/// checked attribute budgets (#1055), and the permissive parser reads them:
/// Word never writes them, but other writers' files carry them. They are
/// still admitted, and the opened document holds the DOM `parse_xdocument`
/// builds from the same part.
#[test]
fn admit_accepts_unquoted_and_valueless_attributes_the_permissive_parser_reads() {
    for (xml, label) in [
        (
            r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:rPr><w:b w:val=on/></w:rPr><w:t>hi</w:t></w:r></w:p></w:body></w:document>"#,
            "unquoted value on w:b",
        ),
        (
            r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:rPr><w:b valueless/></w:rPr><w:t>hi</w:t></w:r></w:p></w:body></w:document>"#,
            "valueless attribute on w:b",
        ),
    ] {
        let bytes = package_with_document(xml);
        let mut wml = WmlDocument::from_bytes(&bytes)
            .unwrap_or_else(|e| panic!("{label}: admit must accept, got {e}"));
        let admit_root = wml.main_document_root().expect("main root after admit");

        let admit_body = wml.dom().serialize_element(admit_root);
        let mut direct = jubarte::xmllinq::Dom::new();
        let direct_doc = direct.parse_xdocument(xml);
        let direct_root = direct.root(direct_doc).expect("direct parse root");
        let direct_body = direct.serialize_element(direct_root);
        assert_eq!(
            admit_body, direct_body,
            "{label}: admit-then-parse DOM must equal direct parse_xdocument DOM"
        );
    }
}

/// Unquoted attributes count against the same budgets: 1025 of them on one
/// tag exceed the attribute cap, and a repeated name is refused, because
/// the DOM keeps one slot per name and would drop a value silently.
#[test]
fn admit_keeps_refusing_over_the_count_cap_and_duplicates_in_html_mode() {
    let over_cap_attrs = (0..1025).map(|i| format!(" a{i}=v{i}")).collect::<String>();
    let over_cap_xml = format!(
        "<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><w:body{over_cap_attrs}></w:body></w:document>"
    );
    let bytes = package_with_document(&over_cap_xml);
    let Err(err) = WmlDocument::from_bytes(&bytes) else {
        panic!("1025 unquoted attributes must still be refused")
    };
    assert!(err.to_string().contains("1024 attributes"), "{err}");

    let dup_xml = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p a=1 a=2/></w:body></w:document>"#;
    let bytes = package_with_document(dup_xml);
    let Err(err) = WmlDocument::from_bytes(&bytes) else {
        panic!("duplicated attributes must still be refused")
    };
    assert!(err.to_string().contains("duplicated attribute"), "{err}");
}

/// The entry points that take no settings budget must refuse a crowd of
/// entries too (#645): `PartFs::open` is the package constructor behind
/// them, and it reads the entry count from the central directory before it
/// indexes the archive, so the refusal does not depend on a caller that
/// remembers to admit.
#[test]
fn partfs_open_refuses_a_crowd_of_entries() {
    let crowded = package_with_entries(10_010);
    let Err(err) = jubarte::opc::PartFs::open(&crowded) else {
        panic!("10_013 entries must not pass the constructor's own budget")
    };
    let text = err.to_string();
    assert!(text.contains("INPUT_LIMIT"), "{text}");
    assert!(text.contains("10013 ZIP entries"), "{text}");
}

/// `scrub` opens its package through the bounded constructor: a crowd is
/// refused before `scrub` inflates it (#645).
#[test]
fn scrub_refuses_a_crowd_of_entries() {
    let crowded = package_with_entries(10_010);
    let err = jubarte::scrub::scrub(&crowded, &jubarte::scrub::ScrubOptions::default())
        .expect_err("scrub reaches the package constructor, which refuses the crowd");
    assert!(err.to_string().contains("INPUT_LIMIT"), "{err}");
}

/// An excessive EOCD count must be refused before the indexer tries to
/// follow it. The base scrub path reaches the reader first and returns a ZIP
/// error; its later validation cannot protect that allocation (#645, #649).
#[test]
fn scrub_refuses_an_excess_declared_count_before_indexing() {
    let mut docx = package(|_, _| {});
    let end = docx.len() - 22;
    assert_eq!(&docx[end..end + 4], b"PK\x05\x06");
    for offset in [8, 10] {
        docx[end + offset..end + offset + 2].copy_from_slice(&10_013u16.to_le_bytes());
    }
    let err = jubarte::scrub::scrub(&docx, &jubarte::scrub::ScrubOptions::default())
        .expect_err("an excessive declared count must be refused before indexing");
    assert!(err.to_string().contains("INPUT_LIMIT"), "{err}");
    assert!(err.to_string().contains("10013 ZIP entries"), "{err}");
}

/// `leaks` has no refusal channel, so it fails closed: a package past the
/// admission budget is reported as a finding, never scanned partially
/// (#645). `edit`'s redaction check turns the finding into a refusal.
#[test]
fn leaks_fail_closed_on_a_crowd_of_entries() {
    let crowded = package_with_entries(10_010);
    let found = jubarte::scrub::leaks(&crowded, "no-such-text-occurs-here");
    assert_eq!(
        found,
        ["the package (INPUT_LIMIT: 10013 ZIP entries; the limit is 10000)".to_string()],
        "a package past the read budget fails closed"
    );
}

/// The Markdown reader admits its input instead of sniffing it (#645):
/// the refusal carries the stable `INPUT_LIMIT` code, not the converter's
/// own wording.
#[test]
fn docx_to_markdown_refuses_a_crowd_of_entries() {
    let crowded = package_with_entries(10_010);
    let err = jubarte::markdown::docx_to_markdown(
        &crowded,
        &jubarte::markdown::MarkdownOptions::default(),
    )
    .expect_err("the Markdown reader admits its input before converting it");
    let text = err.to_string();
    assert!(text.contains("INPUT_LIMIT"), "{text}");
    assert!(text.contains("10013 ZIP entries"), "{text}");
}

/// `debug` opens its package through the same budget (#645).
#[test]
fn debug_reports_refuse_a_crowd_of_entries() {
    let crowded = package_with_entries(10_010);
    let err = jubarte::debug::list(&crowded, None, &jubarte::debug::Options::default())
        .expect_err("debug opens its package through the same budget");
    assert!(err.contains("INPUT_LIMIT"), "{err}");
    assert!(err.contains("10013 ZIP entries"), "{err}");
}

/// The controls: a small valid package still opens through every entry
/// point the crowd tests refuse, and `leaks` still finds a text that
/// really occurs (#645).
#[test]
fn the_unadmitted_entry_points_still_open_a_valid_package() {
    let docx = package(|_, _| {});
    let opened = jubarte::opc::PartFs::open(&docx).expect("PartFs::open");
    assert_eq!(
        opened.main_document_part().as_deref(),
        Some("word/document.xml")
    );
    jubarte::scrub::scrub(&docx, &jubarte::scrub::ScrubOptions::default())
        .expect("scrub opens the package");
    assert_eq!(
        jubarte::scrub::leaks(&docx, "hello"),
        ["word/document.xml".to_string()],
        "a text that occurs is still found"
    );
    let read =
        jubarte::markdown::docx_to_markdown(&docx, &jubarte::markdown::MarkdownOptions::default())
            .expect("docx_to_markdown opens the package");
    assert!(read.markdown.contains("hello"), "{}", read.markdown);
    jubarte::debug::list(&docx, None, &jubarte::debug::Options::default())
        .expect("debug::list opens the package");
}

/// A package written through a non-seekable sink: every entry carries a
/// data descriptor (general-purpose bit 3) and its sizes live only in the
/// central directory. No fixture covers the shape; the bounded constructor
/// and the readers behind it must still open it (#645).
#[test]
fn the_entry_points_open_a_package_with_data_descriptors() {
    let mut writer = ZipWriter::new_stream(Vec::new());
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    for (name, body) in [
        ("[Content_Types].xml", CONTENT_TYPES),
        ("_rels/.rels", PACKAGE_RELS),
        ("word/document.xml", DOCUMENT),
    ] {
        writer.start_file(name, options).unwrap();
        writer.write_all(body.as_bytes()).unwrap();
    }
    let docx = writer.finish().unwrap().into_inner();
    // The fixture means what it says: bit 3 (0x0008) is set in the first
    // entry's local header flag field (offset 6).
    let flags = u16::from_le_bytes([docx[6], docx[7]]);
    assert!(
        flags & 0x0008 != 0,
        "the streamed entry must use a data descriptor"
    );

    let opened = jubarte::opc::PartFs::open(&docx).expect("PartFs::open");
    assert_eq!(
        opened.part_bytes("word/document.xml"),
        Some(DOCUMENT.as_bytes())
    );
    jubarte::scrub::scrub(&docx, &jubarte::scrub::ScrubOptions::default())
        .expect("scrub opens a data-descriptor package");
    assert_eq!(
        jubarte::scrub::leaks(&docx, "hello"),
        ["word/document.xml".to_string()]
    );
    let read =
        jubarte::markdown::docx_to_markdown(&docx, &jubarte::markdown::MarkdownOptions::default())
            .expect("docx_to_markdown opens a data-descriptor package");
    assert!(read.markdown.contains("hello"), "{}", read.markdown);
    jubarte::debug::list(&docx, None, &jubarte::debug::Options::default())
        .expect("debug::list opens a data-descriptor package");
}

/// A caller that already admitted a package under a roomier entry budget
/// keeps that budget through the identical-input drawing-ID fixup as well.
#[test]
fn an_explicit_roomier_count_budget_survives_identical_compare() {
    let docx = package_with_entries(10_010);
    let settings = WmlComparerSettings::default().with_input_limits(InputLimits {
        max_entries: 10_032,
        ..InputLimits::compare()
    });
    compare_documents_with_settings(&docx, &docx, &settings)
        .expect("all constructor calls must preserve the caller's admitted budget");
    jubarte::document_comparer::get_revisions(&docx, &settings)
        .expect("revision listing keeps the caller's budget too");
}

/// CRC-32 (IEEE, reflected) as the ZIP format writes it.
fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let bit = crc & 1;
            crc >>= 1;
            if bit != 0 {
                crc ^= 0xEDB8_8320;
            }
        }
    }
    !crc
}

/// A local file header (`PK\3\4`) for `name`: version 20, `flags`,
/// stored method, no time, the given CRC-32 and sizes.
fn local_header(name: &[u8], flags: u16, crc: u32, sizes: (u32, u32)) -> Vec<u8> {
    let mut header = vec![0x50, 0x4b, 0x03, 0x04];
    header.extend_from_slice(&20u16.to_le_bytes());
    header.extend_from_slice(&flags.to_le_bytes());
    header.extend_from_slice(&0u16.to_le_bytes());
    header.extend_from_slice(&[0; 4]);
    header.extend_from_slice(&crc.to_le_bytes());
    header.extend_from_slice(&sizes.0.to_le_bytes());
    header.extend_from_slice(&sizes.1.to_le_bytes());
    header.extend_from_slice(&u16::try_from(name.len()).unwrap().to_le_bytes());
    header.extend_from_slice(&0u16.to_le_bytes());
    header.extend_from_slice(name);
    header
}

/// The entry's local-header and central-record offsets.
fn header_offsets(bytes: &[u8], name: &str) -> (usize, usize) {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let entry = archive.by_name(name).unwrap();
    (
        usize::try_from(entry.header_start()).unwrap(),
        usize::try_from(entry.central_header_start()).unwrap(),
    )
}

/// A valid package whose `word/document.xml` local header declares a
/// different compression method than its central record (#644 item 1).
fn header_disagreement_probe() -> Vec<u8> {
    let mut bytes = package(|_, _| {});
    let (lfh, _) = header_offsets(&bytes, "word/document.xml");
    bytes[lfh + 8] ^= 0x08;
    bytes
}

/// A valid package whose `b.bin` central record points at a local header
/// embedded in `a.bin`'s payload, with all compared fields matching
/// (#644 item 2).
fn extent_probe() -> Vec<u8> {
    let mut embedded = local_header(b"b.bin", 0, crc32(b"x"), (1, 1));
    embedded.extend_from_slice(b"x");
    let mut bytes = package(|writer, options| {
        let stored = options.compression_method(CompressionMethod::Stored);
        for (name, data) in [("a.bin", embedded.as_slice()), ("b.bin", b"x" as &[u8])] {
            writer.start_file(name, stored).unwrap();
            writer.write_all(data).unwrap();
        }
    });
    let (a_lfh, _) = header_offsets(&bytes, "a.bin");
    let embedded_at = a_lfh + 30 + b"a.bin".len();
    assert_eq!(&bytes[embedded_at..embedded_at + 4], b"PK\x03\x04");
    let (_, b_central) = header_offsets(&bytes, "b.bin");
    bytes[b_central + 42..b_central + 46]
        .copy_from_slice(&u32::try_from(embedded_at).unwrap().to_le_bytes());
    bytes
}

/// A package whose entries carry data descriptors, the last one with a
/// corrupted CRC-32 (#644 item 3).
fn descriptor_probe(corrupt: bool) -> Vec<u8> {
    let mut writer = ZipWriter::new_stream(Vec::new());
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    for (name, body) in [
        ("[Content_Types].xml", CONTENT_TYPES),
        ("_rels/.rels", PACKAGE_RELS),
        ("word/document.xml", DOCUMENT),
    ] {
        writer.start_file(name, options).unwrap();
        writer.write_all(body.as_bytes()).unwrap();
    }
    let mut bytes = writer.finish().unwrap().into_inner();
    if corrupt {
        let last = bytes
            .windows(4)
            .rposition(|w| w == b"PK\x07\x08")
            .expect("signed descriptor");
        bytes[last + 4] ^= 0xff;
    }
    bytes
}

/// A package whose directory entry carries a payload with a corrupt
/// CRC-32 (#644 item 4).
fn directory_probe() -> Vec<u8> {
    let mut bytes = package(|writer, options| {
        writer
            .start_file(
                "word/medi0",
                options.compression_method(CompressionMethod::Stored),
            )
            .unwrap();
        writer.write_all(b"0").unwrap();
    });
    // Rename "word/medi0" to "word/medi/" in both headers.
    let (lfh, central) = header_offsets(&bytes, "word/medi0");
    bytes[lfh + 30..lfh + 40].copy_from_slice(b"word/medi/");
    bytes[central + 46..central + 56].copy_from_slice(b"word/medi/");
    let at = lfh + 30 + b"word/medi/".len();
    bytes[at] = b'7';
    bytes
}

/// Overwrite `entry`'s central record: `version_made_by` at +4 and the
/// external attributes at +38 (#639).
fn set_central_attributes(bytes: &mut [u8], entry: &str, version_made_by: u16, external: u32) {
    let at = {
        let mut archive = zip::ZipArchive::new(Cursor::new(&*bytes)).unwrap();
        usize::try_from(archive.by_name(entry).unwrap().central_header_start()).unwrap()
    };
    assert_eq!(&bytes[at..at + 4], b"PK\x01\x02");
    bytes[at + 4..at + 6].copy_from_slice(&version_made_by.to_le_bytes());
    bytes[at + 38..at + 42].copy_from_slice(&external.to_le_bytes());
}

/// A valid package whose `x.bin` central record marks it a Unix symlink
/// (#639).
fn special_entry_probe() -> Vec<u8> {
    let mut bytes = package(|writer, options| {
        writer
            .start_file(
                "x.bin",
                options.compression_method(CompressionMethod::Stored),
            )
            .unwrap();
        writer.write_all(b"link-target").unwrap();
    });
    set_central_attributes(&mut bytes, "x.bin", 0x0314, 0o120777 << 16);
    bytes
}

/// ZIP-17 (#644): every public route applies the structural refusals the
/// bounded constructor owns — direct admission, the package
/// constructor, scrub/leaks, the Markdown reader — while `debug`
/// reports each defect as a finding so a refused file can be triaged.
#[test]
fn zip17_public_boundaries() {
    for (what, bytes, code, finding) in [
        (
            "header field",
            header_disagreement_probe(),
            "ZIP_HEADER_MISMATCH",
            "ZIP_HEADER_MISMATCH",
        ),
        ("extent", extent_probe(), "INVALID_PACKAGE", "ZIP_EXTENT"),
        (
            "descriptor",
            descriptor_probe(true),
            "INVALID_PACKAGE",
            "ZIP_DESCRIPTOR",
        ),
        (
            "directory payload",
            directory_probe(),
            "INVALID_PACKAGE",
            "ZIP_DIRECTORY_PAYLOAD",
        ),
        (
            "special entry",
            special_entry_probe(),
            "UNSUPPORTED_PACKAGE",
            "ZIP_SPECIAL_ENTRY",
        ),
        // ZIP-15 (#600): two entries naming one part — Word refuses each
        // of these shapes, so every route refuses them with DUPLICATE_PART
        // and `debug` reports the defect for triage.
        (
            "exact duplicate part",
            duplicate_probe("word/document.xm0", "word/document.xml"),
            "DUPLICATE_PART",
            "ZIP_DUPLICATE_PART",
        ),
        (
            "case-aliased main part",
            duplicate_probe("word/document.xm0", "word/Document.xml"),
            "DUPLICATE_PART",
            "ZIP_DUPLICATE_PART",
        ),
        (
            "upper-case main part",
            duplicate_probe("word/document.xm0", "WORD/document.xml"),
            "DUPLICATE_PART",
            "ZIP_DUPLICATE_PART",
        ),
        (
            "case-aliased content types",
            duplicate_probe("[Content_Types].xmx", "[content_types].xml"),
            "DUPLICATE_PART",
            "ZIP_DUPLICATE_PART",
        ),
        (
            "case-aliased package rels",
            duplicate_probe("xrels/.rels", "_RELS/.rels"),
            "DUPLICATE_PART",
            "ZIP_DUPLICATE_PART",
        ),
    ] {
        let err = jubarte::admission::admit(&bytes, jubarte::admission::InputLimits::compare())
            .expect_err("{what} probe must be refused");
        assert_eq!(err.code(), code, "{what}: {err}");
        if finding == "ZIP_DUPLICATE_PART" {
            // The package constructor reports the one OPC defect as the
            // typed variant its callers match on (#600).
            let Err(jubarte::opc::OpcError::DuplicatePartName(_)) =
                jubarte::opc::PartFs::open(&bytes)
            else {
                panic!("{what}: PartFs must refuse with DuplicatePartName");
            };
        } else {
            let route = jubarte::opc::PartFs::open(&bytes)
                .map(|_| ())
                .map_err(|e| e.to_string())
                .expect_err("{what} PartFs");
            assert!(route.contains(code), "{what}: {route}");
        }
        for route in [
            jubarte::scrub::scrub(&bytes, &jubarte::scrub::ScrubOptions::default())
                .map(|_| ())
                .map_err(|e| e.to_string())
                .expect_err("{what} scrub"),
            jubarte::markdown::docx_to_markdown(
                &bytes,
                &jubarte::markdown::MarkdownOptions::default(),
            )
            .map(|_| ())
            .map_err(|e| e.to_string())
            .expect_err("{what} markdown"),
        ] {
            assert!(route.contains(code), "{what}: {route}");
        }
        let found = jubarte::scrub::leaks(&bytes, "no-such-text-occurs-here");
        assert!(!found.is_empty(), "{what}: leaks must fail closed");
        if finding == "ZIP_DUPLICATE_PART" {
            assert!(
                found.iter().any(|f| f.contains(code)),
                "{what}: leaks must name the refusal code: {found:?}"
            );
        }
        let findings =
            jubarte::debug::findings(&bytes).expect("debug opens a package every route refuses");
        assert!(
            findings.iter().any(|f| f.code == finding),
            "{what}: {findings:?}"
        );
    }
    // The retained positive row: a valid descriptor package opens
    // through every route.
    let control = descriptor_probe(false);
    jubarte::admission::admit(&control, jubarte::admission::InputLimits::compare())
        .expect("a descriptor package admits");
    jubarte::opc::PartFs::open(&control).expect("PartFs opens it");
    jubarte::scrub::scrub(&control, &jubarte::scrub::ScrubOptions::default())
        .expect("scrub opens it");
    jubarte::markdown::docx_to_markdown(&control, &jubarte::markdown::MarkdownOptions::default())
        .expect("markdown opens it");
    jubarte::debug::findings(&control).expect("debug opens it");

    // ZIP-15 (#600) controls: OPC part names ignore ASCII case
    // (ECMA-376 Part 2 §6.2.2.3 M1.12), so a single case-variant name IS
    // the part — Word opens each of these packages (probes p3, p4, p5,
    // p8), and every route must read the variant's content.
    let document = |text: &str| {
        format!(
            "<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
            <w:body><w:p><w:r><w:t>{text}</w:t></w:r></w:p><w:sectPr/></w:body></w:document>"
        )
    };
    for (what, text, bytes, stored) in [
        (
            "case-variant main part (p3)",
            "variant main",
            package_named(&[
                ("[Content_Types].xml", CONTENT_TYPES),
                ("_rels/.rels", PACKAGE_RELS),
                ("word/Document.xml", &document("variant main")),
            ]),
            "word/Document.xml",
        ),
        (
            "lower-case content types (p4)",
            "lower content types",
            package_named(&[
                ("[content_types].xml", CONTENT_TYPES),
                ("_rels/.rels", PACKAGE_RELS),
                ("word/document.xml", &document("lower content types")),
            ]),
            "word/document.xml",
        ),
        (
            "upper-case package rels (p5)",
            "upper rels",
            package_named(&[
                ("[Content_Types].xml", CONTENT_TYPES),
                ("_RELS/.RELS", PACKAGE_RELS),
                ("word/document.xml", &document("upper rels")),
            ]),
            "word/document.xml",
        ),
        (
            "directory entry beside its part (p8)",
            "beside its directory",
            package_named(&[
                ("[Content_Types].xml", CONTENT_TYPES),
                ("_rels/.rels", PACKAGE_RELS),
                ("word/", ""),
                ("word/document.xml", &document("beside its directory")),
            ]),
            "word/document.xml",
        ),
    ] {
        let admitted =
            jubarte::admission::admit(&bytes, jubarte::admission::InputLimits::compare())
                .unwrap_or_else(|err| panic!("{what} must admit: {err}"));
        assert_eq!(admitted.main_part, stored, "{what}");
        let fs = jubarte::opc::PartFs::open(&bytes).unwrap_or_else(|err| panic!("{what}: {err}"));
        let found_text = fs
            .part_string("word/document.xml")
            .unwrap_or_else(|| panic!("{what}: the main part resolves under one identity"));
        assert!(found_text.contains(text), "{what}: {found_text}");
        assert_eq!(
            fs.main_document_part().as_deref(),
            Some("word/document.xml"),
            "{what}: the relationship target is reported verbatim"
        );
        jubarte::scrub::scrub(&bytes, &jubarte::scrub::ScrubOptions::default())
            .unwrap_or_else(|err| panic!("{what}: scrub opens it: {err}"));
        let read = jubarte::markdown::docx_to_markdown(
            &bytes,
            &jubarte::markdown::MarkdownOptions::default(),
        )
        .unwrap_or_else(|err| panic!("{what}: markdown opens it: {err}"));
        assert!(read.markdown.contains(text), "{what}: {}", read.markdown);
        jubarte::debug::findings(&bytes)
            .unwrap_or_else(|err| panic!("{what}: debug opens it: {err}"));
        let found = jubarte::scrub::leaks(&bytes, text);
        assert!(!found.is_empty(), "{what}: leaks finds the variant part");
    }

    // ZIP-10 (#604): the pre-index tail checks refuse hostile ZIP64 shapes
    // on every route — the declared-count pass runs before any indexing, so
    // even `debug`, which reports structural defects as findings, fails
    // here. A valid forced-ZIP64 package is the control: `large_file(true)`
    // writes 0xFFFFFFFF sizes plus the 0x0001 field and version-needed 45,
    // and every route resolves it through the ZIP64 extra field.
    for (what, bytes, code) in [
        (
            "ZIP64 count past the budget",
            zip64_tail_probe(10_013, 0),
            "INPUT_LIMIT",
        ),
        (
            "ZIP64 end record past the input",
            zip64_tail_probe(0, 1_000_000),
            "INVALID_PACKAGE",
        ),
        (
            "ZIP32 count sentinel without a locator",
            zip64_sentinel_count_probe(),
            "INVALID_PACKAGE",
        ),
    ] {
        let err = jubarte::admission::admit(&bytes, jubarte::admission::InputLimits::compare())
            .expect_err("{what} probe must be refused");
        assert_eq!(err.code(), code, "{what}: {err}");
        for route in [
            jubarte::opc::PartFs::open(&bytes)
                .map(|_| ())
                .map_err(|e| e.to_string())
                .expect_err("{what} PartFs"),
            jubarte::scrub::scrub(&bytes, &jubarte::scrub::ScrubOptions::default())
                .map(|_| ())
                .map_err(|e| e.to_string())
                .expect_err("{what} scrub"),
            jubarte::markdown::docx_to_markdown(
                &bytes,
                &jubarte::markdown::MarkdownOptions::default(),
            )
            .map(|_| ())
            .map_err(|e| e.to_string())
            .expect_err("{what} markdown"),
            jubarte::debug::findings(&bytes)
                .map(|_| ())
                .expect_err("{what} debug"),
        ] {
            assert!(route.contains(code), "{what}: {route}");
        }
        let found = jubarte::scrub::leaks(&bytes, "no-such-text-occurs-here");
        assert!(!found.is_empty(), "{what}: leaks must fail closed");
    }
    let forced = zip64_forced_probe();
    jubarte::admission::admit(&forced, jubarte::admission::InputLimits::compare())
        .expect("a forced-ZIP64 package admits");
    jubarte::opc::PartFs::open(&forced).expect("PartFs opens it");
    jubarte::scrub::scrub(&forced, &jubarte::scrub::ScrubOptions::default())
        .expect("scrub opens it");
    jubarte::markdown::docx_to_markdown(&forced, &jubarte::markdown::MarkdownOptions::default())
        .expect("markdown opens it");
    jubarte::debug::findings(&forced).expect("debug opens it");
}

/// A 56-byte ZIP64 end-of-central-directory record (APPNOTE 4.3.14)
/// declaring `entries`.
fn eocd64(entries: u64) -> Vec<u8> {
    let mut record = vec![0x50, 0x4b, 0x06, 0x06];
    record.extend_from_slice(&44u64.to_le_bytes());
    record.extend_from_slice(&[45, 0, 45, 0]);
    record.extend_from_slice(&0u32.to_le_bytes());
    record.extend_from_slice(&0u32.to_le_bytes());
    record.extend_from_slice(&entries.to_le_bytes());
    record.extend_from_slice(&entries.to_le_bytes());
    record.extend_from_slice(&0u64.to_le_bytes());
    record.extend_from_slice(&0u64.to_le_bytes());
    assert_eq!(record.len(), 56);
    record
}

/// A valid package whose tail is rewritten ZIP64-shaped: the EOCD's count
/// fields become the 0xFFFF sentinel, a 20-byte locator (APPNOTE 4.3.15)
/// points at a real end record inserted before it declaring `entries`, and
/// `offset_extra` moves the locator's record offset that many bytes — past
/// the input when large (#604).
fn zip64_tail_probe(entries: u64, offset_extra: u64) -> Vec<u8> {
    let bytes = package(|_, _| {});
    let eocd_at = bytes.len() - 22;
    assert_eq!(&bytes[eocd_at..eocd_at + 4], b"PK\x05\x06");
    let mut out = bytes[..eocd_at].to_vec();
    out.extend_from_slice(&eocd64(entries));
    let record_at = u64::try_from(out.len()).unwrap() - 56;
    out.extend_from_slice(&[0x50, 0x4b, 0x06, 0x07]);
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&(record_at + offset_extra).to_le_bytes());
    out.extend_from_slice(&1u32.to_le_bytes());
    let mut eocd = bytes[eocd_at..].to_vec();
    eocd[8..10].copy_from_slice(&u16::MAX.to_le_bytes());
    eocd[10..12].copy_from_slice(&u16::MAX.to_le_bytes());
    out.extend_from_slice(&eocd);
    out
}

/// A valid package whose EOCD count fields carry the 0xFFFF sentinel but
/// no ZIP64 locator precedes them (#604): the declared count cannot be
/// trusted, so the package is refused rather than indexed.
fn zip64_sentinel_count_probe() -> Vec<u8> {
    let mut bytes = package(|_, _| {});
    let eocd_at = bytes.len() - 22;
    assert_eq!(&bytes[eocd_at..eocd_at + 4], b"PK\x05\x06");
    bytes[eocd_at + 8..eocd_at + 10].copy_from_slice(&u16::MAX.to_le_bytes());
    bytes[eocd_at + 10..eocd_at + 12].copy_from_slice(&u16::MAX.to_le_bytes());
    bytes
}

/// A valid small package written `large_file(true)`: every entry carries
/// the 0x0001 ZIP64 extra field, 0xFFFFFFFF size sentinels and
/// version-needed 45 — the shape ZIP-10 (#604) refuses to write but every
/// route must read (#604's descriptor/UTF-16-style control).
fn zip64_forced_probe() -> Vec<u8> {
    package(|writer, _| {
        writer
            .start_file(
                "word/media/wide.bin",
                SimpleFileOptions::default()
                    .compression_method(CompressionMethod::Stored)
                    .large_file(true),
            )
            .unwrap();
        writer.write_all(b"wide").unwrap();
    })
}

#[test]
fn issue650_defaults_and_overrides_reach_admission() {
    use jubarte::admission::{AdmissionErrorKind, InputLimitOverrides, admit};
    for count in [1024, 1025] {
        let attrs: String = (0..count).map(|i| format!(" a{i}='v'")).collect();
        let xml = DOCUMENT.replace("<w:p>", &format!("<w:p{attrs}>"));
        let bytes = package_with_document(&xml);
        let result = admit(&bytes, InputLimits::default());
        assert_eq!(result.is_ok(), count == 1024, "count {count}: {result:?}");
        if let Err(err) = result {
            assert_eq!(err.kind, AdmissionErrorKind::InvalidXml);
            assert!(err.to_string().contains("word/document.xml"));
            assert!(err.to_string().contains("w:p"));
            assert!(err.to_string().contains("1024"));
        }
    }
    let overrides = InputLimitOverrides::from_json(
        r#"{"max_element_attributes": 3,"max_attribute_name_bytes": 128,"max_attribute_value_bytes": 100,"max_start_tag_bytes": 600}"#,
    );
    assert!(
        overrides.is_ok(),
        "attribute budgets must be configurable: {overrides:?}"
    );
    let limits = overrides.unwrap().apply(InputLimits::default());
    for (attrs, accepted) in [
        (" a='1' b='2' c='3'", true),
        (" a='1' b='2' c='3' d='4'", false),
    ] {
        let xml = DOCUMENT.replace("<w:p>", &format!("<w:p{attrs}>"));
        let result = admit(&package_with_document(&xml), limits);
        assert_eq!(result.is_ok(), accepted, "{result:?}");
        let parsed = jubarte::xmllinq::parse::try_parse_xdocument_with_limits(
            &mut jubarte::xmllinq::Dom::new(),
            &xml,
            limits,
        );
        assert_eq!(parsed.is_ok(), accepted, "{parsed:?}");
        if let Err(err) = result {
            assert_eq!(err.kind, AdmissionErrorKind::InvalidXml);
            assert!(err.to_string().contains("w:p"));
            assert!(err.to_string().contains("3 attributes"));
        }
    }
}

fn issue650_byte_boundary(key: &str, cap: usize) {
    use jubarte::admission::{InputLimitOverrides, admit};
    {
        let json = format!(r#"{{"{key}":{cap}}}"#);
        let parsed = InputLimitOverrides::from_json(&json);
        assert!(parsed.is_ok(), "{json}: {parsed:?}");
        let limits = parsed.unwrap().apply(InputLimits::default());
        for extra in [0, 1] {
            let tag = match key {
                "max_attribute_name_bytes" => format!("<w:p {}='v'>", "n".repeat(cap + extra)),
                "max_attribute_value_bytes" => format!("<w:p a='{}'>", "v".repeat(cap + extra)),
                _ => format!("<w:p a='{}'>", "v".repeat(cap + extra - 10)),
            };
            let xml = DOCUMENT.replace("<w:p>", &tag);
            let checked = jubarte::xmllinq::parse::validate_xml_with_limits(&xml, limits);
            let parsed = jubarte::xmllinq::parse::try_parse_xdocument_with_limits(
                &mut jubarte::xmllinq::Dom::new(),
                &xml,
                limits,
            );
            assert_eq!(checked.is_ok(), extra == 0, "checked {key}: {checked:?}");
            assert_eq!(parsed.is_ok(), extra == 0, "DOM {key}: {parsed:?}");
            let result = admit(&package_with_document(&xml), limits);
            assert_eq!(
                result.is_ok(),
                extra == 0,
                "{key} at {}: {result:?}",
                cap + extra
            );
        }
    }
}

#[test]
fn issue650_configurable_name_boundary() {
    issue650_byte_boundary("max_attribute_name_bytes", 100);
}
#[test]
fn issue650_configurable_value_boundary() {
    issue650_byte_boundary("max_attribute_value_bytes", 100);
}
#[test]
fn issue650_configurable_tag_boundary() {
    issue650_byte_boundary("max_start_tag_bytes", 600);
}

#[test]
fn issue650_cli_read_and_validate_enforce_attribute_policy() {
    let dir = tempfile::tempdir().unwrap();
    for (label, attributes, accepted) in [
        (
            "wide",
            (0..1024)
                .map(|i| format!(" a{i}='v{i}'"))
                .collect::<String>(),
            true,
        ),
        (
            "over-count",
            (0..1025).map(|i| format!(" a{i}='v{i}'")).collect(),
            false,
        ),
        ("over-name", format!(" {}='v'", "n".repeat(1025)), false),
        (
            "alias",
            " xmlns:a='urn:x' xmlns:b='urn:x' a:x='1' b:x='2'".to_string(),
            false,
        ),
    ] {
        let xml = DOCUMENT.replace("<w:p>", &format!("<w:p{attributes}>"));
        let file = format!("{label}.docx");
        std::fs::write(dir.path().join(&file), package_with_document(&xml)).unwrap();
        for command in ["read", "validate"] {
            let output = std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
                .current_dir(dir.path())
                .args([command, &file, "--json"])
                .output()
                .unwrap();
            let text = format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(
                output.status.success(),
                accepted,
                "{command} {label}: {text}"
            );
            if !accepted {
                assert!(text.contains("INVALID_XML"), "{command} {label}: {text}");
            }
        }
    }
}

#[test]
fn issue650_attribute_overrides_are_recognized() {
    for key in [
        "max_element_attributes",
        "max_attribute_name_bytes",
        "max_attribute_value_bytes",
        "max_start_tag_bytes",
    ] {
        let json = format!(r#"{{"{key}":1024}}"#);
        let parsed = jubarte::admission::InputLimitOverrides::from_json(&json);
        assert!(parsed.is_ok(), "{key} must be configurable: {parsed:?}");
    }
}

#[test]
fn issue650_raised_attribute_limits_survive_changed_compare() {
    use jubarte::admission::{InputLimitOverrides, admit};
    use jubarte::xmllinq::{Dom, XName, parse::try_parse_xdocument_with_limits};
    let base = WmlComparerSettings::default();
    let overrides = InputLimitOverrides::from_json(
        r#"{"max_element_attributes":4096,"max_start_tag_bytes":268435456}"#,
    )
    .unwrap();
    let limits = overrides.apply(base.input_limits);
    let settings = base.with_input_limits(limits);
    let attrs: String = (0..2000).map(|i| format!(" c:a{i}='v{i}'")).collect();
    let xml = DOCUMENT
        .replace("<w:p>", &format!("<w:p{attrs}>"))
        .replace("<w:document ", "<w:document xmlns:c='urn:issue650' ");
    let original = package_with_document(&xml.replace("hello", "Alice greets Bob"));
    let modified = package_with_document(&xml.replace("hello", "Alice greets Carol Diaz"));
    admit(&original, limits).expect("active limits admit original");
    admit(&modified, limits).expect("active limits admit modified");
    let compared = std::panic::catch_unwind(|| {
        compare_documents_with_settings(&original, &modified, &settings)
    });
    assert!(
        compared.is_ok(),
        "comparison with active limits must return without a panic"
    );
    let output = compared
        .unwrap()
        .expect("comparison with active limits succeeds");
    let patch = jubarte::markdown::patch_documents(
        jubarte::markdown::Source::Docx(&original),
        jubarte::markdown::Source::Docx(&modified),
        &jubarte::markdown::RedlineOptions {
            settings: settings.clone(),
            ..Default::default()
        },
        &jubarte::markdown::PatchOptions {
            old_name: "Alice.docx".into(),
            new_name: "Bob.docx".into(),
            owner: jubarte::markdown::Attribution {
                author: "Carol Diaz".into(),
                date: "2026-10-18T00:00:00Z".into(),
            },
        },
    );
    assert!(
        patch.is_ok(),
        "Markdown patch must retain active comparison limits: {patch:?}"
    );
    let package = jubarte::opc::PartFs::open(&output).unwrap();
    let xml = package.part_string("word/document.xml").unwrap();
    let mut dom = Dom::new();
    let doc = try_parse_xdocument_with_limits(&mut dom, &xml, limits).unwrap();
    let ns = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
    assert!(
        !dom.descendants(doc, Some(&XName::get("ins", ns)))
            .is_empty(),
        "text change must produce an insertion"
    );
    let paragraphs = dom.descendants(doc, Some(&XName::get("p", ns)));
    assert!(!paragraphs.is_empty());
    for paragraph in paragraphs {
        // Namespaced extensions survive; inherited-validity repair removes unqualified attributes.
        // The comparer can add revision metadata; assert every source attribute survives.
        for i in 0..2000 {
            assert_eq!(
                dom.attribute(paragraph, &XName::get(&format!("a{i}"), "urn:issue650")),
                Some(format!("v{i}").as_str())
            );
        }
    }
}

#[test]
fn issue650_markdown_converts_default_and_explicit_raised_widths() {
    use jubarte::admission::{InputLimitOverrides, admit};
    let raised = InputLimitOverrides::from_json(
        r#"{"max_element_attributes":4096,"max_start_tag_bytes":268435456}"#,
    )
    .unwrap()
    .apply(InputLimits::default());
    for count in [1024, 2000] {
        let attrs: String = (0..count).map(|i| format!(" a{i}='v{i}'")).collect();
        let xml = DOCUMENT
            .replace("<w:p>", &format!("<w:p{attrs}>"))
            .replace("hello", "Alice greets Carol Diaz");
        let bytes = package_with_document(&xml);
        let limits = if count == 1024 {
            InputLimits::default()
        } else {
            raised
        };
        admit(&bytes, limits).expect("package admitted under selected limits");
        let converted = std::panic::catch_unwind(|| {
            jubarte::markdown::docx_to_markdown(
                &bytes,
                &jubarte::markdown::MarkdownOptions {
                    input_limits: limits,
                    ..Default::default()
                },
            )
        });
        assert!(
            converted.is_ok(),
            "converter must return without a panic for count {count}"
        );
        let converted = converted.unwrap();
        assert!(
            converted.is_ok(),
            "converter must accept admitted count {count}: {converted:?}"
        );
        assert!(
            converted
                .unwrap()
                .markdown
                .contains("Alice greets Carol Diaz")
        );
    }
}

#[test]
fn issue650_markdown_refuses_default_attribute_count() {
    let attrs: String = (0..1025).map(|i| format!(" a{i}='Alice'")).collect();
    assert_markdown_policy_refusal(
        &DOCUMENT.replace("<w:p>", &format!("<w:p{attrs}>")),
        "attributes",
    );
}

#[test]
fn issue650_markdown_refuses_default_attribute_name() {
    let name = "n".repeat(1025);
    assert_markdown_policy_refusal(
        &DOCUMENT.replace("<w:p>", &format!("<w:p {name}='Bob'>")),
        "attribute name",
    );
}

#[test]
fn issue650_markdown_refuses_aliased_duplicate() {
    assert_markdown_policy_refusal(
        &DOCUMENT.replace(
            "<w:p>",
            "<w:p xmlns:a='urn:alias' xmlns:b='urn:alias' a:x='Alice' b:x='Bob'>",
        ),
        "expanded name",
    );
}

fn assert_markdown_policy_refusal(xml: &str, reason: &str) {
    let bytes = package_with_document(xml);
    let result = jubarte::markdown::docx_to_markdown(&bytes, &Default::default());
    assert!(
        result.is_err(),
        "Markdown must refuse {reason} before constructing attributes"
    );
    let error = result.unwrap_err().to_string();
    assert!(
        error.contains("INVALID_XML")
            && error.contains("word/document.xml")
            && error.contains(reason),
        "{error}"
    );
}

#[test]
fn issue650_markdown_refuses_over_cap_auxiliary_part() {
    let attrs: String = (0..1025).map(|i| format!(" a{i}='Alice'")).collect();
    let styles = format!(
        "<w:styles xmlns:w='http://schemas.openxmlformats.org/wordprocessingml/2006/main'{attrs}/>"
    );
    let bytes = package(|writer, options| {
        writer.start_file("word/styles.xml", options).unwrap();
        writer.write_all(styles.as_bytes()).unwrap();
    });
    let result = jubarte::markdown::docx_to_markdown(&bytes, &Default::default());
    assert!(
        result.is_err(),
        "over-cap auxiliary parts must refuse the whole conversion"
    );
    let error = result.unwrap_err().to_string();
    assert!(
        error.contains("INVALID_XML") && error.contains("word/styles.xml"),
        "{error}"
    );
}

#[test]
fn issue650_markdown_honors_explicit_tighter_budgets() {
    use jubarte::markdown::{MarkdownOptions, docx_to_markdown};
    for limits in [
        InputLimits {
            max_element_attributes: 1,
            ..Default::default()
        },
        InputLimits {
            max_attribute_name_bytes: 2,
            ..Default::default()
        },
        InputLimits {
            max_attribute_value_bytes: 2,
            ..Default::default()
        },
        InputLimits {
            max_start_tag_bytes: 12,
            ..Default::default()
        },
    ] {
        let bytes = package_with_document(DOCUMENT);
        let result = docx_to_markdown(
            &bytes,
            &MarkdownOptions {
                input_limits: limits,
                ..Default::default()
            },
        );
        assert!(
            result.is_err(),
            "explicit tighter tag budget must refuse: {limits:?}"
        );
        assert!(result.unwrap_err().to_string().contains("INVALID_XML"));
    }
}

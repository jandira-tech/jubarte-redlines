// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Shared integration-test helpers: Ring-1 Word-validity gates and the
//! canonical DOCX structural comparator (M0.3).
//!
//! The comparator compares two `.docx` byte streams for *structural* equality,
//! tolerating volatile bits that don't affect document meaning:
//!   - `w:rsid*` attributes (revision-save IDs)
//!   - `w14:paraId` / `w14:textId` (paragraph/text identity)
//!   - any `pt14:*` PowerTools attribute (correlation glue) + its xmlns decl
//!   - the numeric value of `w:id` on tracked-revision elements (renumbered into
//!     document order so two structurally-identical redlines compare equal)
//!   - insignificant (whitespace-only) text nodes / pretty-print indentation
//!   - attribute ordering (sorted)
//!
//! For M0 this is a minimal `quick-xml`-based canonicalizer; M1.6 re-points it
//! onto the `xmllinq` arena DOM once that exists. XML/`.rels` parts are
//! canonicalized; all other parts are compared byte-for-byte.

#![allow(dead_code)]

pub mod validity;

use std::collections::BTreeMap;
use std::io::{Cursor, Read};

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

/// Tracked-revision elements that carry a volatile `w:id`.
const REVISION_ELEMENTS: &[&str] = &[
    "ins",
    "del",
    "moveFrom",
    "moveTo",
    "moveFromRangeStart",
    "moveFromRangeEnd",
    "moveToRangeStart",
    "moveToRangeEnd",
    "rPrChange",
    "pPrChange",
    "tblPrChange",
    "trPrChange",
    "tcPrChange",
    "sectPrChange",
    "tblGridChange",
    "customXmlInsRangeStart",
    "customXmlInsRangeEnd",
    "customXmlDelRangeStart",
    "customXmlDelRangeEnd",
    "customXmlMoveFromRangeStart",
    "customXmlMoveFromRangeEnd",
    "customXmlMoveToRangeStart",
    "customXmlMoveToRangeEnd",
];

fn local_name(qname: &str) -> &str {
    match qname.rfind(':') {
        Some(i) => &qname[i + 1..],
        None => qname,
    }
}

/// True for attributes whose value is volatile and must be dropped.
fn is_volatile_attr(qname: &str) -> bool {
    qname.starts_with("w:rsid")
        || qname == "w14:paraId"
        || qname == "w14:textId"
        || qname.starts_with("pt14:")
        || qname == "xmlns:pt14"
}

/// Read a docx into a sorted map of part-name -> raw bytes.
fn read_parts(bytes: &[u8]) -> BTreeMap<String, Vec<u8>> {
    let mut archive =
        zip::ZipArchive::new(Cursor::new(bytes)).expect("input is not a valid zip/docx");
    let mut parts = BTreeMap::new();
    for i in 0..archive.len() {
        let mut f = archive.by_index(i).expect("zip entry");
        if f.is_dir() {
            continue;
        }
        let name = f.name().to_string();
        let mut buf = Vec::with_capacity(f.size() as usize);
        f.read_to_end(&mut buf).expect("read zip entry");
        parts.insert(name, buf);
    }
    parts
}

fn is_xml_part(name: &str) -> bool {
    name.ends_with(".xml") || name.ends_with(".rels")
}

/// Canonicalize one XML part into a normalized string.
fn canonicalize_xml(bytes: &[u8]) -> String {
    let mut reader = Reader::from_reader(bytes);
    let cfg = reader.config_mut();
    cfg.trim_text(false);
    cfg.expand_empty_elements = false;

    let mut out = String::new();
    let mut rev_counter: u64 = 0;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                emit_start(&mut out, &e, false, &mut rev_counter);
            }
            Ok(Event::Empty(e)) => {
                emit_start(&mut out, &e, true, &mut rev_counter);
            }
            Ok(Event::End(e)) => {
                out.push_str("</");
                out.push_str(&String::from_utf8_lossy(e.name().as_ref()));
                out.push('>');
            }
            Ok(Event::Text(e)) => {
                let raw = e.into_inner();
                let s = String::from_utf8_lossy(&raw);
                if !s.trim().is_empty() {
                    // significant text — keep verbatim (leading/trailing spaces matter in w:t)
                    out.push_str(&s);
                }
            }
            Ok(Event::CData(e)) => {
                out.push_str("<![CDATA[");
                out.push_str(&String::from_utf8_lossy(&e.into_inner()));
                out.push_str("]]>");
            }
            Ok(Event::GeneralRef(e)) => {
                // Entity / character reference (e.g. &amp;, &#65;) — re-emit verbatim.
                out.push('&');
                out.push_str(&String::from_utf8_lossy(&e.into_inner()));
                out.push(';');
            }
            // Drop decl / comments / PIs / doctype — not structurally meaningful.
            Ok(Event::Decl(_))
            | Ok(Event::Comment(_))
            | Ok(Event::PI(_))
            | Ok(Event::DocType(_)) => {}
            Ok(Event::Eof) => break,
            Err(err) => panic!("XML parse error during canonicalization: {err}"),
        }
        buf.clear();
    }
    out
}

fn emit_start(out: &mut String, e: &BytesStart, self_closing: bool, rev_counter: &mut u64) {
    let qname = String::from_utf8_lossy(e.name().as_ref()).into_owned();
    let renumber_id = REVISION_ELEMENTS.contains(&local_name(&qname));

    // Collect, filter, and sort attributes.
    let mut attrs: Vec<(String, String)> = Vec::new();
    for a in e.attributes().with_checks(false) {
        let a = a.expect("attribute");
        let key = String::from_utf8_lossy(a.key.as_ref()).into_owned();
        if is_volatile_attr(&key) {
            continue;
        }
        let val = String::from_utf8_lossy(&a.value).into_owned();
        if renumber_id && key == "w:id" {
            *rev_counter += 1;
            attrs.push((key, rev_counter.to_string()));
        } else {
            attrs.push((key, val));
        }
    }
    attrs.sort_by(|a, b| a.0.cmp(&b.0));

    out.push('<');
    out.push_str(&qname);
    for (k, v) in &attrs {
        out.push(' ');
        out.push_str(k);
        out.push_str("=\"");
        out.push_str(v);
        out.push('"');
    }
    if self_closing {
        out.push_str("/>");
    } else {
        out.push('>');
    }
}

/// Assert two XML strings are structurally equal (canonicalized), or panic.
pub fn assert_xml_structurally_eq(actual: &str, expected: &str, label: &str) {
    let a_canon = canonicalize_xml(actual.as_bytes());
    let e_canon = canonicalize_xml(expected.as_bytes());
    if a_canon != e_canon {
        let pos = a_canon
            .char_indices()
            .zip(e_canon.char_indices())
            .find(|((_, ca), (_, ce))| ca != ce)
            .map(|((i, _), _)| i)
            .unwrap_or(a_canon.len().min(e_canon.len()));
        let lo = pos.saturating_sub(120);
        panic!(
            "XML `{label}` differs near char {pos}:\n  actual:   …{}…\n  expected: …{}…",
            &a_canon[lo..(pos + 120).min(a_canon.len())],
            &e_canon[lo..(pos + 120).min(e_canon.len())],
        );
    }
}

/// Assert two docx byte streams are structurally equal, or panic with the first
/// differing part.
pub fn assert_docx_structurally_eq(actual: &[u8], expected: &[u8]) {
    let a_parts = read_parts(actual);
    let e_parts = read_parts(expected);

    let a_names: Vec<&String> = a_parts.keys().collect();
    let e_names: Vec<&String> = e_parts.keys().collect();
    assert_eq!(
        a_names, e_names,
        "docx part lists differ.\n  actual:   {a_names:?}\n  expected: {e_names:?}"
    );

    for (name, e_bytes) in &e_parts {
        let a_bytes = &a_parts[name];
        if is_xml_part(name) {
            let a_canon = canonicalize_xml(a_bytes);
            let e_canon = canonicalize_xml(e_bytes);
            if a_canon != e_canon {
                let pos = a_canon
                    .char_indices()
                    .zip(e_canon.char_indices())
                    .find(|((_, ca), (_, ce))| ca != ce)
                    .map(|((i, _), _)| i)
                    .unwrap_or(a_canon.len().min(e_canon.len()));
                let lo = pos.saturating_sub(80);
                panic!(
                    "XML part `{name}` differs near byte {pos}:\n  actual:   …{}…\n  expected: …{}…",
                    &a_canon[lo..(pos + 80).min(a_canon.len())],
                    &e_canon[lo..(pos + 80).min(e_canon.len())],
                );
            }
        } else {
            assert!(
                a_bytes == e_bytes,
                "binary part `{name}` differs (actual {} bytes, expected {} bytes)",
                a_bytes.len(),
                e_bytes.len()
            );
        }
    }
}

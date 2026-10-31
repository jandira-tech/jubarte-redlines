// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M8 — ISO/IEC 29500 **Strict** input support (regression: fixture f-3).
//!
//! Microsoft Word reads a Strict `.docx` natively and, when you Compare it,
//! emits a redline whose entire package is **Transitional** — verified against
//! Word's own f-3 output (`Strict01` vs `sd-2517`), which carries ZERO
//! `purl.oclc.org/ooxml` URIs in any part. Our comparer must match that.
//!
//! Before the fix, `compare_documents` PANICKED at `document_comparer.rs:55`
//! ("original has no body"): a Strict `w:body` lives in
//! `http://purl.oclc.org/ooxml/wordprocessingml/main`, but `W::body()` only
//! matches the Transitional URI, so the body lookup returned `None`.
//!
//! The fix (`strict_translation`) normalizes Strict → Transitional at load — the
//! same step the OpenXML SDK performs before PowerTools' WmlComparer runs — so
//! the body is found, the diff runs, and the output is uniformly Transitional
//! and Word-valid.
//!
//! Fixtures are vendored under `tests/fixtures/strict/` (the f-3 input pair).

use std::io::{Cursor, Read, Write};

use jubarte::changes::list_changes;
use jubarte::document_comparer::compare_documents;
use jubarte::opc::PartFs;

const STRICT_W: &str = "http://purl.oclc.org/ooxml/wordprocessingml/main";
const TRANS_W: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";

/// A minimal package around one `word/document.xml`.
fn docx_of(document: &str) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
        let opt = zip::write::SimpleFileOptions::default();
        z.start_file("[Content_Types].xml", opt).unwrap();
        z.write_all(br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#.as_slice()).unwrap();
        z.start_file("_rels/.rels", opt).unwrap();
        z.write_all(br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rIdM" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#.as_slice()).unwrap();
        z.start_file("word/document.xml", opt).unwrap();
        z.write_all(document.as_bytes()).unwrap();
        z.start_file("word/_rels/document.xml.rels", opt).unwrap();
        z.write_all(br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"/>"#.as_slice()).unwrap();
        z.finish().unwrap();
    }
    buf
}

/// The same geometry in both dialects' spellings — the Strict twin writes
/// universal measures (`2in`), the Transitional twin the integer twips
/// (`2880`) they name — on the frame, the floating table, the column grid,
/// the indents and both `w:spacing`s (paragraph and character). Nothing but
/// the spelling differs.
fn geometry_twin(strict: bool) -> Vec<u8> {
    let (w, frame, tblp, grid, col, cols_space, ind, pspacing, rspacing) = if strict {
        (
            STRICT_W,
            ("2in", "1in", "36pt", "36pt"),
            ("9pt", "1in", "72pt"),
            "155.8pt",
            "240pt",
            "18pt",
            ("0.25in", "18pt"),
            ("6pt", "6pt", "12pt"),
            "0.75pt",
        )
    } else {
        (
            TRANS_W,
            ("2880", "1440", "720", "720"),
            ("180", "1440", "1440"),
            "3116",
            "4800",
            "360",
            ("360", "360"),
            ("120", "120", "240"),
            "15",
        )
    };
    let doc = format!(
        "<w:document xmlns:w=\"{w}\"><w:body>\
           <w:p><w:pPr>\
             <w:framePr w:w=\"{f0}\" w:h=\"{f1}\" w:x=\"{f2}\" w:y=\"{f3}\"/>\
             <w:spacing w:before=\"{s0}\" w:after=\"{s1}\" w:line=\"{s2}\" w:lineRule=\"auto\"/>\
             <w:ind w:left=\"{i0}\" w:firstLine=\"{i1}\"/>\
             <w:rPr><w:spacing w:val=\"{r0}\"/></w:rPr>\
           </w:pPr><w:r><w:t>x</w:t></w:r></w:p>\
           <w:tbl><w:tblPr>\
             <w:tblpPr w:leftFromText=\"{t0}\" w:vertAnchor=\"page\" w:tblpX=\"{t1}\" w:tblpY=\"{t2}\"/>\
           </w:tblPr><w:tblGrid><w:gridCol w:w=\"{g0}\"/></w:tblGrid>\
           <w:tr><w:tc><w:tcPr><w:tcW w:w=\"{g0}\" w:type=\"dxa\"/></w:tcPr>\
           <w:p><w:r><w:t>cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl>\
           <w:p/>\
           <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
             <w:cols w:space=\"{c0}\" w:num=\"2\">\
               <w:col w:w=\"{k0}\" w:space=\"{c0}\"/>\
               <w:col w:w=\"{k0}\" w:space=\"{c0}\"/>\
             </w:cols></w:sectPr>\
         </w:body></w:document>",
        f0 = frame.0,
        f1 = frame.1,
        f2 = frame.2,
        f3 = frame.3,
        t0 = tblp.0,
        t1 = tblp.1,
        t2 = tblp.2,
        g0 = grid,
        k0 = col,
        c0 = cols_space,
        i0 = ind.0,
        i1 = ind.1,
        s0 = pspacing.0,
        s1 = pspacing.1,
        s2 = pspacing.2,
        r0 = rspacing,
    );
    docx_of(&doc)
}

/// The Strict ISO original of the f-3 pair.
const STRICT_ORIGINAL: &[u8] = include_bytes!("fixtures/strict/Strict01.docx");
/// A Transitional modified document (the f-3 modified).
const MODIFIED: &[u8] = include_bytes!("fixtures/strict/sd-2517-localized-heading-styles.docx");

/// Both identical-input exits must apply the same validity repairs as a diff
/// (#1796). Strict permits boolean-only cnfStyle; Transitional requires val.
fn assert_identical_strict_validity(original: &[u8], modified: &[u8]) {
    let out = compare_documents(original, modified, "Redline").unwrap();
    let pkg = PartFs::open(&out).unwrap();
    let mut cnf_styles = 0;
    for part in pkg.parts() {
        let Some(xml) = pkg
            .part_string(&part)
            .filter(|xml| xml.contains("cnfStyle"))
        else {
            continue;
        };
        let mut dom = jubarte::xmllinq::Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let root = dom.root(doc).unwrap();
        for node in dom.descendants(root, Some(&jubarte::namespaces::W::name("cnfStyle"))) {
            cnf_styles += 1;
            assert!(
                dom.attribute(node, &jubarte::namespaces::W::val())
                    .is_some(),
                "{part}: Transitional w:cnfStyle must carry w:val"
            );
        }
    }
    assert!(cnf_styles > 0, "fixture must exercise w:cnfStyle");
    let findings = jubarte::validate::ring1(&pkg);
    assert!(findings.is_empty(), "Ring-1 findings: {findings:#?}");
}

#[test]
fn identical_strict_self_compare_runs_validity_sweep() {
    assert_identical_strict_validity(STRICT_ORIGINAL, STRICT_ORIGINAL);
}

#[test]
fn identical_after_strict_translation_runs_validity_sweep() {
    let translated =
        jubarte::strict_translation::strict_to_transitional_docx(STRICT_ORIGINAL).unwrap();
    assert_ne!(STRICT_ORIGINAL, translated.as_slice());
    // Translation is idempotent: prep makes these distinct inputs byte-identical,
    // exercising the second exit rather than the full comparison path.
    assert_eq!(
        translated,
        jubarte::strict_translation::strict_to_transitional_docx(&translated).unwrap()
    );
    assert_identical_strict_validity(STRICT_ORIGINAL, &translated);
}

/// One part of a docx, by raw zip read.
fn read_part(docx: &[u8], name: &str) -> String {
    all_zip_entries(docx)
        .into_iter()
        .find(|(n, _)| n == name)
        .map(|(_, b)| String::from_utf8_lossy(&b).into_owned())
        .unwrap_or_default()
}

/// Every part (parts + `.rels` + `[Content_Types].xml`) of a docx, by raw zip read.
fn all_zip_entries(bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes.to_vec())).expect("valid zip/docx");
    let mut out = Vec::new();
    for i in 0..archive.len() {
        let mut f = archive.by_index(i).expect("zip entry");
        if f.is_dir() {
            continue;
        }
        let name = f.name().expect("decode ZIP fixture name").into_owned();
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).expect("read entry");
        out.push((name, buf));
    }
    out
}

#[test]
fn f3_strict_original_compares_without_panic_and_has_a_body() {
    // Was: panic "original has no body". Now: a real redline.
    let out = compare_documents(STRICT_ORIGINAL, MODIFIED, "Redline")
        .expect("compare must succeed when the ORIGINAL is an ISO Strict document");

    let pkg = PartFs::open(&out).expect("output is a valid OPC package");
    let doc = pkg
        .part_string("word/document.xml")
        .expect("output has a main document part");
    assert!(
        doc.contains("<w:body"),
        "redline output must contain a <w:body>"
    );
}

#[test]
fn f3_strict_output_is_fully_transitional() {
    let out = compare_documents(STRICT_ORIGINAL, MODIFIED, "Redline")
        .expect("compare must succeed when the ORIGINAL is an ISO Strict document");

    // Word's f-3 output has no strict URIs anywhere. Match it across every XML /
    // .rels / [Content_Types].xml entry — including the relationship parts that
    // PartFs hides behind its parsed model.
    for (name, bytes) in all_zip_entries(&out) {
        if name.ends_with(".xml") || name.ends_with(".rels") {
            let s = String::from_utf8_lossy(&bytes);
            assert!(
                !s.contains("purl.oclc.org/ooxml"),
                "ISO Strict URI leaked into output part `{name}` — output is not Transitional"
            );
        }
    }
}

/// f-4 (#531 review defect 1): a Strict document and its identical
/// Transitional twin — the same geometry, the two dialects' spellings —
/// compare with ZERO revisions, in either direction. The diff's property
/// signatures compare raw attribute strings, so the measures must be
/// canonicalized on both inputs at load, where Strict is translated; the
/// output sweep alone left the diff recording `w:pPrChange`,
/// `w:tblPrChange` and `w:tblGridChange` whose old and new sides
/// serialized identically (a formatting change that changes nothing).
#[test]
fn f4_strict_and_transitional_geometry_twins_compare_with_zero_revisions() {
    let strict = geometry_twin(true);
    let trans = geometry_twin(false);
    for (original, modified, dir) in [
        (strict.as_slice(), trans.as_slice(), "strict original"),
        (trans.as_slice(), strict.as_slice(), "transitional original"),
    ] {
        let out = compare_documents(original, modified, "Redline")
            .expect("the geometry twins must compare");
        let listed = list_changes(&out).expect("the redline lists its changes");
        assert!(
            listed.is_empty(),
            "{dir}: spelling-only geometry pair must record no revisions, got {listed:?}"
        );
        let doc = read_part(&out, "word/document.xml");
        for mark in ["pPrChange", "tblPrChange", "tblGridChange", "rPrChange"] {
            assert!(
                !doc.contains(mark),
                "{dir}: spelling-only geometry pair must not record {mark}: {doc}"
            );
        }
    }
}

/// #561 review defect 1: translating Strict01 must clear every
/// `purl.oclc.org` URI the package carries in a namespace site — the
/// `xmlns:*` declarations and the `a:graphicData/@uri` attributes (six of
/// them in this fixture) — while the bytes that are the file's own words
/// keep their Strict spelling: element text, a relationship `Target`, and
/// comments.
#[test]
fn strict01_translation_clears_every_namespace_site() {
    let bytes = std::fs::read("tests/fixtures/strict/Strict01.docx").expect("strict fixture");
    let out = jubarte::strict_translation::strict_to_transitional_docx(&bytes).unwrap();
    for (name, data) in all_zip_entries(&out) {
        if !(name.ends_with(".xml") || name.ends_with(".rels")) {
            continue;
        }
        let part = String::from_utf8_lossy(&data);
        let leaks = strict_uri_leaks(&part);
        assert!(
            leaks.is_empty(),
            "{name}: Strict URIs left in {}",
            leaks.join(", ")
        );
    }
}

/// The tags of `part` that carry a Strict URI in an attribute other than
/// `Target`: the namespace sites the translation owes a rewrite. Element
/// text (between tags), hyperlink targets and comments are the file's own
/// words and never count.
fn strict_uri_leaks(part: &str) -> Vec<&str> {
    let mut leaks = Vec::new();
    let mut rest = part;
    while let Some(lt) = rest.find('<') {
        let after = &rest[lt..];
        // A comment is the file's own words, `<!--` to `-->`.
        if let Some(body) = after.strip_prefix("<!--")
            && let Some(end) = body.find("-->")
        {
            rest = &body[end + 3..];
            continue;
        }
        // A tag runs to the first `>` outside its quoted values.
        let mut quote: Option<char> = None;
        let Some(end) = after.char_indices().skip(1).find_map(|(i, c)| match quote {
            Some(q) if c == q => {
                quote = None;
                None
            }
            Some(_) => None,
            None => match c {
                '"' | '\'' => {
                    quote = Some(c);
                    None
                }
                '>' => Some(i),
                _ => None,
            },
        }) else {
            break;
        };
        let (tag, next) = after.split_at(end + 1);
        let mut attrs = tag;
        while let Some(open) = attrs.find(['"', '\'']) {
            let (before, from_open) = attrs.split_at(open);
            let quote = from_open.chars().next().expect("find hit a quote");
            let Some(rel) = from_open[1..].find(quote) else {
                break;
            };
            let (value, tail) = from_open[1..].split_at(rel);
            let name = before
                .trim_end()
                .trim_end_matches('=')
                .trim_end()
                .rsplit(char::is_whitespace)
                .next()
                .unwrap_or("");
            if name != "Target" && value.contains("purl.oclc.org/ooxml") {
                leaks.push(tag.trim());
            }
            attrs = &tail[1..];
        }
        rest = next;
    }
    leaks
}

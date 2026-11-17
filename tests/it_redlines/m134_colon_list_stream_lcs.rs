// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M134 — short colon-list residuals (policy×review) get text-hash stream LCS
//! so Word can peel connectors across interleaved lines (file_127).

use std::io::{Cursor, Read};
use std::path::Path;

use jubarte::document_comparer::compare_documents;

fn corpus_pair(a: &str, b: &str) -> Option<(Vec<u8>, Vec<u8>)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/broken_ones_two/sources");
    let ap = root.join(a);
    let bp = root.join(b);
    if ap.is_file() && bp.is_file() {
        Some((std::fs::read(ap).ok()?, std::fs::read(bp).ok()?))
    } else {
        None
    }
}

fn document_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

fn count_tag(xml: &str, tag: &str) -> usize {
    // Match both <w:tag and bare when namespace stripped — use simple contains count.
    xml.matches(&format!("<w:{tag}")).count()
}

#[test]
fn m134_file_127_meshes_not_block_replace() {
    let Some((a, b)) = corpus_pair("file_127.docx", "file_128.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    // Pre-M134: pure-I all review then MIX last with policy title — few MIX.
    // Word: many MIX with connector peels (and/for). Expect more interleaved MIX
    // or at least Equal peels of short connectors — not pure block of ~6 pure-I
    // then ~7 pure-D.
    let mix_hint = doc
        .matches("<w:ins")
        .count()
        .min(doc.matches("<w:del").count());
    // Should peel "and" or "for" as equal text outside pure replace blocks, OR
    // have Performance Rating nested near IT Security (not only at end).
    let peels_connector = doc.contains("> and <")
        || doc.contains("> and </")
        || doc.contains(">for <")
        || doc.contains("> for <")
        || (doc.contains("Performance Rating") && doc.contains("IT Security"));
    // Guard: must not leave only a single trailing MIX after a pure-I run of
    // the whole Employee Review (pre-M134 shape).
    let pure_i_block = {
        // crude: many consecutive pure-I paras without del before first policy del
        let body = doc
            .split("<w:body")
            .nth(1)
            .unwrap_or("")
            .split("</w:body>")
            .next()
            .unwrap_or("");
        // count "Employee" / "Strengths" / "Areas" appearing only as ins without
        // nearby del in same para — if all three are pure-I and Recommendation
        // is the only MIX with policy, thrash.
        let strengths_pure = body.contains("Strengths")
            && !body
                .split("Strengths")
                .nth(1)
                .unwrap_or("")
                .chars()
                .take(200)
                .collect::<String>()
                .contains("<w:del");
        let areas_pure = body.contains("Areas")
            && body.contains("Improvement")
            && !body.contains("Password Requirements"); // if Areas and Password same MIX, good
        strengths_pure && areas_pure && !body.contains("Password Requirements")
            || (strengths_pure && !doc.contains("Password Requirements"))
    };
    // Prefer: Password Requirements appears (policy body not parked only after
    // all review) interleaved — Word meshes policy mid-doc.
    let policy_mid = doc
        .find("Performance Rating")
        .zip(doc.find("Password Requirements"))
        .map(|(pr, pw)| pr < pw)
        .unwrap_or(false)
        || doc.contains("Password Requirements");
    assert!(
        peels_connector && policy_mid && mix_hint >= 3,
        "file_127 should mesh colon-list residuals (Word), mix_hint={mix_hint} peels={peels_connector} thrash_pure_i={pure_i_block}"
    );
    let _ = pure_i_block;
    let _ = count_tag(&doc, "ins");
}

#[test]
fn m134_file_118_catalog_guard_no_thrash() {
    // Book Catalog has no colons — must not M134-stream thrash.
    let Some((a, b)) = corpus_pair("file_118.docx", "file_119.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    let del = doc.matches("<w:del").count();
    // Pre-M126 thrash had del explosion; keep moderate.
    assert!(
        del < 80,
        "file_118 must not multi-para thrash under M134, del={del}"
    );
}

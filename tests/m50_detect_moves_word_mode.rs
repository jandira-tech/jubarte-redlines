// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! broken_ones_two file_8×file_9 shape (class):
//! Word Compare marks large relocated blocks as `w:moveFrom`/`w:moveTo`
//! (~144 each on that stem). With `detect_moves` off we only emit del/ins,
//! so strike/underline colors diverge from Word's green move markup and the
//! pixel score collapses (~38). Word-visual default now enables moves.

use jubarte::comparer::{WmlComparerSettings, compare_bodies_faithful};
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

fn doc_body(dom: &mut Dom, inner: &str) -> (jubarte::xmllinq::NodeId, jubarte::xmllinq::NodeId) {
    let xml = format!(
        "<w:document xmlns:w=\"{w}\"><w:body>{inner}</w:body></w:document>",
        w = W::URI
    );
    let d = dom.parse_xdocument(&xml);
    let root = dom.root(d).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    (root, body)
}

fn para(text: &str) -> String {
    format!("<w:p><w:r><w:t>{text}</w:t></w:r></w:p>")
}

/// Relocated multi-word paragraph → moveFrom/moveTo, not bare del/ins.
#[test]
fn m50_relocated_block_emits_move_markup() {
    let mut dom = Dom::new();
    // Base: intro, then a long block that will move, then tail.
    let moved = "Parity real-time coauthoring comments mentions sharing links version history";
    let base = [
        para("Introduction stays put here"),
        para(moved),
        para("Closing section remains last in base"),
    ]
    .concat();
    // Next: same intro, tail first of the shared pair, moved block after → relocation.
    let next = [
        para("Introduction stays put here"),
        para("Closing section remains last in base"),
        para(moved),
        para("Brand new trailing insertion only in next"),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    assert!(s.detect_moves, "Word-visual default must enable moves");
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let xml = dom.serialize_element(out);

    assert!(
        xml.contains("moveFrom") || xml.contains("MovedSource"),
        "relocated block must surface as move source, got snippet: {}",
        &xml.chars().take(800).collect::<String>()
    );
    assert!(
        xml.contains("moveTo") || xml.contains("MovedDestination"),
        "relocated block must surface as move destination"
    );
    // The moved sentence should not appear only as a pure w:del + w:ins pair
    // without any move markup when detection is on.
    assert!(
        xml.contains(moved) || xml.contains("coauthoring"),
        "moved text must still be present in the redline body"
    );
}

/// Real corpus pair: file_8 × file_9 — Word emits ~144 moveFrom/moveTo.
/// With detect_moves on, our package compare must emit some move markup.
#[test]
fn m50_file_8_file_9_package_emits_moves() {
    use jubarte::document_comparer::compare_documents_with_settings;
    use std::path::PathBuf;

    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/broken_ones_two/sources");
    let a = root.join("file_8.docx");
    let b = root.join("file_9.docx");
    if !a.is_file() || !b.is_file() {
        eprintln!("skip: broken_ones_two sources missing");
        return;
    }
    let a_bytes = std::fs::read(&a).expect("file_8");
    let b_bytes = std::fs::read(&b).expect("file_9");

    let default_out =
        compare_documents_with_settings(&a_bytes, &b_bytes, &WmlComparerSettings::default())
            .expect("compare default");
    let default_xml = doc_xml(&default_out);
    let mf = count_tag(&default_xml, "moveFrom");
    let mt = count_tag(&default_xml, "moveTo");
    let ins = count_tag(&default_xml, "ins");
    let del = count_tag(&default_xml, "del");
    eprintln!("default moves: mf={mf} mt={mt} ins={ins} del={del}");

    // Relaxed thresholds: if default fails, diagnose whether min-count/threshold
    // are the gate (still must improve default until this is unnecessary).
    let relaxed = WmlComparerSettings {
        detect_moves: true,
        move_minimum_word_count: 1,
        move_similarity_threshold: 0.5,
        ..WmlComparerSettings::default()
    };
    let relaxed_out =
        compare_documents_with_settings(&a_bytes, &b_bytes, &relaxed).expect("compare relaxed");
    let relaxed_xml = doc_xml(&relaxed_out);
    let rmf = count_tag(&relaxed_xml, "moveFrom");
    let rmt = count_tag(&relaxed_xml, "moveTo");
    eprintln!("relaxed moves: mf={rmf} mt={rmt}");

    assert!(
        mf > 0 && mt > 0,
        "file_8×file_9 must emit moveFrom/moveTo under Word-visual defaults \
         (Word oracle has ~144); got mf={mf} mt={mt} ins={ins} del={del}; \
         relaxed mf={rmf} mt={rmt}"
    );
}

fn doc_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(docx)).expect("zip");
    let mut f = zip.by_name("word/document.xml").expect("document.xml");
    let mut s = String::new();
    std::io::Read::read_to_string(&mut f, &mut s).unwrap();
    s
}

fn count_tag(xml: &str, local: &str) -> usize {
    // Count exact element opens: <w:local or <w:local> / <w:local ...>
    // but NOT longer names (moveFrom must not count moveFromRangeStart).
    let mut n = 0;
    let needle = format!("<w:{local}");
    for (i, _) in xml.match_indices(&needle) {
        let after = &xml[i + needle.len()..];
        let next = after.chars().next();
        if matches!(next, Some(' ') | Some('>') | Some('/') | None) {
            n += 1;
        }
    }
    n
}

/// Accept-then compare of file_8×file_9: revision w:ids must not collide with
/// comment anchors (Word "unreadable content" when move range id == comment id).
#[test]
fn m50_accept_then_revision_ids_unique_vs_comments() {
    use jubarte::document_comparer::{accept_revisions, compare_documents_with_settings};
    use std::collections::HashMap;
    use std::path::PathBuf;

    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/broken_ones_two/sources");
    let a = root.join("file_8.docx");
    let b = root.join("file_9.docx");
    if !a.is_file() || !b.is_file() {
        return;
    }
    let a_bytes = accept_revisions(&std::fs::read(&a).unwrap()).unwrap();
    let b_bytes = accept_revisions(&std::fs::read(&b).unwrap()).unwrap();
    let out = compare_documents_with_settings(&a_bytes, &b_bytes, &WmlComparerSettings::default())
        .expect("compare");
    let xml = doc_xml(&out);
    let mut by_id: HashMap<String, Vec<String>> = HashMap::new();
    for (id, el) in id_element_pairs(&xml) {
        by_id.entry(id).or_default().push(el);
    }
    let revish = [
        "ins",
        "del",
        "moveFrom",
        "moveTo",
        "moveFromRangeStart",
        "moveFromRangeEnd",
        "moveToRangeStart",
        "moveToRangeEnd",
        "rPrChange",
        "tblPrChange",
        "tblGridChange",
        "pPrChange",
        "trPrChange",
        "tcPrChange",
        "sectPrChange",
    ];
    let comments = ["commentRangeStart", "commentRangeEnd", "commentReference"];
    for (id, els) in &by_id {
        let has_rev = els.iter().any(|e| {
            let local = e.rsplit(':').next().unwrap_or(e);
            revish.contains(&local)
        });
        let has_cmt = els.iter().any(|e| {
            let local = e.rsplit(':').next().unwrap_or(e);
            comments.contains(&local)
        });
        assert!(
            !(has_rev && has_cmt),
            "w:id={id} shared by revision {els:?} and comment — Word unreadable content"
        );
    }
    assert!(
        count_tag(&xml, "moveFrom") > 0,
        "expected moves on accept-then file_8×file_9"
    );
    assert!(!regex_move_from_nests_del(&xml));
}

fn id_element_pairs(xml: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut i = 0;
    while let Some(start) = xml[i..].find("w:id=\"") {
        let abs = i + start;
        let open = xml[..abs].rfind('<').unwrap_or(0);
        let name_end = xml[open + 1..]
            .find(|c: char| c.is_whitespace() || c == '/' || c == '>')
            .map(|n| open + 1 + n)
            .unwrap_or(open + 1);
        let el = xml[open + 1..name_end].to_string();
        let id_start = abs + "w:id=\"".len();
        let id_end = xml[id_start..]
            .find('"')
            .map(|n| id_start + n)
            .unwrap_or(id_start);
        let id = xml[id_start..id_end].to_string();
        out.push((id, el));
        i = id_end + 1;
    }
    out
}

/// moveFrom content must not nest w:del (Word repair dialog).
#[test]
fn m50_move_from_does_not_nest_del() {
    use jubarte::document_comparer::compare_documents_with_settings;
    use std::path::PathBuf;

    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/broken_ones_two/sources");
    let a = root.join("file_8.docx");
    let b = root.join("file_9.docx");
    if !a.is_file() || !b.is_file() {
        return;
    }
    // Accept both first so LCS sees relocatable paragraphs (matches Word Compare).
    let a_bytes =
        jubarte::document_comparer::accept_revisions(&std::fs::read(&a).unwrap()).unwrap();
    let b_bytes =
        jubarte::document_comparer::accept_revisions(&std::fs::read(&b).unwrap()).unwrap();
    let out = compare_documents_with_settings(&a_bytes, &b_bytes, &WmlComparerSettings::default())
        .expect("compare");
    let xml = doc_xml(&out);
    assert!(
        count_tag(&xml, "moveFrom") > 0,
        "expected move markup after accept-then compare"
    );
    assert!(
        !regex_move_from_nests_del(&xml),
        "w:moveFrom must not nest w:del (Word-valid); sample: {}",
        first_move_from_snippet(&xml)
    );
}

fn regex_move_from_nests_del(xml: &str) -> bool {
    // Nested <w:del ...> inside moveFrom is illegal. Do NOT match <w:delText.
    let mut i = 0;
    while let Some(start) = xml[i..].find("<w:moveFrom") {
        let abs = i + start;
        let after = &xml[abs + "<w:moveFrom".len()..];
        if !matches!(after.chars().next(), Some(' ') | Some('>') | Some('/')) {
            i = abs + 1;
            continue;
        }
        let Some(rel_end) = xml[abs..].find("</w:moveFrom>") else {
            break;
        };
        let body = &xml[abs..abs + rel_end];
        // exact w:del open tags only
        let mut j = 0;
        while let Some(ds) = body[j..].find("<w:del") {
            let dabs = j + ds;
            let dafter = &body[dabs + "<w:del".len()..];
            if matches!(dafter.chars().next(), Some(' ') | Some('>') | Some('/')) {
                return true;
            }
            j = dabs + 1;
        }
        i = abs + rel_end + 1;
    }
    false
}

fn first_move_from_snippet(xml: &str) -> String {
    xml.find("<w:moveFrom")
        .map(|i| xml[i..].chars().take(200).collect())
        .unwrap_or_default()
}

/// PowerTools-faithful keeps moves off (library default).
#[test]
fn m50_powertools_faithful_keeps_moves_off() {
    let mut dom = Dom::new();
    let moved = "the quick brown fox jumps over the lazy dog today";
    let base = [para("Alpha anchor paragraph here"), para(moved)].concat();
    let next = [para(moved), para("Alpha anchor paragraph here")].concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::powertools_faithful();
    assert!(!s.detect_moves);
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let xml = dom.serialize_element(out);
    assert!(
        !xml.contains("moveFrom") && !xml.contains("moveTo"),
        "faithful mode must not emit move markup: {}",
        &xml.chars().take(600).collect::<String>()
    );
}

#[test]
fn m118_file_175_related_charter_few_moves() {
    use jubarte::document_comparer::compare_documents_with_settings;
    use std::path::PathBuf;

    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/broken_ones_two/sources");
    let a = root.join("file_175.docx");
    let b = root.join("file_176.docx");
    if !a.is_file() || !b.is_file() {
        eprintln!("skip: corpus missing");
        return;
    }
    let a_bytes = std::fs::read(&a).unwrap();
    let b_bytes = std::fs::read(&b).unwrap();
    let out = compare_documents_with_settings(&a_bytes, &b_bytes, &WmlComparerSettings::default())
        .expect("compare");
    let xml = doc_xml(&out);
    let mf = count_tag(&xml, "moveFrom");
    // Word oracle has 2 moves; pre-M118 we emitted ~70.
    assert!(
        mf < 15,
        "related stamped charters must not thrash moves, got mf={mf}"
    );
}

#[test]
fn m122_file_175_spacing_pairs_cut_del_thrash() {
    use jubarte::document_comparer::compare_documents_with_settings;
    use std::path::PathBuf;
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/broken_ones_two/sources");
    let a = root.join("file_175.docx");
    let b = root.join("file_176.docx");
    if !a.is_file() || !b.is_file() {
        eprintln!("skip: corpus missing");
        return;
    }
    let out = compare_documents_with_settings(
        &std::fs::read(&a).unwrap(),
        &std::fs::read(&b).unwrap(),
        &WmlComparerSettings::default(),
    )
    .expect("compare");
    let xml = doc_xml(&out);
    let del = count_tag(&xml, "del");
    let ins = count_tag(&xml, "ins");
    // Pre-M122: del≈208 whole-para thrash. Word≈45. After correlated pairing, dels drop.
    assert!(
        del < 120,
        "spacing-stamped related charter must cut del thrash, got del={del} ins={ins}"
    );
}

#[test]
fn m123_file_93_equal_residual_zip_not_insert_all() {
    use jubarte::document_comparer::compare_documents_with_settings;
    use std::path::PathBuf;
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/broken_ones_two/sources");
    let a = root.join("file_93.docx");
    let b = root.join("file_94.docx");
    if !a.is_file() {
        eprintln!("skip");
        return;
    }
    let out = compare_documents_with_settings(
        &std::fs::read(&a).unwrap(),
        &std::fs::read(&b).unwrap(),
        &WmlComparerSettings::default(),
    )
    .unwrap();
    let xml = doc_xml(&out);
    // Word: 4 MIX paras, pPrChange≥1, rPrChange≈8. Pre-M123: pure-I/D thrash.
    let ppr = count_tag(&xml, "pPrChange");
    let rpr = count_tag(&xml, "rPrChange");
    assert!(
        ppr >= 1 || rpr >= 6,
        "equal-count residual zip should surface format changes, pPrChange={ppr} rPrChange={rpr}"
    );
    let del = count_tag(&xml, "del");
    assert!(del >= 3, "expected mixed dels from word LCS, got del={del}");
}

/// M123b: weak body diagonal (Yellow Highlight ↔ Subscript) must NOT full-zip.
/// Pre-gate blind M123 collapsed file_177 100→66; title-pair + pure I/D bodies
/// is the Word-shaped path.
#[test]
fn m123b_file_177_no_weak_body_diagonal_zip() {
    use jubarte::document_comparer::compare_documents_with_settings;
    use std::path::PathBuf;
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/broken_ones_two/sources");
    let a = root.join("file_177.docx");
    let b = root.join("file_178.docx");
    if !a.is_file() {
        eprintln!("skip");
        return;
    }
    let out = compare_documents_with_settings(
        &std::fs::read(&a).unwrap(),
        &std::fs::read(&b).unwrap(),
        &WmlComparerSettings::default(),
    )
    .unwrap();
    let xml = doc_xml(&out);
    // Full zip invents many tiny del/ins on unrelated chemistry bodies.
    // Title-pair + pure residual I/D keeps del count in a moderate band.
    let del = count_tag(&xml, "del");
    let ins = count_tag(&xml, "ins");
    assert!(
        del < 40 && ins < 40,
        "weak cousins must not full-zip thrash, del={del} ins={ins}"
    );
}

/// Skip-ahead pure-delete gap: B jumps from shared intro to a table that A has
/// later. Word shows the table early as moveTo; we must not leave it after the
/// gap-only deletes (docx_lots_of_comments_addition_removal_redline × clean).
#[test]
fn m50_skip_ahead_equal_table_promotes_to_move() {
    let mut dom = Dom::new();
    let table = |label: &str| {
        format!(
            "<w:tbl><w:tr><w:tc><w:p><w:r><w:t>{label} capability matrix row data here extra words</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"
        )
    };
    // A: intro, A-only middle, then capability table
    let base = [
        para("1. Executive summary shared anchor text"),
        para("Parity real-time coauthoring only in original document version"),
        para("Evidence base and source notes only in original"),
        table("Cap"),
        para("4. Visual proof continues after table"),
    ]
    .concat();
    // B: intro, capability table immediately (skips middle)
    let next = [
        para("1. Executive summary shared anchor text"),
        table("Cap"),
        para("4. Visual proof continues after table"),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let xml = dom.serialize_element(out);

    assert!(
        xml.contains("moveFrom") || xml.contains("moveTo"),
        "skip-ahead table must promote to move markup, got: {}",
        &xml.chars().take(1200).collect::<String>()
    );
    // Capability table text should appear before pure-deleted middle content
    // in serialization order (moveTo early).
    let cap_pos = xml.find("Cap capability").expect("cap text");
    let parity_pos = xml
        .find("Parity real-time")
        .expect("deleted middle content must remain present (data loss if absent)");
    assert!(
        cap_pos < parity_pos,
        "capability (moveTo) must serialize before A-only deleted middle; cap@{cap_pos} parity@{parity_pos}"
    );
}

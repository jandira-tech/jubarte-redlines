// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Word-mode package compare pre-accepts inputs that already carry track
//! changes (accept-then), matching Word Compare of finals.
//! broken_ones_two file_8×file_9 / file_27×file_28.

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;
use std::path::PathBuf;

fn sources(a: &str, b: &str) -> Option<(Vec<u8>, Vec<u8>)> {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/broken_ones_two/sources");
    let pa = root.join(a);
    let pb = root.join(b);
    if !pa.is_file() || !pb.is_file() {
        return None;
    }
    Some((std::fs::read(pa).unwrap(), std::fs::read(pb).unwrap()))
}

fn count_exact(xml: &str, local: &str) -> usize {
    let needle = format!("<w:{local}");
    xml.match_indices(&needle)
        .filter(|(i, _)| {
            let after = &xml[i + needle.len()..];
            matches!(
                after.chars().next(),
                Some(' ') | Some('>') | Some('/') | None
            )
        })
        .count()
}

fn doc_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(docx)).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut s = String::new();
    std::io::Read::read_to_string(&mut f, &mut s).unwrap();
    s
}

/// Direct Word-mode package compare of file_8×file_9 (file_9 has heavy TC)
/// must emit move markup without an external accept CLI step.
#[test]
fn m51_package_pre_accepts_tc_and_emits_moves() {
    let Some((a, b)) = sources("file_8.docx", "file_9.docx") else {
        eprintln!("skip: sources missing");
        return;
    };
    let out =
        compare_documents_with_settings(&a, &b, &WmlComparerSettings::default()).expect("compare");
    let xml = doc_xml(&out);
    let mf = count_exact(&xml, "moveFrom");
    let mt = count_exact(&xml, "moveTo");
    assert!(
        mf >= 10 && mt >= 10,
        "Word-mode pre-accept should surface many moves (accept-then class); got mf={mf} mt={mt}"
    );
}

/// file_27 has pre-TC; after pre-accept, ins count should stay near Word (~12),
/// not the 100+ from stamp re-emit of history.
#[test]
fn m51_package_pre_accept_keeps_ins_near_word() {
    let Some((a, b)) = sources("file_27.docx", "file_28.docx") else {
        return;
    };
    let out =
        compare_documents_with_settings(&a, &b, &WmlComparerSettings::default()).expect("compare");
    let xml = doc_xml(&out);
    let ins = count_exact(&xml, "ins");
    assert!(
        ins < 40,
        "pre-accept should not re-emit file_27 history as hundreds of ins; got {ins}"
    );
}

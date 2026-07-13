//! M124 — very short pure-I at I…I D…D multi-del boundary stays pure
//! (file_29: pure-I "a" then pure-D "Open Sans Font Demo"). M90 still folds
//! content pure-I into multi-del.

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;
use std::io::{Cursor, Read};
use std::path::PathBuf;

fn doc_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx)).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

fn para_kinds(xml: &str) -> Vec<(bool, bool, String)> {
    let mut out = Vec::new();
    for p in xml.split("<w:p").skip(1) {
        let end = p.find("</w:p>").unwrap_or(0);
        let chunk = &p[..end];
        let has_ins = chunk.contains("<w:ins");
        let has_del = chunk.contains("<w:del");
        let mut text = String::new();
        for part in chunk.split("<w:t").skip(1) {
            if let Some(gt) = part.find('>')
                && let Some(c) = part[gt + 1..].find("</w:t>")
            {
                text.push_str(&part[gt + 1..gt + 1 + c]);
            }
        }
        for part in chunk.split("<w:delText").skip(1) {
            if let Some(gt) = part.find('>')
                && let Some(c) = part[gt + 1..].find("</w:delText>")
            {
                text.push_str(&part[gt + 1..gt + 1 + c]);
            }
        }
        if text.trim().is_empty() && !has_ins && !has_del {
            continue;
        }
        out.push((has_ins, has_del, text));
    }
    out
}

#[test]
fn m124_file_29_short_a_not_folded_into_title() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/broken_ones_two/sources");
    let a = root.join("file_29.docx");
    let b = root.join("file_30.docx");
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
    let kinds = para_kinds(&xml);
    for (ins, del, t) in &kinds {
        let k = if *ins && *del {
            "MIX"
        } else if *ins {
            "I"
        } else if *del {
            "D"
        } else {
            "EQ"
        };
        eprintln!("  {k} {:?}", t.chars().take(50).collect::<String>());
    }
    // No MIX of single-letter insert with Open Sans title
    let bad_mix = kinds.iter().any(|(ins, del, t)| {
        *ins && *del && t.contains("Open Sans") && (t.starts_with('a') || t.contains("aOpen"))
    });
    assert!(!bad_mix, "short pure-I 'a' must not fold into pure-D title");
    // Pure-I "a" as own para
    let pure_a = kinds
        .iter()
        .any(|(ins, del, t)| *ins && !*del && t.trim() == "a");
    assert!(pure_a, "expected pure-I paragraph 'a'");
    // Pure-D title
    let pure_title = kinds
        .iter()
        .any(|(ins, del, t)| !*ins && *del && t.contains("Open Sans Font Demo"));
    assert!(pure_title, "expected pure-D title");
}

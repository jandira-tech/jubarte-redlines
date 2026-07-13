//! M137 — Demo next with non-This subtitle then This-body (file_151).
//! Word pure-I title+subtitle, pure-D base title, then body word LCS.

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

fn top_para_kinds(doc: &str) -> Vec<(bool, bool, String)> {
    let i0 = doc.find("<w:body").unwrap_or(0);
    let i1 = doc[i0..].find('>').map(|k| i0 + k + 1).unwrap_or(0);
    let i2 = doc.rfind("</w:body>").unwrap_or(doc.len());
    let s = &doc[i1..i2];
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < s.len() {
        if s[i..].starts_with("<w:sectPr") {
            break;
        }
        if s[i..].starts_with("<w:p ") || s[i..].starts_with("<w:p>") {
            let start = i;
            let mut d = 0i32;
            let mut j = i;
            while j < s.len() {
                if s[j..].starts_with("<w:p ") || s[j..].starts_with("<w:p>") {
                    d += 1;
                    j = s[j..].find('>').map(|k| j + k + 1).unwrap_or(s.len());
                } else if s[j..].starts_with("</w:p>") {
                    d -= 1;
                    j += 6;
                    if d == 0 {
                        let chunk = &s[start..j];
                        let has_ins = chunk.contains("<w:ins");
                        let has_del = chunk.contains("<w:del");
                        let mut text = String::new();
                        let mut p = 0;
                        while p < chunk.len() {
                            if let Some(rel) = chunk[p..].find("<w:t") {
                                let abs = p + rel;
                                let Some(gt) = chunk[abs..].find('>') else {
                                    break;
                                };
                                let st = abs + gt + 1;
                                if let Some(end) = chunk[st..].find("</w:t>") {
                                    text.push_str(&chunk[st..st + end]);
                                    p = st + end + 6;
                                    continue;
                                }
                            }
                            if let Some(rel) = chunk[p..].find("<w:delText") {
                                let abs = p + rel;
                                let Some(gt) = chunk[abs..].find('>') else {
                                    break;
                                };
                                let st = abs + gt + 1;
                                if let Some(end) = chunk[st..].find("</w:delText>") {
                                    text.push_str(&chunk[st..st + end]);
                                    p = st + end + 12;
                                    continue;
                                }
                            }
                            p += 1;
                        }
                        out.push((has_ins, has_del, text));
                        i = j;
                        break;
                    }
                } else {
                    j += 1;
                }
            }
            if j >= s.len() {
                break;
            }
            continue;
        }
        i += 1;
    }
    out
}

#[test]
fn m137_file_151_title_subtitle_pure_i_base_title_pure_d() {
    let Some((a, b)) = corpus_pair("file_151.docx", "file_152.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    let paras = top_para_kinds(&doc);
    // Word p1–p3: pure-I Demo title, pure-I Demonstrating subtitle, pure-D Project Proposal.
    let pure_i_demo = paras
        .iter()
        .any(|(i, d, t)| *i && !*d && t.contains("Bold and Italic Combo Demo"));
    let pure_i_demo_sub = paras
        .iter()
        .any(|(i, d, t)| *i && !*d && t.contains("Demonstrating"));
    let pure_d_proposal = paras
        .iter()
        .any(|(i, d, t)| !*i && *d && t.contains("Project Proposal"));
    // Must not thrash: Project Proposal mixed into Demo title para.
    let thrash_title = paras
        .iter()
        .any(|(i, d, t)| *i && *d && t.contains("Project Proposal") && t.contains("Combo Demo"));
    assert!(
        pure_i_demo && pure_i_demo_sub && pure_d_proposal && !thrash_title,
        "Word title shape: pure-I Demo+Demonstrating, pure-D Proposal — got {:?}",
        paras
            .iter()
            .map(|(i, d, t)| format!(
                "{}{} {}",
                if *i { "I" } else { "" },
                if *d { "D" } else { "" },
                t.chars().take(50).collect::<String>()
            ))
            .collect::<Vec<_>>()
    );
    // Body still peels Equal "This " (This-cousins).
    assert!(
        doc.contains("This ") || doc.contains(">This"),
        "body should still surface This"
    );
}

#[test]
fn m137_file_110_guard_m129_still() {
    let Some((a, b)) = corpus_pair("file_110.docx", "file_111.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    assert!(
        doc.contains("Red Bold") || doc.contains("Project Proposal"),
        "file_110 must still compare"
    );
    let del = doc.matches("<w:del").count();
    assert!(del < 40, "file_110 must not thrash, del={del}");
}

use jubarte::comparer::finalize::merge_replaced_paragraphs;
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;
use std::path::PathBuf;

#[test]
fn m54b_merge_file22_fresh_moves_ins_before_del() {
    let path = PathBuf::from(
        "/var/folders/7w/dvl66_d13wx2v88nv5rcwltc0000gn/T/grok-goal-6fb024e5f95c/implementer/file_22_fresh.docx",
    );
    let bytes = std::fs::read(&path).unwrap();
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut xml = String::new();
    std::io::Read::read_to_string(&mut f, &mut xml).unwrap();
    let mut d = Dom::new();
    let doc = d.parse_xdocument(&xml);
    let root = d.root(doc).unwrap();
    let body = d.element(root, &W::body()).unwrap();
    let children_before: Vec<_> = d.elements(body, None);
    eprintln!("body children {}", children_before.len());
    // classify first few and around 147
    let mut pure_i_idx = Vec::new();
    for (i, &c) in children_before.iter().enumerate() {
        if d.name(c) != Some(W::p()) {
            if i < 5 || (140..155).contains(&i) {
                eprintln!(
                    "child {i} {:?}",
                    d.name(c).map(|n| n.local_name().to_string())
                );
            }
            continue;
        }
        let (mut ins, mut del, mut plain) = (false, false, false);
        for ch in d.elements(c, None) {
            let n = d.name(ch).unwrap();
            if n == W::p_pr() {
                continue;
            } else if n == W::ins() {
                ins = true;
            } else if n == W::del() {
                del = true;
            } else {
                plain = true;
            }
        }
        let mark_del = d
            .element(c, &W::p_pr())
            .and_then(|ppr| d.element(ppr, &W::r_pr()))
            .and_then(|rpr| d.element(rpr, &W::del()))
            .is_some();
        let mark_ins = d
            .element(c, &W::p_pr())
            .and_then(|ppr| d.element(ppr, &W::r_pr()))
            .and_then(|rpr| d.element(rpr, &W::ins()))
            .is_some();
        if !ins && !del && !plain {
            if mark_del {
                del = true;
            }
            if mark_ins {
                ins = true;
            }
        }
        let cls = match (ins, del, plain) {
            (true, false, false) => "I",
            (false, true, false) => "D",
            (true, true, _) => "M",
            _ => "?",
        };
        if cls == "I" {
            pure_i_idx.push(i);
        }
        if i < 5 || (145..152).contains(&i) {
            eprintln!(
                "p {i} cls={cls} ins={ins} del={del} plain={plain} mark_i={mark_ins} mark_d={mark_del}"
            );
        }
    }
    eprintln!("pure I indices: {pure_i_idx:?}");
    merge_replaced_paragraphs(&mut d, root, "Arthur Souza Rodrigues");
    let body = d.element(root, &W::body()).unwrap();
    let paras: Vec<_> = d
        .elements(body, None)
        .into_iter()
        .filter(|&e| d.name(e) == Some(W::p()))
        .collect();
    let p1 = d.serialize_element(paras[1]);
    eprintln!("after p1: {}", &p1.chars().take(180).collect::<String>());
    assert!(
        p1.contains("Title Style"),
        "p1 should be title after reorder"
    );
}

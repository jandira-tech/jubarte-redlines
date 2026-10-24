// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A table cell's two final marks pair, as Word's Compare pairs them: a
//! paragraph added at the end of a cell, a cell's paragraph replaced by
//! two, or every paragraph of a cell deleted leaves the cell's final mark
//! unrevised, and the inserted or deleted mark is the one before it. A
//! revision on a cell's final mark hangs Word's Accept All or Reject All
//! (#1669; Word 16 oracles of 2026-10-23 on the issue).

mod common;

use common::docx::{docx, para, part_string};
use common::validity::check_word_valid_package;
use jubarte::document_comparer::{accept_revisions, compare_documents, reject_revisions};
use jubarte::inspect::paragraphs;

/// A table between two paragraphs: its first cell holds "Full.", its
/// second `cell` (a row whose every cell changes is #1672's).
fn table(cell: &str) -> Vec<u8> {
    let tc = |inner: &str| {
        format!("<w:tc><w:tcPr><w:tcW w:w=\"2000\" w:type=\"dxa\"/></w:tcPr>{inner}</w:tc>")
    };
    docx(&format!(
        "{}<w:tbl><w:tblPr><w:tblW w:w=\"4000\" w:type=\"dxa\"/></w:tblPr><w:tblGrid><w:gridCol w:w=\"2000\"/><w:gridCol w:w=\"2000\"/></w:tblGrid><w:tr>{}{}</w:tr></w:tbl>{}",
        para("Before."),
        tc(&para("Full.")),
        tc(cell),
        para("After.")
    ))
}

fn texts(bytes: &[u8]) -> Vec<String> {
    paragraphs(bytes)
        .unwrap()
        .into_iter()
        .map(|p| p.text)
        .collect()
}

/// The second cell's paragraphs, each as its mark's revision (`+`, `-` or none)
/// and its text, inserted words after `+`, deleted ones after `-`.
fn cell(bytes: &[u8]) -> Vec<(&'static str, String)> {
    let xml = part_string(bytes, "word/document.xml").unwrap();
    let start = xml.rfind("<w:tc>").unwrap();
    let end = start + xml[start..].find("</w:tc>").unwrap();
    let paragraph = regex::Regex::new(r"(?s)<w:p(?: [^>]*)?(?:/>|>.*?</w:p>)").unwrap();
    let mark = regex::Regex::new(r"(?s)<w:pPr>.*?<w:rPr>(.*?)</w:rPr>.*?</w:pPr>").unwrap();
    let token =
        regex::Regex::new(r"<w:(ins|del) |</w:(ins|del)>|<w:(?:t|delText)(?: [^>]*)?>([^<]*)</w:")
            .unwrap();
    paragraph
        .find_iter(&xml[start..end])
        .map(|p| {
            let p = p.as_str();
            let revision = match mark.captures(p).map(|c| c[1].to_string()) {
                Some(rpr) if rpr.contains("<w:ins ") => "+",
                Some(rpr) if rpr.contains("<w:del ") => "-",
                _ => "",
            };
            let body = p.find("</w:pPr>").map_or(p, |e| &p[e..]);
            let (mut side, mut text) = ("", String::new());
            for c in token.captures_iter(body) {
                if let Some(open) = c.get(1) {
                    side = if open.as_str() == "ins" { "+" } else { "-" };
                } else if c.get(2).is_some() {
                    side = "";
                } else {
                    text.push_str(&format!("{side}{}", &c[3]));
                }
            }
            (revision, text)
        })
        .collect()
}

/// Compare `a` with `b`: the redline is Word-valid (no revision on a cell's
/// final mark), accepts to `b` and rejects to `a`; its cell is `expected`.
fn check(a: &[u8], b: &[u8], expected: &[(&str, &str)], what: &str) -> Vec<u8> {
    let out = check_valid(a, b, what);
    let expected: Vec<(&str, String)> = expected.iter().map(|(m, t)| (*m, t.to_string())).collect();
    assert_eq!(cell(&out), expected, "{what}");
    out
}

#[test]
fn a_paragraph_appended_to_a_cell_inserts_the_mark_before_the_final_one() {
    check(
        &table(&para("Old.")),
        &table(&format!("{}{}", para("Old."), para("New."))),
        &[("+", "Old."), ("", "+New.")],
        "append",
    );
    check(
        &table(&para("Old.")),
        &table(&format!("{}{}{}", para("Old."), para("One."), para("Two."))),
        &[("+", "Old."), ("+", "+One."), ("", "+Two.")],
        "append two",
    );
}

#[test]
fn a_cells_paragraph_replaced_by_two_inserts_the_mark_before_the_final_one() {
    let out = check(
        &table(&para("Old.")),
        &table(&format!("{}{}", para("One."), para("Two."))),
        &[("+", "+One-Old."), ("", "+Two.")],
        "replace by two",
    );
    // The first paragraph's words are the two sides' first words, in
    // either order; the mark after them is the inserted one.
    assert!(cell(&out)[0].1.contains("+One") && cell(&out)[0].1.contains("-Old"));
}

#[test]
fn deleting_every_paragraph_of_a_cell_deletes_the_mark_before_the_final_one() {
    check(
        &table(&format!("{}{}", para("One."), para("Two."))),
        &table("<w:p/>"),
        &[("-", "-One."), ("", "-Two.")],
        "delete all",
    );
}

/// The final mark records the original's properties when the revised last
/// paragraph is formatted otherwise: Reject All gives the original its
/// properties back, Accept All keeps the revision's.
#[test]
fn an_appended_cell_paragraph_records_the_original_final_properties() {
    let centered = r#"<w:p><w:pPr><w:jc w:val="center"/><w:rPr><w:b/></w:rPr></w:pPr><w:r><w:t>Old.</w:t></w:r></w:p>"#;
    let a = table(centered);
    let b = table(&format!("{centered}{}", para("New.")));
    let out = check(&a, &b, &[("+", "Old."), ("", "+New.")], "formatted");
    let properties = |bytes: &[u8]| -> Vec<String> {
        let xml = part_string(bytes, "word/document.xml").unwrap();
        let start = xml.rfind("<w:tc>").unwrap();
        let end = start + xml[start..].find("</w:tc>").unwrap();
        let ppr = regex::Regex::new(r"(?s)<w:p(?: [^>]*)?(?:/>|>(.*?)</w:p>)").unwrap();
        ppr.captures_iter(&xml[start..end])
            .map(|c| {
                let p = c.get(1).map_or("", |m| m.as_str()).replace(" />", "/>");
                let jc = p.contains(r#"<w:jc w:val="center"/>"#);
                let bold = p.contains("<w:b/>");
                format!("jc={jc} b={bold}")
            })
            .collect()
    };
    assert_eq!(
        properties(&accept_revisions(&out).unwrap()),
        properties(&b),
        "Accept All"
    );
    assert_eq!(
        properties(&reject_revisions(&out).unwrap()),
        properties(&a),
        "Reject All"
    );
}

/// A cell's paragraphs replaced by others that share no word: whichever
/// order the comparer writes the two sides in, the closing marks pair and
/// the cell's final mark stays unrevised.
#[test]
fn a_cells_paragraphs_replaced_whole_keep_the_final_mark() {
    for (old, new) in [
        (para("Alpha beta gamma."), para("Delta epsilon zeta.")),
        (
            format!("{}{}", para("Alpha beta."), para("Gamma delta.")),
            para("Epsilon zeta eta theta."),
        ),
        (
            para("Alpha beta gamma delta."),
            format!("{}{}", para("Epsilon zeta."), para("Eta theta.")),
        ),
    ] {
        let a = table(&old);
        let b = table(&new);
        let out = check_valid(&a, &b, "replaced whole");
        assert_eq!(cell(&out).last().unwrap().0, "", "{:?}", cell(&out));
    }
}

/// A cell holding a nested table between two paragraphs, replaced by an
/// empty paragraph: the outer cell's closing marks pair over the deleted
/// table, as Word writes it.
#[test]
fn a_cells_tail_replaced_over_a_nested_table_keeps_the_final_mark() {
    let nested = format!(
        r#"{}<w:tbl><w:tblPr><w:tblW w:w="1800" w:type="dxa"/></w:tblPr><w:tblGrid><w:gridCol w:w="1800"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w="1800" w:type="dxa"/></w:tcPr>{}</w:tc></w:tr></w:tbl>{}"#,
        para("Lead."),
        para("Inner."),
        para("Tail.")
    );
    check_valid(&table(&nested), &table("<w:p/>"), "nested tail");
}

/// A row gaining a cell. Here the comparer rewrites the row (the old one
/// deleted, the new one inserted, every mark with its row); where it keeps
/// the row and adds the cell, the new cell's mark stays unrevised and only
/// its words are inserted (Word 16, ff42b4a7a3's "Role" cell:
/// `observed_m348_rewritten_cells_keep_revised_properties_and_complete_original_history`).
/// Either way no cell's final mark carries a revision its row does not.
#[test]
fn a_row_gaining_a_cell_resolves_both_ways() {
    let row = |cells: &[&str]| {
        let tc = |text: &str| {
            format!(
                "<w:tc><w:tcPr><w:tcW w:w=\"2000\" w:type=\"dxa\"/></w:tcPr>{}</w:tc>",
                para(text)
            )
        };
        let grid: String = cells.iter().map(|_| "<w:gridCol w:w=\"2000\"/>").collect();
        docx(&format!(
            "{}<w:tbl><w:tblPr><w:tblW w:w=\"6000\" w:type=\"dxa\"/></w:tblPr><w:tblGrid>{grid}</w:tblGrid><w:tr>{}</w:tr></w:tbl>{}",
            para("Before."),
            cells.iter().map(|c| tc(c)).collect::<String>(),
            para("After.")
        ))
    };
    let a = row(&["Name", "Department"]);
    let b = row(&["Name", "Department", "Role"]);
    check_valid(&a, &b, "added cell");
    check_valid(&b, &a, "removed cell");
}

/// A row added or removed whole keeps every final mark's revision, a
/// cell in a row-level content control (`w:tr/w:sdt/…/w:tc`) included:
/// the row's revision covers it (codex on #1680, round 9).
#[test]
fn a_wrapped_cell_of_a_whole_row_keeps_its_revised_marks() {
    let rows = |second: Option<(&str, &str)>| {
        let tc = |inner: String| {
            format!("<w:tc><w:tcPr><w:tcW w:w=\"2000\" w:type=\"dxa\"/></w:tcPr>{inner}</w:tc>")
        };
        let wrapped = second.map_or(String::new(), |(one, two)| {
            format!(
                "<w:tr><w:sdt><w:sdtPr/><w:sdtContent>{}</w:sdtContent></w:sdt></w:tr>",
                tc(para(one) + &para(two))
            )
        });
        docx(&format!(
            "{}<w:tbl><w:tblPr><w:tblW w:w=\"2000\" w:type=\"dxa\"/></w:tblPr><w:tblGrid><w:gridCol w:w=\"2000\"/></w:tblGrid><w:tr>{}</w:tr>{wrapped}</w:tbl>{}",
            para("Before."),
            tc(para("Keep.")),
            para("After.")
        ))
    };
    let short = rows(None);
    let long = rows(Some(("New.", "Two.")));
    check(
        &short,
        &long,
        &[("+", "+New."), ("+", "+Two.")],
        "row added",
    );
    check(
        &long,
        &short,
        &[("-", "-New."), ("-", "-Two.")],
        "row removed",
    );
}

/// The redline of `a` against `b` is Word-valid, accepts to `b` and rejects
/// to `a`.
fn check_valid(a: &[u8], b: &[u8], what: &str) -> Vec<u8> {
    let out = compare_documents(a, b, "Redline").expect("compare");
    let report = check_word_valid_package(&out);
    assert!(
        report.ok(),
        "{what}: {:?}\n{}",
        report.errors,
        part_string(&out, "word/document.xml").unwrap()
    );
    assert_eq!(
        texts(&accept_revisions(&out).unwrap()),
        texts(b),
        "{what}: Accept All"
    );
    assert_eq!(
        texts(&reject_revisions(&out).unwrap()),
        texts(a),
        "{what}: Reject All"
    );
    out
}

/// `body` with `cell` as the one cell of a table in `story`: a footnote,
/// an endnote, a header, a footer (either under a name of its own), or a
/// text box in the body. Returns the package and the part the table is in.
fn in_story(story: &str, cell: &str) -> (Vec<u8>, &'static str) {
    use common::docx::{Part, W_NS, docx_with, docx_with_sect};
    let table = format!(
        r#"<w:tbl><w:tblPr><w:tblW w:w="2000" w:type="dxa"/></w:tblPr><w:tblGrid><w:gridCol w:w="2000"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w="2000" w:type="dxa"/></w:tcPr>{cell}</w:tc></w:tr></w:tbl><w:p/>"#
    );
    let r_ns = common::docx::R_NS;
    let note = |kind: &str| {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:{kind}s xmlns:w="{W_NS}"><w:{kind} w:id="1">{table}</w:{kind}></w:{kind}s>"#
        )
    };
    let reference = |kind: &str| {
        format!(r#"<w:p><w:r><w:t>Body.</w:t></w:r><w:r><w:{kind}Reference w:id="1"/></w:r></w:p>"#)
    };
    let chrome = |root: &str, ignorable: bool| {
        // Word's own header root declares extension namespaces it may not
        // use, and lists them ignorable.
        let mc = if ignorable {
            r#" xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml" xmlns:w15="http://schemas.microsoft.com/office/word/2012/wordml" mc:Ignorable="w14 w15""#
        } else {
            ""
        };
        format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:{root} xmlns:w="{W_NS}" xmlns:r="{r_ns}"{mc}>{table}</w:{root}>"#
        )
    };
    const CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.";
    const REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/";
    match story {
        "footnote" | "endnote" => {
            let name: &'static str = if story == "footnote" {
                "word/footnotes.xml"
            } else {
                "word/endnotes.xml"
            };
            let xml = note(story);
            let content_type = format!("{CT}{story}s+xml");
            let rel_type = format!("{REL}{story}s");
            let part = Part {
                name,
                content_type: &content_type,
                rel_type: &rel_type,
                xml: &xml,
            };
            (docx_with(&reference(story), &[part]), name)
        }
        "header"
        | "footer"
        | "renamed header"
        | "renamed footer"
        | "renamed ignorable header"
        | "header.dat"
        | "footer.dat" => {
            // A story part is known by its relationship and content type,
            // not its name: `word/chrome.xml` can hold a header, and so can
            // `word/headerA.dat`, whose name starts as a header's does.
            let (name, root, kind): (&'static str, &str, &str) = match story {
                "header" => ("word/header1.xml", "hdr", "header"),
                "footer" => ("word/footer1.xml", "ftr", "footer"),
                "renamed header" | "renamed ignorable header" => {
                    ("word/chrome.xml", "hdr", "header")
                }
                "header.dat" => ("word/headerA.dat", "hdr", "header"),
                "footer.dat" => ("word/footerA.dat", "ftr", "footer"),
                _ => ("word/bottom.xml", "ftr", "footer"),
            };
            let xml = chrome(root, story == "renamed ignorable header");
            let content_type = format!("{CT}{kind}+xml");
            let rel_type = format!("{REL}{kind}");
            let part = Part {
                name,
                content_type: &content_type,
                rel_type: &rel_type,
                xml: &xml,
            };
            let sect = format!(r#"<w:{kind}Reference w:type="default" r:id="rIdX0"/>"#);
            (docx_with_sect(&para("Body."), &[part], &sect), name)
        }
        "text box" => (
            docx(&format!(
                r#"<w:p><w:r><w:pict><v:shape xmlns:v="urn:schemas-microsoft-com:vml" id="box" style="width:200pt;height:100pt"><v:textbox><w:txbxContent>{table}</w:txbxContent></v:textbox></v:shape></w:pict></w:r></w:p>"#
            )),
            "word/document.xml",
        ),
        _ => unreachable!("no story {story}"),
    }
}

/// `part`'s blocks as `[tbl [tr [tc [p text]]]]`.
fn blocks_of(bytes: &[u8], part: &str) -> String {
    let xml = part_string(bytes, part).unwrap();
    let token = regex::Regex::new(
        r"<w:(tbl|tr|tc|p)\b[^>]*?(/?)>|</w:(tbl|tr|tc|p)>|<w:t(?: [^>]*)?>([^<]*)</w:t>",
    )
    .unwrap();
    let mut out = String::new();
    for c in token.captures_iter(&xml) {
        if let Some(open) = c.get(1) {
            out.push('[');
            out.push_str(open.as_str());
            if !c[2].is_empty() {
                out.push(']');
            }
        } else if c.get(3).is_some() {
            out.push(']');
        } else {
            out.push_str(&c[4]);
        }
    }
    out
}

/// A cell in every story beside the body, in both modes: a paragraph
/// appended, and a nested table at the cell's start cleared with the text
/// after it. The redline is Word-valid, accepts to the revised story and
/// rejects to the original (#1684: the pairing never ran in the notes,
/// and missed headers, footers and text boxes; a header or footer under
/// another name was missed too, codex on #1680, round 10, and one whose
/// name starts `word/header` but does not end `.xml`, CodeRabbit, round 12).
#[test]
fn a_cells_final_mark_settles_in_every_story() {
    use jubarte::comparer::{CompareMode, WmlComparerSettings};
    use jubarte::document_comparer::compare_documents_with_settings;
    let nested = format!(
        r#"<w:tbl><w:tblPr><w:tblW w:w="1800" w:type="dxa"/></w:tblPr><w:tblGrid><w:gridCol w:w="1800"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w="1800" w:type="dxa"/></w:tcPr>{}</w:tc></w:tr></w:tbl>{}"#,
        para("Inner."),
        para("Tail.")
    );
    let pairs = [
        (para("Old."), format!("{}{}", para("Old."), para("New."))),
        (nested, "<w:p/>".to_string()),
    ];
    let mut failures = Vec::new();
    for story in [
        "footnote",
        "endnote",
        "header",
        "footer",
        "renamed header",
        "renamed ignorable header",
        "renamed footer",
        "header.dat",
        "footer.dat",
        "text box",
    ] {
        for (old, new) in &pairs {
            let (a, part) = in_story(story, old);
            let (b, _) = in_story(story, new);
            for mode in [CompareMode::Word, CompareMode::PowerTools] {
                let what = format!(
                    "{story}, {} ({mode:?})",
                    if new == "<w:p/>" {
                        "cleared"
                    } else {
                        "appended"
                    }
                );
                let out = compare_documents_with_settings(&a, &b, &WmlComparerSettings::new(mode))
                    .expect("compare");
                let report = check_word_valid_package(&out);
                if !report.ok() {
                    failures.push(format!("{what}: {:?}", report.errors));
                }
                let accepted = blocks_of(&accept_revisions(&out).unwrap(), part);
                if accepted != blocks_of(&b, part) {
                    failures.push(format!("{what}: Accept All gives {accepted}"));
                }
                let rejected = blocks_of(&reject_revisions(&out).unwrap(), part);
                if rejected != blocks_of(&a, part) {
                    failures.push(format!("{what}: Reject All gives {rejected}"));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// A nested table holding `Inner fixed.` and `Inner.` in two cells.
fn two_cell_nested() -> String {
    let tc = |text: &str| {
        format!(
            r#"<w:tc><w:tcPr><w:tcW w:w="900" w:type="dxa"/></w:tcPr>{}</w:tc>"#,
            para(text)
        )
    };
    format!(
        r#"<w:tbl><w:tblPr><w:tblW w:w="1800" w:type="dxa"/></w:tblPr><w:tblGrid><w:gridCol w:w="900"/><w:gridCol w:w="900"/></w:tblGrid><w:tr>{}{}</w:tr></w:tbl>"#,
        tc("Inner fixed."),
        tc("Inner.")
    )
}

/// A paragraph with properties `ppr` (the inside of `w:pPr`) and the runs
/// `runs` (each its formatting, the inside of `w:rPr`, and its text).
fn styled(ppr: &str, runs: &[(&str, &str)]) -> String {
    let runs: String = runs
        .iter()
        .map(|(rpr, text)| {
            let rpr = if rpr.is_empty() {
                String::new()
            } else {
                format!("<w:rPr>{rpr}</w:rPr>")
            };
            format!(r#"<w:r>{rpr}<w:t xml:space="preserve">{text}</w:t></w:r>"#)
        })
        .collect();
    let ppr = if ppr.is_empty() {
        String::new()
    } else {
        format!("<w:pPr>{ppr}</w:pPr>")
    };
    format!("<w:p>{ppr}{runs}</w:p>")
}

/// A final mark's revision moves only where resolving it joins nothing the
/// other version keeps apart: here the paragraphs after the mark before
/// keep words of their own in B, so the final mark loses its deletion, as
/// Word writes the first pair (Word 16 oracle o14, 2026-10-23). Codex and
/// GLM on #1680, round 10: the move joined `Epsilon zeta eta theta.` and
/// `One.`, two of B's paragraphs, and a join of the paragraphs around a
/// formatting change merged B's last two.
#[test]
fn a_final_mark_keeps_apart_the_paragraphs_b_keeps() {
    let center = r#"<w:jc w:val="center"/>"#;
    let both = styled("", &[("", "Ka."), ("<w:b/>", "Kb.")]);
    let pairs = [
        (
            "over a deleted nested table",
            format!(
                "{}{}{}",
                para("Alpha beta gamma delta."),
                two_cell_nested(),
                para("Two.")
            ),
            format!("{}{}", para("Epsilon zeta eta theta."), para("One.")),
        ),
        (
            "around a formatting change",
            format!("{}{}", styled(center, &[("", "Ka.")]), para("Ka.")),
            format!("{}{both}{both}", para("Gb.")),
        ),
        (
            "around a formatting change, one more",
            format!("{}{}", styled(center, &[("", "Ka.")]), para("Ka.")),
            format!("{}{both}{both}{}", para("Gb."), para("Zz.")),
        ),
    ];
    for (what, old, new) in pairs {
        check_valid(&table(&old), &table(&new), what);
    }
}

/// A cell losing an empty paragraph while it gains words: the comparer
/// drops one of the two empty paragraphs (#1686), and the deletion left on
/// the final mark does not move onto `Ka.`'s, which would join `Ka.` and
/// `Gb.`, two of B's paragraphs (GLM on #1680, round 10). Reject All still
/// misses one empty paragraph: that is #1686.
#[test]
fn a_dropped_empty_paragraph_does_not_join_the_paragraphs_b_keeps() {
    let pairs = [
        (
            format!("{}<w:p/><w:p/>", para("Ka.")),
            format!("{}{}", para("Ka."), para("Gb.")),
        ),
        (
            format!("{}{}<w:p/><w:p/>", para("Ka."), para("Kb.")),
            format!("{}{}{}", para("Ka."), para("Kb."), para("Gb.")),
        ),
    ];
    for (old, new) in pairs {
        let (a, b) = (table(&old), table(&new));
        let out = compare_documents(&a, &b, "Redline").expect("compare");
        let report = check_word_valid_package(&out);
        assert!(report.ok(), "{old}: {:?}", report.errors);
        assert_eq!(
            texts(&accept_revisions(&out).unwrap()),
            texts(&b),
            "{old}: Accept All\n{}",
            part_string(&out, "word/document.xml").unwrap()
        );
    }
}

/// The change records the settle pass writes take ids Word's schema
/// accepts (a signed 32-bit number), whatever the part already holds: a
/// comment id of 2147483647 sent them past it (codex on #1680, round 10;
/// the OOXML validator refused 2147483648).
#[test]
fn a_settled_cells_records_take_ids_the_schema_accepts() {
    use common::docx::{Part, W_NS, docx_with};
    let comments = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:comments xmlns:w="{W_NS}"><w:comment w:id="2147483647" w:author="Ann" w:date="2026-10-01T00:00:00Z" w:initials="A"><w:p><w:r><w:t>Comment.</w:t></w:r></w:p></w:comment></w:comments>"#
    );
    let document = |cells: [String; 2]| {
        let tc = |inner: &str| {
            format!("<w:tc><w:tcPr><w:tcW w:w=\"2000\" w:type=\"dxa\"/></w:tcPr>{inner}</w:tc>")
        };
        let body = format!(
            r#"<w:p><w:commentRangeStart w:id="2147483647"/><w:r><w:t>Anchor.</w:t></w:r><w:commentRangeEnd w:id="2147483647"/><w:r><w:commentReference w:id="2147483647"/></w:r></w:p><w:tbl><w:tblPr><w:tblW w:w="6000" w:type="dxa"/></w:tblPr><w:tblGrid><w:gridCol w:w="2000"/><w:gridCol w:w="2000"/><w:gridCol w:w="2000"/></w:tblGrid><w:tr>{}{}{}</w:tr></w:tbl>{}"#,
            tc(&para("Fixed.")),
            tc(&cells[0]),
            tc(&cells[1]),
            para("After.")
        );
        let part = Part {
            name: "word/comments.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
            xml: &comments,
        };
        docx_with(&body, &[part])
    };
    let old = |text: &str| {
        styled(
            r#"<w:jc w:val="center"/><w:rPr><w:b/></w:rPr>"#,
            &[("", text)],
        )
    };
    let new = |text: &str| styled(r#"<w:jc w:val="right"/>"#, &[("", text)]);
    let a = document([old("Old0."), old("Old1.")]);
    let b = document([
        format!("{}{}", old("Old0."), new("New0.")),
        format!("{}{}", old("Old1."), new("New1.")),
    ]);
    let out = check_valid(&a, &b, "a comment id at the top of the range");
    let xml = part_string(&out, "word/document.xml").unwrap();
    let ids = regex::Regex::new(r#"<w:(?:pPrChange|rPrChange|ins|del) [^>]*w:id="(\d+)""#).unwrap();
    let too_big: Vec<u64> = ids
        .captures_iter(&xml)
        .map(|c| c[1].parse::<u64>().unwrap())
        .filter(|&id| id > i32::MAX as u64)
        .collect();
    assert!(
        too_big.is_empty(),
        "ids past 2147483647: {too_big:?}\n{xml}"
    );
    assert!(xml.contains("<w:pPrChange "), "no record written: {xml}");
}

/// A cell's paragraphs merged, split, or ending in an empty paragraph
/// gained or lost resolve both ways in both modes: the paragraphs a final
/// mark's revision joins once moved keep no words of their own there, so
/// the move stands (the rule `a_final_mark_keeps_apart_the_paragraphs_b_keeps`
/// pins from the other side).
#[test]
fn a_cells_paragraphs_merged_or_split_resolve_both_ways() {
    use jubarte::comparer::{CompareMode, WmlComparerSettings};
    use jubarte::document_comparer::compare_documents_with_settings;
    let (x, y, z, w) = (
        para("Alpha beta."),
        para("Gamma delta."),
        para("Zeta eta."),
        para("Omega."),
    );
    let e = "<w:p/>";
    let pairs = [
        (format!("{x}{e}"), para("Alpha beta. Zeta eta.")),
        (format!("{x}{y}"), para("Alpha beta. Zeta eta.")),
        (format!("{x}{y}"), para("Alpha beta. Gamma delta.")),
        (para("Alpha beta. Gamma delta."), format!("{x}{y}")),
        (para("Alpha beta. Gamma delta."), format!("{x}{z}")),
        (para("Alpha beta. Zeta eta."), format!("{x}{e}")),
        (
            format!("{w}{x}{y}"),
            format!("{w}{}", para("Alpha beta. Gamma delta.")),
        ),
        (
            format!("{w}{}", para("Alpha beta. Gamma delta.")),
            format!("{w}{x}{y}"),
        ),
        (x.clone(), format!("{x}{e}")),
        (format!("{x}{e}"), x.clone()),
    ];
    for (old, new) in pairs {
        let (a, b) = (table(&old), table(&new));
        for mode in [CompareMode::Word, CompareMode::PowerTools] {
            let what = format!("{old} -> {new} ({mode:?})");
            let out = compare_documents_with_settings(&a, &b, &WmlComparerSettings::new(mode))
                .expect("compare");
            let report = check_word_valid_package(&out);
            assert!(report.ok(), "{what}: {:?}", report.errors);
            assert_eq!(
                texts(&accept_revisions(&out).unwrap()),
                texts(&b),
                "{what}: Accept All"
            );
            assert_eq!(
                texts(&reject_revisions(&out).unwrap()),
                texts(&a),
                "{what}: Reject All"
            );
        }
    }
}

/// A part that is no story the comparison wrote, though it holds a cell
/// whose final marks carry revisions, comes out as it went in: a custom
/// XML data item (the data behind a bound content control) or the
/// glossary's building blocks. Only stories, found by their relationship
/// types, settle their final marks (minimax on #1680, round 11).
#[test]
fn a_part_that_is_no_story_is_copied_as_it_was() {
    use common::docx::{Part, W_NS, docx_with};
    const REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/";
    let rev = r#"w:author="Ann" w:date="2026-10-01T00:00:00Z""#;
    let table = format!(
        r#"<w:tbl><w:tblPr><w:tblW w:w="2000" w:type="dxa"/></w:tblPr><w:tblGrid><w:gridCol w:w="2000"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w="2000" w:type="dxa"/></w:tcPr><w:p><w:pPr><w:rPr><w:del w:id="31" {rev}/></w:rPr></w:pPr><w:r><w:t>One.</w:t></w:r></w:p><w:p><w:pPr><w:rPr><w:ins w:id="32" {rev}/></w:rPr></w:pPr><w:ins w:id="33" {rev}><w:r><w:t>Two.</w:t></w:r></w:ins></w:p></w:tc></w:tr></w:tbl>"#
    );
    let data = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:customData xmlns:w="{W_NS}">{table}</w:customData>"#
    );
    let glossary = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:glossaryDocument xmlns:w="{W_NS}"><w:docParts><w:docPart><w:docPartPr><w:name w:val="Block"/></w:docPartPr><w:docPartBody>{table}<w:p/></w:docPartBody></w:docPart></w:docParts></w:glossaryDocument>"#
    );
    let customxml_rel = format!("{REL}customXml");
    let glossary_rel = format!("{REL}glossaryDocument");
    for (what, name, content_type, rel_type, xml) in [
        (
            "custom XML data",
            "word/customXml/item1.xml",
            "application/xml",
            customxml_rel.as_str(),
            data.as_str(),
        ),
        // Under a header's name it is custom data still, known by the
        // relationship that reaches it (codex on #1680).
        (
            "custom XML data under a header's name",
            "word/headerData.xml",
            "application/xml",
            customxml_rel.as_str(),
            data.as_str(),
        ),
        (
            "glossary",
            "word/glossary/document.xml",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document.glossary+xml",
            glossary_rel.as_str(),
            glossary.as_str(),
        ),
    ] {
        let part = Part {
            name,
            content_type,
            rel_type,
            xml,
        };
        let a = docx_with(&para("Body one."), std::slice::from_ref(&part));
        let b = docx_with(&para("Body two."), std::slice::from_ref(&part));
        let out = compare_documents(&a, &b, "Redline").expect("compare");
        assert_eq!(
            part_string(&out, name).unwrap(),
            part_string(&a, name).unwrap(),
            "{what}"
        );
    }
}

/// A renamed header whose revised side alone carries Word 2010 paragraph
/// ids keeps them declared and ignorable when its cells settle (CodeRabbit
/// on #1680: the story sweep ran no namespace finalizer).
#[test]
fn a_renamed_headers_extension_ids_stay_declared_when_its_cells_settle() {
    use jubarte::comparer::{CompareMode, WmlComparerSettings};
    use jubarte::document_comparer::compare_documents_with_settings;
    let (a, part) = in_story("renamed header", &para("Old."));
    let revised = format!("{}{}", para("Old."), para("New.")).replace(
        "<w:p>",
        r#"<w:p w14:paraId="1A2B3C4D" w14:textId="77777777">"#,
    );
    let (b, _) = in_story("renamed ignorable header", &revised);
    assert!(part_string(&b, part).unwrap().contains("w14:paraId"));
    for mode in [CompareMode::Word, CompareMode::PowerTools] {
        let out = compare_documents_with_settings(&a, &b, &WmlComparerSettings::new(mode))
            .expect("compare");
        let report = check_word_valid_package(&out);
        assert!(report.ok(), "{mode:?}: {:?}", report.errors);
        let xml = part_string(&out, part).unwrap();
        if xml.contains("w14:") {
            assert!(xml.contains("xmlns:w14="), "{mode:?}: {xml}");
            assert!(xml.contains("mc:Ignorable="), "{mode:?}: {xml}");
        }
    }
}

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A paragraph whose mark goes on Accept All or Reject All joins the next
//! one. When a block content control holds one of the two, Word keeps the
//! control a block unless both paragraphs bring something to the join
//! (#1694). Every case here is a redline Word 16 resolved in the local A/B
//! of 2026-10-24 (the file named beside it); the expected shape is Word's.
//!
//! - The paragraph that brings nothing (no words left, no control of its
//!   own) goes, and the other keeps its block control.
//! - When both bring something, they merge into one paragraph and each
//!   control becomes an inline control there, an emptied control included.
//! - A control holding more than one paragraph cannot become inline, and
//!   Word takes no paragraph out of it: where the join would cross its edge
//!   and the paragraph outside it brings something, the mark stays, and the
//!   paragraph that kept it takes the next paragraph's properties.

mod common;

use common::docx::{docx, para};
use common::validity::check_word_valid_package;
use jubarte::document_comparer::{accept_revisions, reject_revisions};

const REV: &str = r#"w:author="Ann" w:date="2026-10-01T00:00:00Z""#;

fn mark(kind: &str, id: u32) -> String {
    format!(r#"<w:pPr><w:rPr><w:{kind} w:id="{id}" {REV}/></w:rPr></w:pPr>"#)
}

fn run(text: &str) -> String {
    format!("<w:r><w:t>{text}</w:t></w:r>")
}

fn tracked(kind: &str, text: &str, id: u32) -> String {
    let tag = if kind == "del" { "delText" } else { "t" };
    format!(r#"<w:{kind} w:id="{id}" {REV}><w:r><w:{tag}>{text}</w:{tag}></w:r></w:{kind}>"#)
}

fn aligned(jc: &str, mark: &str) -> String {
    let rpr = mark
        .strip_prefix("<w:pPr>")
        .and_then(|m| m.strip_suffix("</w:pPr>"))
        .unwrap_or("");
    format!(r#"<w:pPr><w:jc w:val="{jc}"/>{rpr}</w:pPr>"#)
}

fn p(ppr: &str, content: &str) -> String {
    format!("<w:p>{ppr}{content}</w:p>")
}

fn sdt(blocks: &str) -> String {
    format!("<w:sdt><w:sdtPr/><w:sdtContent>{blocks}</w:sdtContent></w:sdt>")
}

fn cell(blocks: &str) -> String {
    format!(
        r#"<w:tbl><w:tblPr><w:tblW w:w="4000" w:type="dxa"/></w:tblPr><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w="4000" w:type="dxa"/></w:tcPr>{blocks}</w:tc></w:tr></w:tbl>"#
    )
}

/// The body between "Before." and "After." as nested blocks:
/// `[sdt[pOne.]]` is a block control around a paragraph, `[p[sdtOne.]]` an
/// inline control in one, `{jc=right}` a paragraph's alignment.
fn shape(bytes: &[u8]) -> String {
    let xml = common::docx::part_string(bytes, "word/document.xml").unwrap();
    let token = regex::Regex::new(
        r#"<w:(tbl|tr|tc|p|sdt)\b[^>]*?(/?)>|</w:(tbl|tr|tc|p|sdt)>|<w:(?:t|delText)(?: [^>]*)?>([^<]*)</w:|<w:jc w:val="(\w+)""#,
    )
    .unwrap();
    let body = &xml[xml.find("<w:body").unwrap()..];
    let mut out = String::new();
    for c in token.captures_iter(body) {
        if let Some(open) = c.get(1) {
            out.push('[');
            out.push_str(open.as_str());
            if !c[2].is_empty() {
                out.push(']');
            }
        } else if c.get(3).is_some() {
            out.push(']');
        } else if let Some(jc) = c.get(5) {
            out.push_str(&format!("{{jc={}}}", jc.as_str()));
        } else {
            out.push_str(&c[4]);
        }
    }
    out.replace("[pBefore.]", "").replace("[pAfter.]", "")
}

fn resolve(body: &str, mode: &str) -> Vec<u8> {
    let redline = docx(&format!("{}{body}{}", para("Before."), para("After.")));
    let out = match mode {
        "accept" => accept_revisions(&redline),
        _ => reject_revisions(&redline),
    }
    .unwrap();
    let report = check_word_valid_package(&out);
    assert!(report.ok(), "{mode}: {:?}", report.errors);
    let xml = common::docx::part_string(&out, "word/document.xml").unwrap();
    for left in ["<w:del ", "<w:ins ", "<w:moveFrom ", "<w:moveTo "] {
        assert!(!xml.contains(left), "{mode} left {left}: {xml}");
    }
    out
}

#[test]
fn the_paragraph_after_a_control_that_brings_nothing_goes() {
    let cases = [
        // hn8: the inserted mark goes on Reject, and the next paragraph's
        // words were all inserted.
        (
            "hn8",
            sdt(&p(&mark("ins", 1), &run("One."))) + &p("", &tracked("ins", "Two.", 2)),
            "reject",
            "[sdt[pOne.]]",
        ),
        // hn9: the same in a cell, deleted, on Accept.
        (
            "hn9",
            cell(&(sdt(&p(&mark("del", 1), &run("One."))) + &p("", &tracked("del", "Two.", 2)))),
            "accept",
            "[tbl[tr[tc[sdt[pOne.]]]]]",
        ),
        // j21: the cell's empty final paragraph, untracked.
        (
            "j21",
            cell(&(sdt(&p(&mark("del", 1), &run("Sdt."))) + &p("", ""))),
            "accept",
            "[tbl[tr[tc[sdt[pSdt.]]]]]",
        ),
        // wo17 (Word's own redline): the next paragraph's words inserted.
        (
            "wo17",
            cell(&(sdt(&p(&mark("ins", 4), &run("Old."))) + &p("", &tracked("ins", "New.", 6)))),
            "reject",
            "[tbl[tr[tc[sdt[pOld.]]]]]",
        ),
        // hn11: the kept paragraph carries the properties of the paragraph
        // that went, whose mark it now ends with.
        (
            "hn11",
            sdt(&p(&aligned("center", &mark("del", 1)), &run("One.")))
                + &p(&aligned("right", ""), &tracked("del", "Two.", 2)),
            "accept",
            "[sdt[p{jc=right}One.]]",
        ),
        // hn16: an empty, untracked paragraph with properties of its own.
        (
            "hn16",
            sdt(&p(&aligned("center", &mark("del", 1)), &run("One.")))
                + &p(&aligned("right", ""), ""),
            "accept",
            "[sdt[p{jc=right}One.]]",
        ),
        // hn19: a paragraph holding only a bookmark brings nothing.
        (
            "hn19",
            sdt(&p(&mark("del", 1), &run("One.")))
                + &p(
                    "",
                    r#"<w:bookmarkStart w:id="0" w:name="mark"/><w:bookmarkEnd w:id="0"/>"#,
                ),
            "accept",
            "[sdt[pOne.]]",
        ),
        // hn18: a control whose words and mark go stays a block, emptied,
        // when the paragraph it joins brings nothing either.
        (
            "hn18",
            sdt(&p(&mark("del", 1), &tracked("del", "One.", 2))) + &p("", ""),
            "accept",
            "[sdt[p]]",
        ),
        // hn22: proofing marks are nothing either.
        (
            "hn22",
            sdt(&p(&mark("del", 1), &run("One.")))
                + &p(
                    "",
                    r#"<w:proofErr w:type="spellStart"/><w:proofErr w:type="spellEnd"/>"#,
                ),
            "accept",
            "[sdt[pOne.]]",
        ),
        // hn31: a two-paragraph control whose paragraphs all go, before a
        // paragraph that brings nothing, stays a block, emptied.
        (
            "hn31",
            sdt(&(p(&mark("del", 1), &tracked("del", "One.", 2))
                + &p(&mark("del", 3), &tracked("del", "Two.", 4))))
                + &p("", ""),
            "accept",
            "[sdt[p]]",
        ),
        // hn23: the last paragraph of a two-paragraph control joins a
        // paragraph that brings nothing, and takes its properties.
        (
            "hn23",
            sdt(&(p("", &run("X.")) + &p(&aligned("center", &mark("del", 1)), &run("Y."))))
                + &p(&aligned("right", ""), &tracked("del", "Z.", 2)),
            "accept",
            "[sdt[pX.][p{jc=right}Y.]]",
        ),
    ];
    for (word, body, mode, expected) in cases {
        assert_eq!(shape(&resolve(&body, mode)), expected, "{word} {mode}");
    }
}

#[test]
fn the_paragraph_before_a_control_that_brings_nothing_goes() {
    let cases = [
        // j03: a deleted paragraph before the control's inserted mark.
        (
            "j03",
            cell(
                &(p(&mark("del", 1), &tracked("del", "One.", 2))
                    + &sdt(&p(&mark("ins", 3), &run("Sdt.")))
                    + &p("", "")),
            ),
            "accept",
            "[tbl[tr[tc[sdt[pSdt.]][p]]]]",
        ),
        // j04: an inserted empty paragraph before the control whose words
        // were inserted: the control stays a block, emptied.
        (
            "j04",
            cell(&(p(&mark("ins", 1), "") + &sdt(&p("", &tracked("ins", "Sdt.", 2))))),
            "reject",
            "[tbl[tr[tc[sdt[p]]]]]",
        ),
        // j03, the other way: the empty final paragraph after the control
        // goes, the paragraph before it stays.
        (
            "j03",
            cell(
                &(p(&mark("del", 1), &tracked("del", "One.", 2))
                    + &sdt(&p(&mark("ins", 3), &run("Sdt.")))
                    + &p("", "")),
            ),
            "reject",
            "[tbl[tr[tc[pOne.][sdt[pSdt.]]]]]",
        ),
        // hn17: in the body, with properties: the control's paragraph keeps
        // its own.
        (
            "hn17",
            p(
                &aligned("center", &mark("del", 1)),
                &tracked("del", "One.", 2),
            ) + &sdt(&p(&aligned("right", ""), &run("Two."))),
            "accept",
            "[sdt[p{jc=right}Two.]]",
        ),
        // hn24: before a two-paragraph control.
        (
            "hn24",
            p(&mark("del", 1), &tracked("del", "One.", 2))
                + &sdt(&(p("", &run("Two.")) + &p("", &run("Three.")))),
            "accept",
            "[sdt[pTwo.][pThree.]]",
        ),
    ];
    for (word, body, mode, expected) in cases {
        assert_eq!(shape(&resolve(&body, mode)), expected, "{word} {mode}");
    }
}

#[test]
fn an_emptied_control_stays_inline_where_both_paragraphs_bring_something() {
    let cases = [
        // j16: the control's words and mark were inserted; the paragraph
        // after it keeps "One.".
        (
            "j16",
            cell(
                &(sdt(&p(&mark("ins", 1), &tracked("ins", "Sdt.", 2)))
                    + &p(&mark("del", 3), &run("One."))
                    + &sdt(&p("", &tracked("del", "Sdt.", 4)))),
            ),
            "reject",
            "[tbl[tr[tc[p[sdt]One.][sdt[pSdt.]]]]]",
        ),
        // j09: two controls, both emptied, their paragraphs joined.
        (
            "j09",
            cell(
                &(p(&mark("ins", 1), &tracked("ins", "One.", 2))
                    + &sdt(&p(&mark("del", 3), &tracked("del", "Sdt.", 4)))
                    + &sdt(&p("", &tracked("del", "Sdt.", 6)))),
            ),
            "accept",
            "[tbl[tr[tc[pOne.][p[sdt][sdt]]]]]",
        ),
        // j23: the first control emptied, the second keeping its words.
        (
            "j23",
            cell(
                &(sdt(&p(&mark("del", 1), &tracked("del", "Sdt.", 2)))
                    + &sdt(&p("", &run("Sdt.")))),
            ),
            "accept",
            "[tbl[tr[tc[p[sdt][sdtSdt.]]]]]",
        ),
        // hn12: in the body.
        (
            "hn12",
            sdt(&p(&mark("del", 1), &tracked("del", "One.", 2))) + &p("", &run("Two.")),
            "accept",
            "[p[sdt]Two.]",
        ),
    ];
    for (word, body, mode, expected) in cases {
        assert_eq!(shape(&resolve(&body, mode)), expected, "{word} {mode}");
    }
}

#[test]
fn a_control_joined_to_a_paragraph_that_keeps_words_becomes_inline() {
    // hn4–hn7: both paragraphs keep words; Word and jubarte agree already.
    let cases = [
        (
            "hn4",
            sdt(&p(&mark("del", 1), &run("One."))) + &p("", &run("Two.")),
            "accept",
            "[p[sdtOne.]Two.]",
        ),
        (
            "hn7",
            cell(&(sdt(&p(&mark("ins", 1), &run("One."))) + &p("", &run("Two.")))),
            "reject",
            "[tbl[tr[tc[p[sdtOne.]Two.]]]]",
        ),
        // hn14: words before a control's paragraph.
        (
            "hn14",
            p(&mark("del", 1), &run("One.")) + &sdt(&p("", &run("Two."))),
            "accept",
            "[pOne.[sdtTwo.]]",
        ),
        // hn15: two controls, both keeping words.
        (
            "hn15",
            sdt(&p(&mark("del", 1), &run("One."))) + &sdt(&p("", &run("Two."))),
            "accept",
            "[p[sdtOne.][sdtTwo.]]",
        ),
    ];
    for (word, body, mode, expected) in cases {
        assert_eq!(shape(&resolve(&body, mode)), expected, "{word} {mode}");
    }
}

#[test]
fn a_join_that_would_split_a_multi_paragraph_control_keeps_the_mark() {
    let two = |kind: &str| sdt(&(p("", &run("X.")) + &p(&mark(kind, 1), &run("Y."))));
    let cases = [
        // hn13: the control's last mark is deleted, the next paragraph
        // keeps words.
        (
            "hn13",
            two("del") + &p("", &run("Z.")),
            "accept",
            "[sdt[pX.][pY.]][pZ.]",
        ),
        // hn25: the same, inserted, on Reject.
        (
            "hn25",
            two("ins") + &p("", &run("Z.")),
            "reject",
            "[sdt[pX.][pY.]][pZ.]",
        ),
        // hn20: words before a two-paragraph control.
        (
            "hn20",
            p(&mark("del", 1), &run("One."))
                + &sdt(&(p("", &run("Two.")) + &p("", &run("Three.")))),
            "accept",
            "[pOne.][sdt[pTwo.][pThree.]]",
        ),
        // hn38: the same, inserted, on Reject.
        (
            "hn38",
            p(&mark("ins", 1), &run("One."))
                + &sdt(&(p("", &run("Two.")) + &p("", &run("Three.")))),
            "reject",
            "[pOne.][sdt[pTwo.][pThree.]]",
        ),
        // hn32: two two-paragraph controls.
        (
            "hn32",
            two("del") + &sdt(&(p("", &run("C.")) + &p("", &run("D.")))),
            "accept",
            "[sdt[pX.][pY.]][sdt[pC.][pD.]]",
        ),
        // hn33: a one-paragraph control after a two-paragraph one.
        (
            "hn33",
            two("del") + &sdt(&p("", &run("C."))),
            "accept",
            "[sdt[pX.][pY.]][sdt[pC.]]",
        ),
        // hn40: a one-paragraph control before a two-paragraph one.
        (
            "hn40",
            sdt(&p(&mark("del", 1), &run("One.")))
                + &sdt(&(p("", &run("Two.")) + &p("", &run("Three.")))),
            "accept",
            "[sdt[pOne.]][sdt[pTwo.][pThree.]]",
        ),
        // hn35: an emptied control of its own brings something.
        (
            "hn35",
            two("del") + &sdt(&p("", &tracked("del", "Z.", 2))),
            "accept",
            "[sdt[pX.][pY.]][sdt[p]]",
        ),
        // hn36: in a cell.
        (
            "hn36",
            cell(&(two("del") + &p("", &run("Z.")))),
            "accept",
            "[tbl[tr[tc[sdt[pX.][pY.]][pZ.]]]]",
        ),
        // hn34: the control's paragraph brings nothing, the one outside it
        // keeps words: the control keeps its paragraph, emptied.
        (
            "hn34",
            sdt(&(p("", &run("X.")) + &p(&mark("del", 1), &tracked("del", "Y.", 2))))
                + &p("", &run("Z.")),
            "accept",
            "[sdt[pX.][p]][pZ.]",
        ),
        // hn39: the same before the control.
        (
            "hn39",
            p(&mark("del", 1), &run("One."))
                + &sdt(&(p("", &tracked("del", "Two.", 2)) + &p("", &run("Three.")))),
            "accept",
            "[pOne.][sdt[p][pThree.]]",
        ),
        // hn41: the paragraph that kept its mark takes the next one's
        // properties.
        (
            "hn41",
            sdt(&(p("", &run("X.")) + &p(&aligned("center", &mark("del", 1)), &run("Y."))))
                + &p(&aligned("right", ""), &run("Z.")),
            "accept",
            "[sdt[pX.][p{jc=right}Y.]][p{jc=right}Z.]",
        ),
        // hn37: a join inside the control is an ordinary join.
        (
            "hn37",
            sdt(&(p(&mark("del", 1), &run("X.")) + &p("", &run("Y.")))),
            "accept",
            "[sdt[pX.Y.]]",
        ),
    ];
    for (word, body, mode, expected) in cases {
        assert_eq!(shape(&resolve(&body, mode)), expected, "{word} {mode}");
    }
}

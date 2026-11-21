// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! Cross-shape round trips through the public compare.
//!
//! Every ordered pair of document shapes below (prose, demos, stamped
//! files, label stubs, nested lists, tables, equations, pictures, cover
//! pages, headings, formatting demos, textless layouts, ...) is compared in
//! both the Word-visual and the PowerTools-faithful preset. Whatever route
//! the comparer takes for a pair (unrelated sources, stamp confetti, label
//! peels, merged replacements, word-level paragraphs), the redline must stay
//! a faithful pair of sources: accepting it yields the revised document's
//! text, paragraph by paragraph, and rejecting it yields the original's.

use super::*;
use crate::test_docs::{Lcg, Shapes, para, plain, split};

const MAIN: &str = "word/document.xml";
const REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/";
const VML: &str = "urn:schemas-microsoft-com:vml";
const MATH: &str = "http://schemas.openxmlformats.org/officeDocument/2006/math";

fn xml(local: &str, body: &str) -> String {
    format!(
        "<w:{local} xmlns:w=\"{}\" xmlns:r=\"{}\" xmlns:m=\"{MATH}\" xmlns:v=\"{VML}\">{body}</w:{local}>",
        W::URI,
        R::URI
    )
}

fn add_part(pkg: &mut PartFs, local: &str, body: &str, kind: &str, ctype: &str) {
    let path = format!("word/{local}.xml");
    pkg.set_part(&path, xml(kind, body).into_bytes());
    pkg.add_content_type_override(&format!("/{path}"), ctype);
    pkg.add_document_relationship(MAIN, &format!("{REL}{local}"), &format!("{local}.xml"));
}

const STYLES: &str = "<w:style w:type='paragraph' w:default='1' w:styleId='Normal'><w:name w:val='Normal'/></w:style>\
<w:style w:type='paragraph' w:styleId='Title'><w:name w:val='Title'/><w:basedOn w:val='Normal'/><w:rPr><w:sz w:val='56'/></w:rPr></w:style>\
<w:style w:type='paragraph' w:styleId='Heading1'><w:name w:val='heading 1'/><w:basedOn w:val='Normal'/><w:pPr><w:outlineLvl w:val='0'/></w:pPr><w:rPr><w:b/></w:rPr></w:style>\
<w:style w:type='paragraph' w:styleId='ListParagraph'><w:name w:val='List Paragraph'/><w:basedOn w:val='Normal'/><w:pPr><w:ind w:left='720'/></w:pPr></w:style>\
<w:style w:type='table' w:styleId='TableGrid'><w:name w:val='Table Grid'/></w:style>";

const NUMBERING: &str = "<w:abstractNum w:abstractNumId='0'><w:multiLevelType w:val='multilevel'/>\
<w:lvl w:ilvl='0'><w:start w:val='1'/><w:numFmt w:val='decimal'/><w:lvlText w:val='%1.'/><w:lvlJc w:val='left'/><w:pPr><w:ind w:left='360' w:hanging='360'/></w:pPr></w:lvl>\
<w:lvl w:ilvl='1'><w:start w:val='1'/><w:numFmt w:val='lowerLetter'/><w:lvlText w:val='%2)'/><w:lvlJc w:val='left'/><w:pPr><w:ind w:left='720' w:hanging='360'/></w:pPr></w:lvl>\
<w:lvl w:ilvl='2'><w:start w:val='1'/><w:numFmt w:val='lowerRoman'/><w:lvlText w:val='%3.'/><w:lvlJc w:val='left'/><w:pPr><w:ind w:left='1080' w:hanging='360'/></w:pPr></w:lvl>\
</w:abstractNum><w:num w:numId='9'><w:abstractNumId w:val='0'/></w:num>";

/// A minimal package holding `body` as its main story, a small style sheet,
/// a three-level numbering definition and, when the body refers to them,
/// footnotes, comments and a default header whose text depends on `side`.
fn package(body: &str, side: usize) -> Vec<u8> {
    let mut pkg = PartFs::open(include_bytes!(
        "../../../../../tests/fixtures/word_probes/tokens/cell_a.docx"
    ))
    .unwrap();
    for part in pkg.parts() {
        pkg.remove_part(&part);
    }
    let empty =
        b"<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"/>";
    pkg.set_part("_rels/.rels", empty.to_vec());
    pkg.set_part("word/_rels/document.xml.rels", empty.to_vec());
    pkg.add_package_relationship(&format!("{REL}officeDocument"), MAIN);
    pkg.add_content_type_override(
        "/word/document.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml",
    );
    let v = VOCAB[side];
    let header = if body.contains("HEADER-STORY") {
        pkg.set_part(
            "word/header1.xml",
            xml(
                "hdr",
                &format!(
                    "<w:p><w:pPr><w:jc w:val='center'/></w:pPr><w:r><w:t>{} running head</w:t></w:r></w:p><w:p><w:r><w:t>{} second line</w:t></w:r></w:p>",
                    v[0], v[6]
                ),
            )
            .into_bytes(),
        );
        pkg.add_content_type_override(
            "/word/header1.xml",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
        );
        let id = pkg.add_document_relationship(MAIN, &format!("{REL}header"), "header1.xml");
        format!("<w:headerReference w:type='default' r:id='{id}'/>")
    } else {
        String::new()
    };
    let section = format!(
        "<w:sectPr>{header}<w:pgSz w:w='12240' w:h='15840'/><w:pgMar w:top='1440' w:right='1440' w:bottom='1440' w:left='1440' w:header='720' w:footer='720' w:gutter='0'/></w:sectPr>"
    );
    pkg.set_part(
        MAIN,
        xml("document", &format!("<w:body>{body}{section}</w:body>")).into_bytes(),
    );
    add_part(
        &mut pkg,
        "styles",
        STYLES,
        "styles",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
    );
    add_part(
        &mut pkg,
        "numbering",
        NUMBERING,
        "numbering",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml",
    );
    if body.contains("w:footnoteReference") {
        let notes: String = (1..=2)
            .map(|id| {
                format!(
                    "<w:footnote w:id='{id}'><w:p><w:pPr><w:pStyle w:val='FootnoteText'/></w:pPr><w:r><w:footnoteRef/></w:r><w:r><w:t xml:space='preserve'> {} note {id} on {}</w:t></w:r></w:p></w:footnote>",
                    v[id], v[(id + 6) % 8]
                )
            })
            .collect();
        add_part(
            &mut pkg,
            "footnotes",
            &format!(
                "<w:footnote w:type='separator' w:id='-1'><w:p><w:r><w:separator/></w:r></w:p></w:footnote><w:footnote w:type='continuationSeparator' w:id='0'><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:footnote>{notes}"
            ),
            "footnotes",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml",
        );
    }
    if body.contains("w:commentReference") {
        let comments: String = (0..=1)
            .map(|id| {
                format!(
                    "<w:comment w:id='{id}' w:author='Reviewer' w:date='2001-01-01T00:00:00Z' w:initials='R'><w:p><w:r><w:t>{} remark {id}</w:t></w:r></w:p></w:comment>",
                    v[id + 2]
                )
            })
            .collect();
        add_part(
            &mut pkg,
            "comments",
            &comments,
            "comments",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
        );
    }
    pkg.to_zip().unwrap()
}

fn table(rows: &[&[&str]]) -> String {
    let cols = rows.first().map_or(1, |r| r.len());
    let grid: String = (0..cols).map(|_| "<w:gridCol w:w='2400'/>").collect();
    let body: String = rows
        .iter()
        .map(|row| {
            let cells: String = row
                .iter()
                .map(|c| {
                    format!(
                        "<w:tc><w:tcPr><w:tcW w:w='2400' w:type='dxa'/></w:tcPr>{}</w:tc>",
                        plain(c)
                    )
                })
                .collect();
            format!("<w:tr>{cells}</w:tr>")
        })
        .collect();
    format!(
        "<w:tbl><w:tblPr><w:tblStyle w:val='TableGrid'/><w:tblW w:w='0' w:type='auto'/></w:tblPr><w:tblGrid>{grid}</w:tblGrid>{body}</w:tbl>"
    )
}

const VOCAB: [[&str; 8]; 3] = [
    [
        "quartz", "bronze", "saffron", "walnut", "orchard", "harvest", "copper", "meadow",
    ],
    [
        "violet", "cobalt", "glacier", "silver", "harbor", "mariner", "sapphire", "summit",
    ],
    [
        "quartz", "bronze", "saffron", "walnut", "orchard", "harvest", "lantern", "meadow",
    ],
];

/// The document shapes. `side` picks a vocabulary: 0 and 1 are disjoint,
/// 2 is side 0 with one word changed in every paragraph.
const FAMILIES: &str = "prose long_prose demo demo_short stamped stamped_long labels \
    label_stubs nested_list uniform_list flat_list table digits_table \
    multi_table math picture textless cover sectioned headings \
    format_demo breaks ooxml_demo first_second single blank_led \
    footnoted commented linked fielded controls textbox tracked moved \
    merged_cells nested_table sectioned_breaks headed symbols \
    paragraph_props";

fn blocks(family: &str, side: usize) -> Vec<String> {
    let v = VOCAB[side];
    let w = |i: usize| v[i % 8];
    let sentence = |i: usize| format!("{} {} {} record {i}", w(i), w(i + 1), w(i + 2));
    let mut out = Vec::new();
    match family {
        "prose" => {
            for i in 0..5 {
                out.push(plain(&sentence(i)));
            }
        }
        "long_prose" => {
            for i in 0..24 {
                out.push(plain(&format!(
                    "{} and the {} of {} carry clause {i}",
                    w(i),
                    w(i + 3),
                    w(i + 5)
                )));
            }
        }
        "demo" => {
            out.push(para(&format!("{} Alignment Demo", w(0)), "Title", None, ""));
            for i in 1..6 {
                out.push(plain(&format!(
                    "This document demonstrates {} {} controls",
                    w(i),
                    w(i + 1)
                )));
            }
        }
        "demo_short" => {
            out.push(para(&format!("{} Highlight Demo", w(1)), "Title", None, ""));
            out.push(plain(&format!("{} text sample", w(2))));
            out.push(plain(&format!("{} closing line", w(3))));
        }
        "stamped" => {
            out.push(plain(&format!("file_{}.docx", 137 + side)));
            for i in 0..4 {
                out.push(plain(&sentence(i)));
            }
        }
        "stamped_long" => {
            out.push(plain(&format!("file_{}.docx", 196 + side)));
            out.push(para(&format!("{} quarterly report", w(0)), "Title", None, ""));
            for i in 0..22 {
                out.push(plain(&format!("{} section {i} of {}", w(i), w(i + 4))));
            }
            out.push(table(&[&["Metric", "Value"], &[w(1), "42"]]));
        }
        "labels" => {
            for (i, label) in ["ONE", "a", "b", "TWO", "c", "d", "THREE", "e"]
                .iter()
                .enumerate()
            {
                let level = if label.len() == 1 { 1 } else { 0 };
                let text = if side == 1 {
                    label.to_lowercase()
                } else {
                    (*label).to_string()
                };
                out.push(para(&text, "ListParagraph", Some(level), ""));
                if i == 3 {
                    out.push(plain(""));
                }
            }
        }
        "label_stubs" => {
            for label in ["One", "Two", "a", "b", "3"] {
                out.push(para(label, "", Some(0), ""));
            }
        }
        "nested_list" => {
            for i in 0..9 {
                let level = [0u8, 1, 1, 0, 1, 2, 0, 1, 1][i];
                out.push(para(
                    &format!("{} {}", w(i), i + 1),
                    "ListParagraph",
                    Some(level),
                    "",
                ));
            }
        }
        "uniform_list" => {
            for _ in 0..4 {
                out.push(para("test", "ListParagraph", Some(0), ""));
            }
        }
        "flat_list" => {
            for i in 0..7 {
                out.push(para(&format!("{} item", w(i)), "", Some(0), ""));
            }
        }
        "table" => {
            out.push(plain(&format!("{} table report", w(0))));
            out.push(table(&[
                &["Name", "Role"],
                &[w(1), w(2)],
                &[w(3), w(4)],
            ]));
            out.push(plain(&format!("{} after table", w(5))));
        }
        "digits_table" => {
            out.push(table(&[&["1", "2"], &["3", "4"]]));
            out.push(plain(""));
        }
        "multi_table" => {
            for t in 0..3 {
                out.push(plain(&format!("{} table {t}", w(t))));
                out.push(table(&[&[w(t + 1), w(t + 2)], &[w(t + 3), w(t + 4)]]));
            }
            out.push(plain(&format!("{} closing", w(7))));
        }
        "math" => {
            out.push(plain(&format!("Math {} formulas", w(0))));
            out.push(String::from("<w:p><m:oMathPara><m:oMath><m:r><m:t>x+y</m:t></m:r></m:oMath></m:oMathPara></w:p>"));
            out.push(plain(&sentence(2)));
            out.push(format!(
                "<w:p><w:r><w:t xml:space='preserve'>{} </w:t></w:r><m:oMath><m:r><m:t>a=b</m:t></m:r></m:oMath></w:p>",
                w(3)
            ));
        }
        "picture" => {
            out.push(plain(&format!("{} picture caption", w(0))));
            out.push(String::from("<w:p><w:r><w:pict><v:rect style='width:40pt;height:20pt'/></w:pict></w:r></w:p>"));
            out.push(plain(&sentence(1)));
            out.push(plain(&sentence(2)));
        }
        "textless" => {
            for i in 0..4 {
                out.push(format!(
                    "<w:p><w:pPr><w:spacing w:after='{}'/></w:pPr></w:p>",
                    120 + 40 * i + side * 10
                ));
            }
        }
        "cover" => {
            out.push(para("Service agreement", "Title", None, ""));
            out.push(plain(&format!("Prepared by {}", w(0))));
            out.push(plain(&format!("January {}", 2026 + side)));
            out.push(plain("Client Name"));
            out.push(String::from("<w:p><w:r><w:br w:type='page'/></w:r></w:p>"));
            for i in 0..3 {
                out.push(plain(&sentence(i)));
            }
        }
        "sectioned" => {
            for i in 0..6 {
                out.push(plain(&format!(
                    "{}) {} section {}",
                    (b'A' + i as u8) as char,
                    w(i),
                    i + 1
                )));
            }
        }
        "headings" => {
            for i in 0..3 {
                out.push(para(&format!("{} heading {i}", w(i)), "Heading1", None, ""));
                out.push(plain(&sentence(i + 1)));
                out.push(plain(&sentence(i + 2)));
            }
        }
        "format_demo" => {
            out.push(para(&format!("{} Formatting Demo", w(0)), "Title", None, ""));
            for (text, rpr) in [
                ("This text is bold", "<w:b/>"),
                ("This text is italic", "<w:i/>"),
                ("This text is underline", "<w:u w:val='single'/>"),
                (
                    "Font sizes improve the heading",
                    "<w:sz w:val='32'/><w:rFonts w:ascii='Verdana' w:hAnsi='Verdana'/>",
                ),
            ] {
                let text = if side == 1 {
                    format!("{text} {}", w(1))
                } else {
                    text.to_string()
                };
                out.push(para(&text, "", None, rpr));
            }
        }
        "breaks" => {
            for i in 0..4 {
                out.push(format!(
                    "<w:p><w:r><w:t>{}</w:t><w:tab/><w:t>{}</w:t><w:br/><w:t>{}</w:t></w:r></w:p>",
                    w(i),
                    w(i + 1),
                    w(i + 2)
                ));
            }
            out.push(String::from("<w:p><w:r><w:br w:type='page'/></w:r></w:p>"));
            out.push(plain(&sentence(5)));
        }
        "ooxml_demo" => {
            out.push(para("OOXML Property Demo", "Title", None, ""));
            out.push(plain(&format!("Sample text with {}", w(0))));
            out.push(para(&format!("Sample text {}", w(1)), "", None, "<w:b/>"));
        }
        "first_second" => {
            out.push(plain("First"));
            out.push(plain(if side == 1 { "Second" } else { "Middle" }));
            out.push(plain("End"));
        }
        "single" => {
            out.push(plain(&format!(
                "{} tightly wraps every {} line of the {} page",
                w(0),
                w(1),
                w(2)
            )));
        }
        "blank_led" => {
            out.push(plain(""));
            out.push(plain(""));
            for i in 0..4 {
                out.push(plain(&sentence(i + 3)));
            }
            out.push(plain(""));
        }
        "footnoted" => {
            for i in 0..4 {
                let note = if i % 2 == 0 {
                    format!(
                        "<w:r><w:rPr><w:vertAlign w:val='superscript'/></w:rPr><w:footnoteReference w:id='{}'/></w:r>",
                        i / 2 + 1
                    )
                } else {
                    String::new()
                };
                out.push(format!(
                    "<w:p><w:r><w:t xml:space='preserve'>{} </w:t></w:r>{note}<w:r><w:t xml:space='preserve'> and {}</w:t></w:r></w:p>",
                    sentence(i),
                    w(i + 4)
                ));
            }
        }
        "commented" => {
            for i in 0..4 {
                if i < 2 {
                    out.push(format!(
                        "<w:p><w:commentRangeStart w:id='{i}'/><w:r><w:t>{}</w:t></w:r><w:commentRangeEnd w:id='{i}'/><w:r><w:commentReference w:id='{i}'/></w:r></w:p>",
                        sentence(i)
                    ));
                } else {
                    out.push(plain(&sentence(i)));
                }
            }
        }
        "linked" => {
            for i in 0..4 {
                out.push(format!(
                    "<w:p><w:bookmarkStart w:id='{i}' w:name='mark{i}'/><w:r><w:t xml:space='preserve'>{} </w:t></w:r><w:bookmarkEnd w:id='{i}'/><w:hyperlink w:anchor='mark{}'><w:r><w:rPr><w:u w:val='single'/></w:rPr><w:t>{} link</w:t></w:r></w:hyperlink></w:p>",
                    sentence(i),
                    (i + 1) % 4,
                    w(i + 3)
                ));
            }
        }
        "fielded" => {
            out.push(format!(
                "<w:p><w:r><w:t xml:space='preserve'>Page </w:t></w:r><w:r><w:fldChar w:fldCharType='begin'/></w:r><w:r><w:instrText xml:space='preserve'> PAGE </w:instrText></w:r><w:r><w:fldChar w:fldCharType='separate'/></w:r><w:r><w:t>{}</w:t></w:r><w:r><w:fldChar w:fldCharType='end'/></w:r></w:p>",
                side + 1
            ));
            out.push(format!(
                "<w:p><w:fldSimple w:instr=' DATE '><w:r><w:t>{} date</w:t></w:r></w:fldSimple></w:p>",
                w(1)
            ));
            for i in 2..5 {
                out.push(plain(&sentence(i)));
            }
        }
        "controls" => {
            out.push(format!(
                "<w:sdt><w:sdtPr><w:alias w:val='Party'/><w:tag w:val='party'/></w:sdtPr><w:sdtContent><w:p><w:r><w:t>{} party block</w:t></w:r></w:p><w:p><w:r><w:t>{} second block</w:t></w:r></w:p></w:sdtContent></w:sdt>",
                w(0),
                w(1)
            ));
            out.push(format!(
                "<w:p><w:r><w:t xml:space='preserve'>Signed by </w:t></w:r><w:sdt><w:sdtPr><w:showingPlcHdr/></w:sdtPr><w:sdtContent><w:r><w:t>{}</w:t></w:r></w:sdtContent></w:sdt></w:p>",
                w(2)
            ));
            for i in 3..6 {
                out.push(plain(&sentence(i)));
            }
        }
        "textbox" => {
            out.push(plain(&format!("{} boxed caption", w(0))));
            out.push(format!(
                "<w:p><w:r><w:pict><v:shape style='width:100pt;height:40pt'><v:textbox><w:txbxContent><w:p><w:r><w:t>{} inside box</w:t></w:r></w:p></w:txbxContent></v:textbox></v:shape></w:pict></w:r></w:p>",
                w(1)
            ));
            for i in 2..5 {
                out.push(plain(&sentence(i)));
            }
        }
        "tracked" => {
            out.push(format!(
                "<w:p><w:r><w:t xml:space='preserve'>{} </w:t></w:r><w:ins w:id='901' w:author='Earlier' w:date='2000-01-01T00:00:00Z'><w:r><w:t>{} added</w:t></w:r></w:ins></w:p>",
                sentence(0),
                w(5)
            ));
            out.push(format!(
                "<w:p><w:r><w:t xml:space='preserve'>{} </w:t></w:r><w:del w:id='902' w:author='Earlier' w:date='2000-01-01T00:00:00Z'><w:r><w:delText>{} removed</w:delText></w:r></w:del></w:p>",
                sentence(1),
                w(6)
            ));
            out.push(format!(
                "<w:p><w:pPr><w:jc w:val='center'/><w:pPrChange w:id='903' w:author='Earlier' w:date='2000-01-01T00:00:00Z'><w:pPr/></w:pPrChange></w:pPr><w:r><w:rPr><w:b/><w:rPrChange w:id='904' w:author='Earlier' w:date='2000-01-01T00:00:00Z'><w:rPr/></w:rPrChange></w:rPr><w:t>{}</w:t></w:r></w:p>",
                sentence(2)
            ));
            out.push(plain(&sentence(3)));
        }
        "moved" => {
            for i in 0..6 {
                out.push(plain(&format!(
                    "{} paragraph {i} keeps {} and {} together with {} in a long line",
                    w(i),
                    w(i + 1),
                    w(i + 2),
                    w(i + 3)
                )));
            }
        }
        "merged_cells" => {
            out.push(plain(&format!("{} merged table", w(0))));
            out.push(format!(
                "<w:tbl><w:tblPr><w:tblW w:w='4800' w:type='dxa'/></w:tblPr><w:tblGrid><w:gridCol w:w='2400'/><w:gridCol w:w='2400'/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w='4800' w:type='dxa'/><w:gridSpan w:val='2'/></w:tcPr>{}</w:tc></w:tr><w:tr><w:tc><w:tcPr><w:vMerge w:val='restart'/></w:tcPr>{}</w:tc><w:tc>{}</w:tc></w:tr><w:tr><w:tc><w:tcPr><w:vMerge/></w:tcPr><w:p/></w:tc><w:tc>{}</w:tc></w:tr></w:tbl>",
                plain(w(1)),
                plain(w(2)),
                plain(w(3)),
                plain(w(4))
            ));
            out.push(plain(""));
        }
        "nested_table" => {
            let inner = table(&[&[w(5), w(6)]]);
            out.push(format!(
                "<w:tbl><w:tblPr><w:tblW w:w='4800' w:type='dxa'/></w:tblPr><w:tblGrid><w:gridCol w:w='4800'/></w:tblGrid><w:tr><w:tc>{}{inner}{}</w:tc></w:tr></w:tbl>",
                plain(w(1)),
                plain(w(2))
            ));
            out.push(plain(&sentence(3)));
        }
        "sectioned_breaks" => {
            for i in 0..4 {
                if i == 1 {
                    out.push(format!(
                        "<w:p><w:pPr><w:sectPr><w:pgSz w:w='12240' w:h='15840' w:orient='portrait'/><w:type w:val='nextPage'/></w:sectPr></w:pPr><w:r><w:t>{}</w:t></w:r></w:p>",
                        sentence(i)
                    ));
                } else {
                    out.push(plain(&sentence(i)));
                }
            }
        }
        "headed" => {
            out.push(plain("HEADER-STORY"));
            for i in 0..3 {
                out.push(plain(&sentence(i)));
            }
        }
        "symbols" => {
            out.push(format!(
                "<w:p><w:r><w:t xml:space='preserve'>{} </w:t><w:sym w:font='Wingdings' w:char='F0E0'/><w:softHyphen/><w:noBreakHyphen/><w:t xml:space='preserve'> {}</w:t></w:r></w:p>",
                w(0),
                w(1)
            ));
            out.push(format!(
                "<w:p><w:r><w:t xml:space='preserve'>{}\u{a0}{} – {}</w:t></w:r></w:p>",
                w(2),
                w(3),
                w(4)
            ));
            out.push(plain(&sentence(5)));
            out.push(plain(&sentence(6)));
        }
        "paragraph_props" => {
            for (i, ppr) in [
                "<w:jc w:val='center'/>",
                "<w:spacing w:before='240' w:after='120' w:line='360' w:lineRule='auto'/>",
                "<w:ind w:left='720' w:hanging='360'/>",
                "<w:keepNext/><w:pBdr><w:top w:val='single' w:sz='4' w:space='1' w:color='auto'/></w:pBdr>",
                "<w:shd w:val='clear' w:color='auto' w:fill='FFFF00'/>",
            ]
            .iter()
            .enumerate()
            {
                out.push(format!(
                    "<w:p><w:pPr>{ppr}</w:pPr><w:r><w:rPr><w:color w:val='{}'/></w:rPr><w:t>{}</w:t></w:r></w:p>",
                    ["FF0000", "00FF00", "0000FF"][side],
                    sentence(i)
                ));
            }
        }
        other => unreachable!("unknown family {other}"),
    }
    out
}

fn parsed(pkg: &PartFs, part: &str) -> Option<(Dom, NodeId)> {
    // A Strict OOXML source keeps the purl.oclc.org namespace until the
    // comparer rewrites it, so read both forms as one.
    let source = pkg.part_string(part)?.replace(
        "http://purl.oclc.org/ooxml/wordprocessingml/main",
        "http://schemas.openxmlformats.org/wordprocessingml/2006/main",
    );
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&source);
    let root = dom.root(doc)?;
    Some((dom, root))
}

/// The non-whitespace characters of every `w:t` under `node`, in order.
/// Of an `mc:AlternateContent` only the first `mc:Choice` counts: the
/// later choices and `mc:Fallback` are other renderings of it, and Word's
/// own compare drops them too (checked with Word on the
/// `alternate_content.docx` corpus pairs).
fn content(dom: &Dom, node: NodeId) -> String {
    let alternative = |a: NodeId| {
        let local = dom.name(a).map(|n| n.local_name().to_string());
        let first_choice = dom
            .parent(a)
            .and_then(|p| dom.elements(p, None).into_iter().next());
        match local.as_deref() {
            Some("Fallback") => true,
            Some("Choice") => first_choice != Some(a),
            _ => false,
        }
    };
    dom.descendants(node, Some(&W::t()))
        .into_iter()
        .filter(|&t| !dom.ancestors(t, None).into_iter().any(alternative))
        .flat_map(|t| dom.value(t).chars().collect::<Vec<_>>())
        .filter(|c| !c.is_whitespace())
        .collect()
}

/// The text content of the body and of the footnotes, in the order the
/// body refers to them. Paragraph boundaries and whitespace are layout,
/// not content; this projection checks that no character is lost,
/// duplicated or reordered. Headers are left out on purpose: Word's own
/// compare carries a header the original lacks into the redline without
/// marking it, so rejecting its redline keeps that header too (checked
/// with Word on the word/clean pair `2f23773d63_… -> 308862bb53_…`).
fn story_content(docx: &[u8]) -> Vec<(&'static str, String)> {
    let pkg = PartFs::open(docx).unwrap();
    let (main, root) = parsed(&pkg, MAIN).unwrap();
    let mut out = vec![("body", content(&main, root))];
    let notes = parsed(&pkg, "word/footnotes.xml");
    let referenced: String = main
        .descendants(root, Some(&W::name("footnoteReference")))
        .into_iter()
        .filter_map(|r| main.attribute(r, &W::id()).map(str::to_string))
        .filter_map(|id| {
            let (dom, notes_root) = notes.as_ref()?;
            dom.descendants(*notes_root, Some(&W::name("footnote")))
                .into_iter()
                .find(|&n| dom.attribute(n, &W::id()) == Some(id.as_str()))
                .map(|n| content(dom, n))
        })
        .collect();
    out.push(("footnotes", referenced));
    out
}

fn presets() -> Vec<(&'static str, WmlComparerSettings)> {
    let stamp = |s: WmlComparerSettings| WmlComparerSettings {
        author_for_revisions: "Shape matrix".into(),
        date_time_for_revisions: "2001-02-03T04:05:06Z".into(),
        ..s
    };
    vec![
        ("word", stamp(WmlComparerSettings::default())),
        (
            "powertools",
            stamp(WmlComparerSettings::powertools_faithful()),
        ),
    ]
}

/// Compare `a` with `b`. `Ok(changed)` when the redline projects back onto
/// both sources (accepting it gives `b`'s content, rejecting it `a`'s),
/// else every divergence (accept, reject or both).
fn round_trip(a: &[u8], b: &[u8], settings: &WmlComparerSettings) -> Result<bool, String> {
    let compared =
        compare_documents_with_settings(a, b, settings).map_err(|e| format!("compare: {e}"))?;
    let accepted = accept_revisions(&compared).map_err(|e| format!("accept: {e}"))?;
    let rejected = reject_revisions(&compared).map_err(|e| format!("reject: {e}"))?;
    let limits = crate::admission::InputLimits::compare();
    let changed = docx_has_tracked_changes(&compared, limits);
    // A source that already carries revisions has no single text that
    // rejecting the new redline must restore (its own revisions may be
    // rejected with it), so such pairs are compared, accepted and rejected
    // without a text expectation.
    if docx_has_tracked_changes(a, limits) || docx_has_tracked_changes(b, limits) {
        return Ok(changed);
    }
    let (acc, rej) = (story_content(&accepted), story_content(&rejected));
    let (want_b, want_a) = (story_content(b), story_content(a));
    let mut wrong = Vec::new();
    if acc != want_b {
        wrong.push(format!("accept {acc:?} != revised {want_b:?}"));
    }
    if rej != want_a {
        wrong.push(format!("reject {rej:?} != original {want_a:?}"));
    }
    if !wrong.is_empty() {
        if let Some(dir) = std::env::var_os("JR_DUMP_DIR") {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            (a, b).hash(&mut h);
            let key = format!("{:016x}", h.finish());
            let dir = std::path::Path::new(&dir);
            std::fs::write(dir.join(format!("{key}-a.docx")), a).unwrap();
            std::fs::write(dir.join(format!("{key}-b.docx")), b).unwrap();
            wrong.push(format!("dump {key}"));
        }
        return Err(wrong.join(" AND "));
    }
    if want_a != want_b && !changed {
        return Err("different texts produced no revision".to_string());
    }
    Ok(changed)
}

/// KNOWN DEFECTS: confirmed engine defects, tracked in #2488, not
/// expected behaviour. These are cases whose redline does not
/// project back onto its sources (accepting it does not give the revised
/// text, or rejecting it does not give the original: content lost,
/// duplicated or reordered). Each entry is the exact case name. Every
/// `[word]` and `[powertools]` entry also diverges through the `jubarte`
/// CLI (`compare --mode`, then `accept` / `reject`); `[word-nomoves]` and
/// `[word-coarse]` are library settings the CLI does not expose. Run a
/// sweep with `JR_DUMP_DIR=<dir>` to write each diverging pair there.
///
/// The list is a ratchet and may only shrink: a case outside it that
/// diverges fails the sweep, and so does a listed case that round-trips
/// (remove its entry). Never add an entry here; a new divergence is a bug
/// to fix or to report in #2488.
const KNOWN_DEFECTS: &[&str] = &[
    "controls -> demo/1 [word]",
    "controls -> moved/0 [word]",
    "controls -> moved/1 [word]",
    "controls -> nested_list/1 [word]",
    "controls -> paragraph_props/0 [powertools]",
    "controls -> paragraph_props/0 [word]",
    "controls -> sectioned/0 [word]",
    "controls -> sectioned/1 [word]",
    "controls -> table/1 [word]",
    "cover -> controls/1 [word]",
    "demo_short/0 -> swap_ends [word]",
    "demo_short/1 -> swap_ends [word]",
    "flat_list -> textbox/0 [word]",
    "footnoted -> controls/1 [word]",
    "headed -> controls/1 [word]",
    "linked -> controls/1 [word]",
    "math -> controls/1 [word]",
    "math -> headings/0 [word]",
    "multi_table/0 -> delete_middle [word]",
    "multi_table/1 -> delete_middle [word]",
    "nested_table -> controls/1 [word]",
    "nested_table -> demo/0 [word]",
    "nested_table -> demo/1 [word]",
    "nested_table -> fielded/1 [word]",
    "nested_table -> flat_list/0 [word]",
    "nested_table -> flat_list/1 [word]",
    "nested_table -> format_demo/0 [word]",
    "nested_table -> format_demo/1 [word]",
    "nested_table -> headed/1 [word]",
    "nested_table -> headings/1 [word]",
    "nested_table -> moved/0 [word]",
    "nested_table -> moved/1 [word]",
    "nested_table -> nested_list/0 [word]",
    "nested_table -> nested_list/1 [word]",
    "nested_table -> paragraph_props/1 [word]",
    "nested_table -> prose/1 [word]",
    "nested_table -> sectioned/0 [word]",
    "nested_table -> sectioned/1 [word]",
    "nested_table -> symbols/0 [word]",
    "nested_table -> symbols/1 [word]",
    "nested_table -> uniform_list/0 [word]",
    "nested_table -> uniform_list/1 [word]",
    "paragraph_props -> controls/0 [powertools]",
    "paragraph_props -> controls/0 [word]",
    "paragraph_props -> cover/0 [word]",
    "paragraph_props -> headed/0 [word]",
    "paragraph_props -> picture/0 [word]",
    "picture -> controls/1 [word]",
    "random 1000+11 [word-nomoves]",
    "random 1000+11 [word]",
    "random 1000+3 [word-coarse]",
    "random 1000+3 [word]",
    "random 2000+42 [word-coarse]",
    "random 2000+42 [word]",
    "random 3000+61 [word-coarse]",
    "random 3000+61 [word]",
    "template 10+45 [word-coarse]",
    "template 10+58 [word-coarse]",
    "template 10+58 [word-nomoves]",
    "template 10+58 [word]",
    "template 20000+115 [word-coarse]",
    "template 20000+115 [word-nomoves]",
    "template 20000+115 [word]",
    "template 30000+53 [word-coarse]",
    "template 30000+53 [word-nomoves]",
    "template 30000+53 [word]",
    "template 40000+111 [word-coarse]",
    "template 40000+111 [word-nomoves]",
    "template 40000+111 [word]",
    "template 40000+118 [word-coarse]",
    "template 40000+118 [word-nomoves]",
    "template 40000+118 [word]",
    "template 40000+152 [word-coarse]",
    "template 40000+152 [word-nomoves]",
    "template 40000+152 [word]",
    "template 40000+157 [word-coarse]",
    "template 40000+157 [word]",
];

fn check(results: &[(String, Result<bool, String>)]) {
    let known = |context: &str| KNOWN_DEFECTS.contains(&context);
    let failures: Vec<String> = results
        .iter()
        .filter_map(|(context, r)| match r {
            Err(e) if !known(context) => Some(format!("{context}: {e}")),
            Ok(_) if known(context) => Some(format!(
                "{context}: listed in KNOWN_DEFECTS but now round-trips; remove it"
            )),
            _ => None,
        })
        .collect();
    assert!(
        failures.is_empty(),
        "{} of {} round trips break the known-defect ratchet:\n{}",
        failures.len(),
        results.len(),
        failures.join("\n")
    );
}

/// Compare every family in `left` against every family (disjoint
/// vocabulary), and against its own lightly edited copy.
fn sweep(left: &[&str]) {
    let docs: Vec<(&str, usize, Vec<u8>)> = split(FAMILIES)
        .into_iter()
        .flat_map(|f| (0..3).map(move |side| (f, side)))
        .map(|(f, side)| (f, side, package(&blocks(f, side).concat(), side)))
        .collect();
    let mut results = Vec::new();
    for (fa, _, a) in docs.iter().filter(|d| left.contains(&d.0) && d.1 == 0) {
        for (fb, sb, b) in &docs {
            if *sb == 2 && fb != fa {
                continue;
            }
            for (name, settings) in presets() {
                let context = format!("{fa} -> {fb}/{sb} [{name}]");
                results.push((context, round_trip(a, b, &settings)));
            }
        }
    }
    check(&results);
}

/// The edits applied to one shape: move, delete, insert, duplicate,
/// reformat, split, join and reword blocks.
fn mutations(blocks: &[String], side: usize) -> Vec<(&'static str, Vec<String>)> {
    let n = blocks.len();
    let mut out = Vec::new();
    if n >= 2 {
        let mut moved = blocks.to_vec();
        let first = moved.remove(0);
        moved.push(first);
        out.push(("move_first_to_end", moved));
        let mut deleted = blocks.to_vec();
        deleted.remove(n / 2);
        out.push(("delete_middle", deleted));
        let mut swapped = blocks.to_vec();
        swapped.swap(0, n - 1);
        out.push(("swap_ends", swapped));
    }
    let mut inserted = blocks.to_vec();
    inserted.insert(
        n / 2,
        plain(&format!(
            "{} inserted clause for review",
            VOCAB[1][side + 2]
        )),
    );
    out.push(("insert_middle", inserted));
    // A copy of a block holding ids (notes, comments, bookmarks) would
    // repeat them.
    if !blocks[0].contains("w:id=") {
        let mut duplicated = blocks.to_vec();
        duplicated.push(blocks[0].clone());
        out.push(("duplicate_first", duplicated));
    }
    out.push((
        "bold_everything",
        blocks
            .iter()
            .map(|b| {
                b.replace("<w:r><w:rPr>", "<w:r><w:rPr><w:b/>")
                    .replace("<w:r><w:t", "<w:r><w:rPr><w:i/></w:rPr><w:t")
            })
            .collect(),
    ));
    out.push((
        "split_runs",
        blocks
            .iter()
            .map(|b| {
                let word = format!("{} ", VOCAB[side][1]);
                if b.contains("delText") {
                    b.clone()
                } else {
                    b.replacen(
                        &word,
                        &format!(
                            "{}</w:t></w:r><w:r><w:t xml:space='preserve'> ",
                            VOCAB[side][1]
                        ),
                        1,
                    )
                }
            })
            .collect(),
    ));
    out.push((
        "reword_last",
        blocks
            .iter()
            .enumerate()
            .map(|(i, b)| {
                if i + 1 == n {
                    b.replace(VOCAB[side][0], "zephyr")
                        .replace(VOCAB[side][3], "tundra")
                } else {
                    b.clone()
                }
            })
            .collect(),
    ));
    if n >= 2 {
        // Join two simple paragraphs: the second's runs move into the first.
        let simple = |b: &str| b.starts_with("<w:p>") && b.matches("<w:p>").count() == 1;
        if simple(&blocks[0]) && simple(&blocks[1]) {
            let second = blocks[1].trim_start_matches("<w:p>");
            let runs = second
                .split_once("</w:pPr>")
                .map_or(second, |(_, rest)| rest);
            let mut joined = blocks.to_vec();
            joined.remove(1);
            joined[0] = format!("{}{runs}", blocks[0].trim_end_matches("</w:p>"));
            out.push(("join_first_two", joined));
        }
    }
    out.push(("empty_tail", {
        let mut v = blocks.to_vec();
        v.push(plain(""));
        v.push(plain(""));
        v
    }));
    out.push(("emptied", vec![plain("")]));
    out
}

/// Every mutation of every family in `left`, compared both ways.
fn edit_sweep(left: &[&str]) {
    let mut results = Vec::new();
    for &family in left {
        for side in [0, 1] {
            let base = blocks(family, side);
            let a = package(&base.concat(), side);
            for (edit, edited) in mutations(&base, side) {
                let b = package(&edited.concat(), side);
                for (name, settings) in presets() {
                    results.push((
                        format!("{family}/{side} -> {edit} [{name}]"),
                        round_trip(&a, &b, &settings),
                    ));
                    results.push((
                        format!("{family}/{side} <- {edit} [{name}]"),
                        round_trip(&b, &a, &settings),
                    ));
                }
            }
        }
    }
    check(&results);
}

#[test]
fn prose_and_demo_shapes_round_trip_against_every_shape() {
    sweep(&split("prose long_prose demo demo_short single"));
}

#[test]
fn stamped_and_cover_shapes_round_trip_against_every_shape() {
    sweep(&split("stamped stamped_long cover blank_led"));
}

#[test]
fn list_shapes_round_trip_against_every_shape() {
    sweep(&split(
        "labels label_stubs nested_list uniform_list flat_list",
    ));
}

#[test]
fn table_shapes_round_trip_against_every_shape() {
    sweep(&split(
        "table digits_table multi_table merged_cells nested_table",
    ));
}

#[test]
fn payload_shapes_round_trip_against_every_shape() {
    sweep(&split("math picture textless breaks textbox symbols"));
}

#[test]
fn styled_shapes_round_trip_against_every_shape() {
    sweep(&split(
        "sectioned headings format_demo ooxml_demo first_second paragraph_props",
    ));
}

#[test]
fn annotated_shapes_round_trip_against_every_shape() {
    sweep(&split(
        "footnoted commented linked fielded controls tracked moved sectioned_breaks headed",
    ));
}

#[test]
fn prose_edits_round_trip() {
    edit_sweep(&split(
        "prose long_prose demo demo_short single moved cover blank_led stamped",
    ));
}

#[test]
fn list_and_table_edits_round_trip() {
    edit_sweep(&split(
        "labels label_stubs nested_list uniform_list flat_list table digits_table multi_table merged_cells nested_table",
    ));
}

#[test]
fn payload_and_style_edits_round_trip() {
    edit_sweep(&split(
        "math picture textless breaks textbox symbols sectioned headings format_demo ooxml_demo first_second paragraph_props",
    ));
}

#[test]
fn annotated_edits_round_trip() {
    edit_sweep(&split(
        "footnoted commented linked fielded controls tracked sectioned_breaks headed stamped_long",
    ));
}

/// A small deterministic generator (64-bit LCG) for the random sweep.
const POOL: &str = "agreement party the of and text sample demo bold italic table row \
    item list first second end summary section clause quartz bronze \
    saffron walnut orchard harvest copper meadow violet cobalt glacier \
    silver harbor mariner sapphire summit test a b ONE TWO 3 this \
    document demonstrates heading is with";

/// One random block drawn from the shapes the comparer special-cases.
fn random_block(rng: &mut Lcg, words: &[&str]) -> String {
    let sentence = |rng: &mut Lcg, n: usize| -> String {
        (0..n)
            .map(|_| words[rng.below(words.len())])
            .collect::<Vec<_>>()
            .join(" ")
    };
    match rng.below(20) {
        0..=5 => {
            let n = 1 + rng.below(10);
            plain(&sentence(rng, n))
        }
        6 => para(&sentence(rng, 3), "Title", None, ""),
        7 => para(&sentence(rng, 2), "Heading1", None, ""),
        8 | 9 => {
            let level = rng.below(3) as u8;
            let n = 1 + rng.below(3);
            para(&sentence(rng, n), "ListParagraph", Some(level), "")
        }
        10 => plain(""),
        11 => {
            let rows = 1 + rng.below(3);
            let cols = 1 + rng.below(3);
            let mut cells: Vec<Vec<String>> = Vec::new();
            for _ in 0..rows {
                let mut row = Vec::new();
                for _ in 0..cols {
                    let n = rng.below(3);
                    row.push(sentence(rng, n));
                }
                cells.push(row);
            }
            let refs: Vec<Vec<&str>> = cells
                .iter()
                .map(|r| r.iter().map(String::as_str).collect())
                .collect();
            let rows: Vec<&[&str]> = refs.iter().map(Vec::as_slice).collect();
            table(&rows)
        }
        12 => "<w:p><m:oMathPara><m:oMath><m:r><m:t>x+y</m:t></m:r></m:oMath></m:oMathPara></w:p>"
            .to_string(),
        13 => "<w:p><w:r><w:pict><v:rect style='width:40pt;height:20pt'/></w:pict></w:r></w:p>"
            .to_string(),
        14 => format!(
            "<w:p><w:r><w:pict><v:shape style='width:90pt;height:30pt'><v:textbox><w:txbxContent>{}</w:txbxContent></v:textbox></v:shape></w:pict></w:r></w:p>",
            plain(&sentence(rng, 3))
        ),
        15 => format!("{} Demo", sentence(rng, 2)),
        16 => plain(&format!("file_{}.docx", 100 + rng.below(3))),
        17 => {
            let rpr = [
                "<w:b/>",
                "<w:i/>",
                "<w:u w:val='single'/>",
                "<w:sz w:val='32'/>",
            ][rng.below(4)];
            para(&sentence(rng, 4), "", None, rpr)
        }
        18 => format!(
            "<w:p><w:r><w:t>{}</w:t><w:tab/><w:t>{}</w:t><w:br/><w:t>{}</w:t></w:r></w:p>",
            words[rng.below(words.len())],
            words[rng.below(words.len())],
            words[rng.below(words.len())]
        ),
        _ => "<w:p><w:r><w:br w:type='page'/></w:r></w:p>".to_string(),
    }
}

/// A random document of `len` blocks over `words`.
fn random_doc(rng: &mut Lcg, words: &[&str], len: usize) -> Vec<String> {
    (0..len)
        .map(|_| {
            let b = random_block(rng, words);
            if b.starts_with('<') {
                b
            } else {
                // A bare title text: style it as a document title.
                para(&b, "Title", None, "")
            }
        })
        .collect()
}

/// Random edits of `doc`: delete, insert, replace, move and reformat
/// blocks.
fn random_edit(rng: &mut Lcg, doc: &[String], words: &[&str]) -> Vec<String> {
    let mut out = doc.to_vec();
    for _ in 0..1 + rng.below(4) {
        match rng.below(5) {
            0 if out.len() > 1 => {
                let i = rng.below(out.len());
                out.remove(i);
            }
            1 => {
                let i = rng.below(out.len() + 1);
                out.insert(i, random_doc(rng, words, 1).remove(0));
            }
            2 if !out.is_empty() => {
                let i = rng.below(out.len());
                out[i] = random_doc(rng, words, 1).remove(0);
            }
            3 if out.len() > 1 => {
                let i = rng.below(out.len());
                let b = out.remove(i);
                let j = rng.below(out.len() + 1);
                out.insert(j, b);
            }
            _ if !out.is_empty() => {
                let i = rng.below(out.len());
                out[i] = out[i].replace("<w:r><w:rPr>", "<w:r><w:rPr><w:b/>");
            }
            _ => {}
        }
    }
    if out.is_empty() {
        out.push(plain(""));
    }
    out
}

/// The two presets plus Word-mode variants that open other routes:
/// no move detection, case-insensitive words, a coarser detail threshold
/// and conflated non-breaking spaces.
fn all_presets() -> Vec<(&'static str, WmlComparerSettings)> {
    let mut out = presets();
    let word = out[0].1.clone();
    out.push((
        "word-nomoves",
        WmlComparerSettings {
            detect_moves: false,
            detect_format_changes: false,
            ..word.clone()
        },
    ));
    out.push((
        "word-coarse",
        WmlComparerSettings {
            detail_threshold: 0.15,
            case_insensitive: true,
            conflate_breaking_and_nonbreaking_spaces: true,
            simplify_move_markup: true,
            ..word
        },
    ));
    out
}

/// `count` seeded random pairs starting at `seed`: half are independent
/// documents over overlapping vocabularies, half are edits of one
/// document. Every pair must round trip in both presets.
fn random_sweep(seed: u64, count: usize) {
    seeded_sweep(
        "random",
        seed,
        count,
        0x9e37_79b9_7f4a_7c15,
        |rng, left, right| {
            let span = if rng.chance(20) { 30 } else { 8 };
            let len_a = 1 + rng.below(span);
            let a = random_doc(rng, left, len_a);
            let b = if rng.chance(50) {
                random_edit(rng, &a, right)
            } else {
                let span = if rng.chance(20) { 30 } else { 8 };
                let len_b = 1 + rng.below(span);
                random_doc(rng, right, len_b)
            };
            (a, b)
        },
    );
}

/// `count` seeded cases from `seed`: each draws two overlapping windows of
/// [`POOL`] and builds a pair from them with `make`.
fn seeded_sweep(
    label: &str,
    seed: u64,
    count: usize,
    stride: u64,
    make: impl Fn(&mut Lcg, &[&str], &[&str]) -> (Vec<String>, Vec<String>),
) {
    let pool = split(POOL);
    let mut results = Vec::new();
    for case in 0..count {
        let mut rng = Lcg(seed.wrapping_add(case as u64).wrapping_mul(stride));
        let start = rng.below(pool.len() - 12);
        let width = 4 + rng.below(16);
        let left_words = &pool[start..(start + width).min(pool.len())];
        let shift = rng.below(12);
        let right_start = (start + shift).min(pool.len() - 4);
        let right_words = &pool[right_start..(right_start + width).min(pool.len())];
        let (a, b) = make(&mut rng, left_words, right_words);
        let (a, b) = (package(&a.concat(), 0), package(&b.concat(), 0));
        for (name, settings) in all_presets() {
            results.push((
                format!("{label} {seed}+{case} [{name}]"),
                round_trip(&a, &b, &settings),
            ));
        }
    }
    check(&results);
}

#[test]
fn random_pairs_round_trip_0() {
    random_sweep(1, 150);
}

#[test]
fn random_pairs_round_trip_1() {
    random_sweep(1_000, 150);
}

#[test]
fn random_pairs_round_trip_2() {
    random_sweep(2_000, 150);
}

#[test]
fn random_pairs_round_trip_3() {
    random_sweep(3_000, 150);
}

/// `docx` with an XML comment after every element of its WordprocessingML
/// parts. Comments carry no content; the comparer must read past them in
/// every container it walks (body, properties, styles, numbering, notes,
/// comments and headers).
fn with_xml_comments(docx: &[u8]) -> Vec<u8> {
    with_noise(docx, "<!--n-->")
}

/// `noise` after every closing and empty tag of every `word/` XML part
/// (and a comment after the root start tag).
fn with_noise(docx: &[u8], noise: &str) -> Vec<u8> {
    let mut pkg = PartFs::open(docx).unwrap();
    for part in pkg.parts() {
        if !part.starts_with("word/") || !part.ends_with(".xml") {
            continue;
        }
        let source = pkg.part_string(&part).unwrap();
        let mut noisy = String::with_capacity(source.len() * 2);
        let mut rest = source.as_str();
        // Past the root start tag only: nothing may precede the root.
        let Some(root_open) = rest
            .match_indices('<')
            .map(|(i, _)| i)
            .find(|&i| rest[i + 1..].starts_with(|c: char| c.is_ascii_alphabetic()))
        else {
            continue;
        };
        let root_end = root_open + rest[root_open..].find('>').unwrap() + 1;
        noisy.push_str(&rest[..root_end]);
        noisy.push_str("<!--root-->");
        rest = &rest[root_end..];
        while let Some(i) = rest.find('>') {
            let (head, tail) = rest.split_at(i + 1);
            noisy.push_str(head);
            // After a closing or empty tag, never inside text.
            let tag = &head[head.rfind('<').unwrap_or(0)..];
            if tag.starts_with("</") || tag.ends_with("/>") {
                noisy.push_str(noise);
            }
            rest = tail;
        }
        noisy.push_str(rest);
        pkg.set_part(&part, noisy.into_bytes());
    }
    pkg.to_zip().unwrap()
}

#[test]
fn xml_comments_in_every_part_round_trip() {
    let mut results = Vec::new();
    for family in split(FAMILIES) {
        for side in [0, 1] {
            let base = blocks(family, side);
            let a = with_xml_comments(&package(&base.concat(), side));
            let mut edits = mutations(&base, side);
            edits.truncate(4);
            let other = with_xml_comments(&package(&blocks(family, 1 - side).concat(), 1 - side));
            for (edit, edited) in edits {
                let b = with_xml_comments(&package(&edited.concat(), side));
                for (name, settings) in presets() {
                    results.push((
                        format!("{family}/{side} -> {edit} [{name}]"),
                        round_trip(&a, &b, &settings),
                    ));
                }
            }
            for (name, settings) in presets() {
                results.push((
                    format!("{family}/{side} -> commented other [{name}]"),
                    round_trip(&a, &other, &settings),
                ));
            }
        }
    }
    check(&results);
}

/// Whole-document templates for the shapes the unrelated-source and
/// paragraph-window heuristics fingerprint: alpha-list clusters, cover
/// pages, label stubs, field demos, annotations, equation demos, word
/// statistics, memos, property testers, table-dominated reports, single
/// wrapped paragraphs, stamped files, demo cousins, lettered sections and
/// colon lists. Random filler blocks over `words` pad some of them.
fn template_doc(rng: &mut Lcg, words: &[&str]) -> Vec<String> {
    let pick = |rng: &mut Lcg| words[rng.below(words.len())];
    let shapes = Shapes {
        pool: words,
        filler_span: 9,
    };
    let filler = |rng: &mut Lcg, n: usize| shapes.filler(rng, n);
    let mut out: Vec<String> = Vec::new();
    match rng.below(24) {
        0 => {
            for label in ["ONE", "a", "TWO", "b"] {
                out.push(para(label, "", Some(u8::from(label.len() == 1)), ""));
            }
        }
        1 => {
            for label in ["ONE", "a", "b", "c", "TWO", "d", "e", "THREE", "f"] {
                out.push(para(label, "", Some(u8::from(label.len() == 1)), ""));
            }
        }
        2 => {
            out.push(para(&format!("{} agreement", pick(rng)), "Title", None, ""));
            out.push(plain(&format!("Prepared by {}", pick(rng))));
            out.push(plain("March 10, 2040"));
            out.push(plain("[Client Name]"));
            out.push(plain(&format!("{}@example.com", pick(rng))));
        }
        3 => {
            for label in ["a", "x", "x", "b", "c"] {
                out.push(plain(label));
            }
        }
        4 => {
            out.push(para("Fields test", "Title", None, ""));
            out.push(plain("html input type text"));
            out.extend(filler(rng, 2));
        }
        5 => {
            out.push(plain(&format!(
                "Oftentimes {} authors suggest amendments",
                pick(rng)
            )));
            out.push(plain("Please leave a comment on this paragraph"));
        }
        6 => {
            out.push(para("Math borderBox demo", "Title", None, ""));
            for _ in 0..6 {
                out.push(format!(
                    "<w:p><w:r><w:t xml:space='preserve'>{} </w:t></w:r><m:oMath><m:borderBox><m:e><m:r><m:t>x</m:t></m:r></m:e></m:borderBox></m:oMath></w:p>",
                    pick(rng)
                ));
            }
        }
        7 => {
            out.push(plain("Num words 20"));
            out.push(plain("Num chars 120"));
            out.push(plain("Num pages 3"));
            if rng.chance(50) {
                out.push("<w:p><w:r><w:br w:type='page'/></w:r></w:p>".to_string());
            }
            out.push(plain("test"));
            out.push(plain("page 3"));
        }
        8 => {
            out.extend(shapes.memo(rng, 14, 10));
        }
        9 => {
            out.push(para("OOXML w:b property tester", "Title", None, ""));
            out.push(para("Bold on", "", None, "<w:b/>"));
            out.push(para("Bold off", "", None, "<w:b w:val='0'/>"));
            if rng.chance(50) {
                out.push(table(&[&["w:sz", "24"], &["w:color", "red"]]));
            }
        }
        10 => {
            out.push(plain(&format!("{} report", pick(rng))));
            for t in 0..3 + rng.below(3) {
                let a = format!("{} {t}", pick(rng));
                let b = pick(rng).to_string();
                out.push(table(&[&[a.as_str(), b.as_str()], &["1", "2"]]));
            }
        }
        11 => {
            let n = 30 + rng.below(20);
            out.extend(filler(rng, n));
            for t in 0..4 {
                let a = format!("{} total {t}", pick(rng));
                out.push(table(&[&[a.as_str(), "9"]]));
                out.extend(filler(rng, 2));
            }
        }
        12 => {
            out.push(table(&[
                &["Name", "Role", "Site"],
                &["Ann", "Lead", "Rome"],
                &["Bo", "Dev", "Oslo"],
            ]));
        }
        13 => {
            let n = 20 + rng.below(10);
            out.push(plain(
                &(0..n)
                    .map(|_| words[rng.below(words.len())])
                    .collect::<Vec<_>>()
                    .join(" "),
            ));
        }
        14 => {
            out.extend(shapes.essay(rng, 3, 25));
        }
        15 => {
            out.extend(shapes.demo(rng));
        }
        16 => {
            out.extend(shapes.list_demo(rng));
        }
        17 => {
            out.extend(shapes.lettered(rng));
        }
        18 => {
            out.push(para("Tab Alignment", "Title", None, ""));
            out.push(plain(""));
            out.push(plain(&format!("left\t{}", pick(rng))));
            out.push(plain(&format!("right\t{}", pick(rng))));
            out.push(plain(""));
        }
        19 => {
            out.push(para("Tab Tests", "Title", None, ""));
            out.push(plain("CENTER TAB"));
            out.push(plain("First Second End"));
        }
        20 => {
            out.push(para(&format!("{} Tester", pick(rng)), "Title", None, ""));
            out.extend(filler(rng, 3));
            if rng.chance(50) {
                out.push(table(&[&["k", "v"]]));
            }
        }
        21 => {
            out.push(para(&format!("{} summary", pick(rng)), "Title", None, ""));
            for i in 0..4 {
                out.push(plain(&format!("{}: {} {i}", pick(rng), pick(rng))));
            }
        }
        22 => {
            out.push(para("SD-2672 plain 3x3", "Title", None, ""));
            out.push(plain(""));
            out.push(table(&[
                &["1", "2", "3"],
                &["4", "5", "6"],
                &["7", "8", "9"],
            ]));
        }
        _ => {
            out.push(para(
                &format!("{} heading", pick(rng)),
                "Heading1",
                None,
                "",
            ));
            let n = 4 + rng.below(12);
            for i in 0..n {
                let level = (i % 3) as u8;
                out.push(para(
                    &format!("{} {}", pick(rng), i + 1),
                    "ListParagraph",
                    Some(level),
                    "",
                ));
            }
        }
    }
    if rng.chance(30) {
        let n = 1 + rng.below(4);
        out.extend(filler(rng, n));
    }
    if rng.chance(20) {
        out.insert(0, plain(""));
    }
    out
}

/// `count` seeded template pairs: two templates over overlapping
/// vocabularies, or one template and its random edit, in every preset.
fn template_sweep(seed: u64, count: usize) {
    seeded_sweep(
        "template",
        seed,
        count,
        0x2545_f491_4f6c_dd1d,
        |rng, left, right| {
            let a = template_doc(rng, left);
            let b = if rng.chance(25) {
                random_edit(rng, &a, right)
            } else {
                template_doc(rng, right)
            };
            (a, b)
        },
    );
}

#[test]
fn template_pairs_round_trip_0() {
    template_sweep(10, 200);
}

#[test]
fn template_pairs_round_trip_1() {
    template_sweep(20_000, 200);
}

#[test]
fn template_pairs_round_trip_2() {
    template_sweep(30_000, 200);
}

#[test]
fn template_pairs_round_trip_3() {
    template_sweep(40_000, 200);
}

/// Real documents from the repository's fixtures, paired across files
/// no fixture compares: Word styles, numbering, notes, comments, fields,
/// equations, pictures, sections and strict markup in combinations the
/// synthetic shapes do not produce.
const FIXTURES: &str = "1676-word.docx 1677-word.docx 1683-case1-word.docx \
    1686-word-compare.docx f4/original.docx redline/original.docx \
    redline/modified.docx nested_table/hf_a.docx \
    nested_table/hf_b.docx sections/multi_section.docx \
    multibody/multibody.docx relids/hyperlink_doc.docx \
    relids/image_doc.docx math/builtin-equations.docx \
    math/quadratic-in-table.docx math/matrix-subscripts.docx \
    math/word2007-equations.docx from-docx/equations.docx \
    from-docx/critic/synthetic/all.accepted.docx \
    from-docx/critic/synthetic/tables.accepted.docx \
    from-docx/critic/synthetic/hard-notes.accepted.docx \
    from-docx/critic/synthetic/hard-lists.accepted.docx \
    from-docx/critic/synthetic/structure.accepted.docx \
    from-docx/critic/synthetic/comments.accepted.docx \
    agent-view/received.docx agent-view/word-checklist.docx \
    re-dialect/annex.docx re-dialect/step1.docx \
    inline-pictures/1975-header.docx inline-pictures/1975-textbox.docx \
    md-roundtrip/demo.docx word_probes/keep_next_picture_kn_1001.docx";

/// Each fixture in `left` against the fixtures a few places after it (a
/// fixed spread of offsets keeps the sweep to a few hundred pairs).
/// Each fixture of `list` (paths under `tests/fixtures`) with its bytes.
fn fixtures(list: &str) -> Vec<(&str, Vec<u8>)> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures");
    split(list)
        .into_iter()
        .map(|name| (name, std::fs::read(root.join(name)).unwrap()))
        .collect()
}

fn fixture_sweep(left: std::ops::Range<usize>) {
    const OFFSETS: [usize; 7] = [1, 2, 3, 5, 8, 13, 21];
    let settings = presets().remove(0).1;
    let small = fixtures(FIXTURES);
    let mut results = Vec::new();
    for i in left {
        let (name_a, a) = &small[i];
        for offset in OFFSETS {
            let (name_b, b) = &small[(i + offset) % small.len()];
            results.push((
                format!("fixture {name_a} -> {name_b} [word]"),
                round_trip(a, b, &settings),
            ));
        }
    }
    check(&results);
}

#[test]
fn fixture_pairs_round_trip_0() {
    fixture_sweep(0..8);
}

#[test]
fn fixture_pairs_round_trip_1() {
    fixture_sweep(8..16);
}

#[test]
fn fixture_pairs_round_trip_2() {
    fixture_sweep(16..24);
}

#[test]
fn fixture_pairs_round_trip_3() {
    fixture_sweep(24..split(FIXTURES).len());
}

/// The larger fixtures (long contracts, a strict-markup package) cost
/// seconds per comparison, so each meets two partners, both ways.
const LARGE_FIXTURES: &str = "redline-inpi/original-new.docx redline-inpi/modified-new-2.docx \
    strict/Strict01.docx f4/modified.docx";

fn large_fixture_sweep(range: std::ops::Range<usize>) {
    let settings = presets().remove(0).1;
    let (small, large) = (fixtures(FIXTURES), fixtures(LARGE_FIXTURES));
    let mut results = Vec::new();
    for i in range {
        let (name_l, l) = &large[i];
        for offset in [3, 11] {
            let (name_s, s) = &small[(i * 7 + offset) % small.len()];
            results.push((
                format!("fixture {name_l} -> {name_s} [word]"),
                round_trip(l, s, &settings),
            ));
            results.push((
                format!("fixture {name_s} -> {name_l} [word]"),
                round_trip(s, l, &settings),
            ));
        }
    }
    check(&results);
}

#[test]
fn large_fixture_pairs_round_trip_0() {
    large_fixture_sweep(0..2);
}

#[test]
fn large_fixture_pairs_round_trip_1() {
    large_fixture_sweep(2..4);
}

/// The sorted `.docx` files of a corpus folder under `tests/corpus`.
fn corpus(folder: &str) -> Vec<(String, Vec<u8>)> {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/corpus")
        .join(folder);
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "docx"))
        .collect();
    files.sort();
    // About ten evenly spaced files per folder keeps the crate's test time
    // in budget; the full folders are the corpus's own pair tests.
    let step = files.len().div_ceil(10).max(1);
    let files: Vec<_> = files.into_iter().step_by(step).collect();
    files
        .into_iter()
        .map(|p| {
            (
                p.file_name().unwrap().to_string_lossy().into_owned(),
                std::fs::read(&p).unwrap(),
            )
        })
        .collect()
}

/// Corpus documents in `part` of `parts` against the documents `offsets`
/// places after them: pairs the corpus's own pair tests never make.
fn corpus_sweep(folder: &str, offsets: &[usize], part: usize, parts: usize) {
    corpus_sweep_in(folder, offsets, part, parts, "word");
}

/// [`corpus_sweep`] in the named preset of [`all_presets`].
fn corpus_sweep_in(folder: &str, offsets: &[usize], part: usize, parts: usize, preset: &str) {
    let docs = corpus(folder);
    let settings = all_presets()
        .into_iter()
        .find(|(name, _)| *name == preset)
        .unwrap()
        .1;
    let mut results = Vec::new();
    for i in (0..docs.len()).filter(|i| i % parts == part) {
        for offset in offsets {
            let (name_a, a) = &docs[i];
            let (name_b, b) = &docs[(i + offset) % docs.len()];
            results.push((
                format!("corpus {name_a} -> {name_b} [{preset}]"),
                round_trip(a, b, &settings),
            ));
        }
    }
    check(&results);
}

#[test]
fn superdoc_corpus_pairs_round_trip_0() {
    corpus_sweep(
        "bench/neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source",
        &[1, 17, 43],
        0,
        4,
    );
}

#[test]
fn superdoc_corpus_pairs_round_trip_1() {
    corpus_sweep(
        "bench/neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source",
        &[1, 17, 43],
        1,
        4,
    );
}

#[test]
fn superdoc_corpus_pairs_round_trip_2() {
    corpus_sweep(
        "bench/neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source",
        &[1, 17, 43],
        2,
        4,
    );
}

#[test]
fn superdoc_corpus_pairs_round_trip_3() {
    corpus_sweep(
        "bench/neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source",
        &[1, 17, 43],
        3,
        4,
    );
}

const WORD_BASED: &str = "bench/neurotic_docx_bench/corpus/word_based/docx_source";
const WORD_CLEAN: &str = "bench/neurotic_docx_bench/corpus/word/clean/docx";
const RANDOMIZED: &str = "bench/neurotic_docx_bench/corpus/word_based/docx_source_randomized";
const COMMENTED: &str = "bench/neurotic_docx_bench/corpus/word/with_comments_tracking/docx";
const STAMPED: &str = "broken_ones_two/sources";

#[test]
fn word_based_corpus_pairs_round_trip_0() {
    corpus_sweep(WORD_BASED, &[1, 7, 19], 0, 2);
}

#[test]
fn word_based_corpus_pairs_round_trip_1() {
    corpus_sweep(WORD_BASED, &[1, 7, 19], 1, 2);
}

#[test]
fn word_clean_corpus_pairs_round_trip_0() {
    corpus_sweep(WORD_CLEAN, &[1, 7, 19], 0, 2);
}

#[test]
fn word_clean_corpus_pairs_round_trip_1() {
    corpus_sweep(WORD_CLEAN, &[1, 7, 19], 1, 2);
}

#[test]
fn randomized_corpus_pairs_round_trip() {
    corpus_sweep(RANDOMIZED, &[1, 7, 19], 0, 1);
}

#[test]
fn commented_corpus_pairs_round_trip() {
    corpus_sweep(COMMENTED, &[1, 7, 19], 0, 1);
}

#[test]
fn stamped_corpus_pairs_round_trip_0() {
    corpus_sweep(STAMPED, &[1, 29], 0, 3);
}

#[test]
fn stamped_corpus_pairs_round_trip_1() {
    corpus_sweep(STAMPED, &[1, 29], 1, 3);
}

#[test]
fn stamped_corpus_pairs_round_trip_2() {
    corpus_sweep(STAMPED, &[1, 29], 2, 3);
}

const UNACCEPTED: &str = "bench/_to_improve_accepted_changes/_original_unaccepted_docx";
const REJECTED: &str = "bench/neurotic_docx_bench/grok_run/wr0928/rejected_tracking/docx";
const GROK_WORD_BASED: &str = "bench/neurotic_docx_bench/grok_run/word_based/docx_source";
const TRACKING: &str = "bench/neurotic_docx_bench/corpus/word/tracking_without_comments/docx";

#[test]
fn unaccepted_corpus_pairs_round_trip_0() {
    corpus_sweep(UNACCEPTED, &[1, 13], 0, 2);
}

#[test]
fn unaccepted_corpus_pairs_round_trip_1() {
    corpus_sweep(UNACCEPTED, &[1, 13], 1, 2);
}

#[test]
fn rejected_corpus_pairs_round_trip_0() {
    corpus_sweep(REJECTED, &[1, 31], 0, 3);
}

#[test]
fn rejected_corpus_pairs_round_trip_1() {
    corpus_sweep(REJECTED, &[1, 31], 1, 3);
}

#[test]
fn rejected_corpus_pairs_round_trip_2() {
    corpus_sweep(REJECTED, &[1, 31], 2, 3);
}

#[test]
fn grok_word_based_corpus_pairs_round_trip() {
    corpus_sweep(GROK_WORD_BASED, &[1, 7], 0, 1);
}

#[test]
fn tracking_corpus_pairs_round_trip_0() {
    corpus_sweep(TRACKING, &[1], 0, 3);
}

#[test]
fn tracking_corpus_pairs_round_trip_1() {
    corpus_sweep(TRACKING, &[1], 1, 3);
}

#[test]
fn tracking_corpus_pairs_round_trip_2() {
    corpus_sweep(TRACKING, &[1], 2, 3);
}

const SUPERDOC: &str = "bench/neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source";

#[test]
fn superdoc_corpus_pairs_round_trip_powertools() {
    corpus_sweep_in(SUPERDOC, &[1, 17], 0, 1, "powertools");
}

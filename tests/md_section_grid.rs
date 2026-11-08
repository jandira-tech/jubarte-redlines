// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! A section's column line (`w:cols w:sep`) and line grid (`w:docGrid`)
//! come back. Demo.docx's second section draws a line between its two
//! columns and both sections snap their lines to a 15.6pt grid; the round
//! trip dropped both, so the columns lost their divider and every line moved.

mod common;

use common::docx::{docx_with_sect_pr, part_string};
use jubarte::changes::{ChangeFilter, reject_changes};
use jubarte::markdown::{DocxOptions, ReadOptions, TrackChanges, markdown_to_docx, read};

fn view(docx: &[u8]) -> String {
    let options = ReadOptions {
        page_markers: false,
        track_changes: TrackChanges::All,
        ..ReadOptions::default()
    };
    read(docx, &options)
        .expect("the docx reads")
        .markdown
        .lines()
        .filter(|l| !l.starts_with("source: ") && !l.starts_with("sha256: "))
        .collect::<Vec<_>>()
        .join("\n")
}

fn write(markdown: &str) -> Vec<u8> {
    let docx = markdown_to_docx(markdown, &DocxOptions::default())
        .expect("the Markdown reads")
        .docx;
    common::validity::assert_word_valid_package(&docx);
    docx
}

const PAGE: &str = r#"<w:pgSz w:w="11906" w:h="16838"/><w:pgMar w:top="1440" w:right="1800" w:bottom="1440" w:left="1800" w:header="851" w:footer="992" w:gutter="0"/>"#;

/// A two-section docx: section 1 (one paragraph) ends with `first`, the
/// body's final section is `last`; each is the tail after the page.
fn two_sections(first: &str, last: &str) -> Vec<u8> {
    let body = format!(
        "<w:p><w:pPr><w:sectPr>{PAGE}{first}</w:sectPr></w:pPr><w:r><w:t>One</w:t></w:r></w:p>\
         <w:p><w:r><w:t>Two</w:t></w:r></w:p>"
    );
    docx_with_sect_pr(&body, &[], &format!("<w:sectPr>{PAGE}{last}</w:sectPr>"))
}

/// The `w:sectPr`s of `docx`'s document, in order.
fn sections(docx: &[u8]) -> Vec<String> {
    let document = part_string(docx, "word/document.xml").unwrap();
    document
        .split("<w:sectPr")
        .skip(1)
        .map(|s| s.split("</w:sectPr>").next().unwrap().to_string())
        .collect()
}

/// The live `w:sectPr`s of `docx`'s document, in order: a
/// `w:sectPrChange`'s saved section is not one of them.
fn live_sections(docx: &[u8]) -> Vec<String> {
    let mut document = part_string(docx, "word/document.xml").unwrap();
    while let Some(at) = document.find("<w:sectPrChange") {
        let end = document[at..].find("</w:sectPrChange>").unwrap() + at;
        document.replace_range(at..end + "</w:sectPrChange>".len(), "");
    }
    document
        .split("<w:sectPr")
        .skip(1)
        .map(|s| s.split("</w:sectPr>").next().unwrap().to_string())
        .collect()
}

/// The saved `w:sectPr` of the last `w:sectPrChange` in `docx`.
fn saved_section(docx: &[u8]) -> String {
    let document = part_string(docx, "word/document.xml").unwrap();
    let at = document.rfind("<w:sectPrChange").expect("a section change");
    let rest = &document[at..];
    rest[..rest.find("</w:sectPrChange>").unwrap()].to_string()
}

#[test]
fn a_column_line_comes_back() {
    let source = two_sections(
        r#"<w:cols w:space="720"/>"#,
        r#"<w:cols w:space="360" w:num="2" w:sep="1"/>"#,
    );
    let first = view(&source);
    assert!(
        first.contains("columns 2, column gap 0.25in, column line"),
        "{first}"
    );
    let again = write(&first);
    let sects = sections(&again);
    assert!(sects[1].contains(r#"w:num="2""#), "{sects:?}");
    assert!(sects[1].contains(r#"w:sep="1""#), "{sects:?}");
    assert!(!sects[0].contains("w:sep"), "{sects:?}");
    assert_eq!(view(&again), first);
}

#[test]
fn a_line_grid_comes_back_in_every_section() {
    let grid = r#"<w:docGrid w:linePitch="312" w:charSpace="0" w:type="lines"/>"#;
    let source = two_sections(
        &format!(r#"<w:cols w:space="720"/>{grid}"#),
        &format!(r#"<w:cols w:space="360" w:num="2"/>{grid}"#),
    );
    let first = view(&source);
    assert!(first.contains("line grid 15.6pt"), "{first}");
    let again = write(&first);
    for sect in sections(&again) {
        assert!(
            sect.contains(r#"<w:docGrid w:type="lines" w:linePitch="312"/>"#),
            "{sect}"
        );
    }
    assert_eq!(view(&again), first);
}

#[test]
fn a_section_that_states_its_type_comes_back() {
    // `w:type` says how a section starts; `nextPage` is the default, but a
    // document that states it keeps it (ids-authors, ids-comments, #528),
    // and `continuous` must come back too (ids-font-sizes).
    for ty in ["nextPage", "continuous", "evenPage", "oddPage"] {
        let source = two_sections(
            format!(r#"<w:type w:val="{ty}"/><w:cols w:space="720"/>"#).as_str(),
            format!(r#"<w:type w:val="{ty}"/><w:cols w:space="720"/>"#).as_str(),
        );
        let first = view(&source);
        assert!(first.contains(&format!("starts {ty}")), "{ty}:\n{first}");
        let again = write(&first);
        for sect in sections(&again) {
            assert!(
                sect.contains(&format!(r#"<w:type w:val="{ty}"/>"#)),
                "{ty}: {sect}"
            );
        }
        assert_eq!(view(&again), first);
    }
}

#[test]
fn a_section_that_states_no_type_writes_none() {
    let source = two_sections(r#"<w:cols w:space="720"/>"#, r#"<w:cols w:space="720"/>"#);
    let first = view(&source);
    assert!(!first.contains("starts "), "{first}");
    let again = write(&first);
    for sect in sections(&again) {
        assert!(!sect.contains("<w:type"), "{sect}");
    }
    assert_eq!(view(&again), first);
}

#[test]
fn a_title_page_switched_off_is_not_a_first_page() {
    // `w:val="0"` states the default: no first page of the section's own.
    // The round trip turned it into `first: none`, and the docx it wrote
    // gained a `w:titlePg` Word's document never had (ids-tables, #528).
    let source = two_sections(
        r#"<w:cols w:space="720"/>"#,
        r#"<w:cols w:space="720"/><w:titlePg w:val="0"/>"#,
    );
    let first = view(&source);
    assert!(!first.contains("first: none"), "{first}");
    let again = write(&first);
    for sect in sections(&again) {
        assert!(!sect.contains("titlePg"), "{sect}");
    }
    assert_eq!(view(&again), first);
}

#[test]
fn a_title_page_switched_on_names_its_first_page() {
    let source = two_sections(
        r#"<w:cols w:space="720"/>"#,
        r#"<w:cols w:space="720"/><w:titlePg/>"#,
    );
    let first = view(&source);
    assert!(first.contains("first: none"), "{first}");
    let again = write(&first);
    assert!(sections(&again)[1].contains("<w:titlePg/>"));
    assert_eq!(view(&again), first);
}

#[test]
fn a_grid_word_does_not_snap_to_is_not_printed() {
    // Word's own template writes `w:linePitch` with no type: no grid.
    let source = two_sections(
        r#"<w:cols w:space="720"/><w:docGrid w:linePitch="360"/>"#,
        r#"<w:cols w:space="720"/><w:docGrid w:linePitch="360"/>"#,
    );
    let first = view(&source);
    assert!(!first.contains("grid"), "{first}");
    assert!(!first.contains("column line"), "{first}");
}

#[test]
fn a_character_grid_keeps_its_pitch_and_spacing() {
    let grid = r#"<w:docGrid w:type="linesAndChars" w:linePitch="360" w:charSpace="4096"/>"#;
    let source = two_sections(grid, grid);
    let first = view(&source);
    assert!(
        first.contains("line and char grid 18pt, char space 4096"),
        "{first}"
    );
    let again = write(&first);
    for sect in sections(&again) {
        assert!(
            sect.contains(
                r#"<w:docGrid w:type="linesAndChars" w:linePitch="360" w:charSpace="4096"/>"#
            ),
            "{sect}"
        );
    }
}

#[test]
fn a_character_grid_without_a_line_pitch_comes_back() {
    // `tests/convert_docx_to_pdf.rs`'s character grid: `snapToChars` sets
    // only character spacing, so `w:linePitch` is absent.
    let grid = r#"<w:docGrid w:type="snapToChars" w:charSpace="40960"/>"#;
    let source = two_sections(grid, grid);
    let first = view(&source);
    assert!(first.contains("char grid, char space 40960"), "{first}");
    let again = write(&first);
    for sect in sections(&again) {
        assert!(sect.contains(grid), "{sect}");
    }
    assert_eq!(view(&again), first);
}

/// #561 review defect 4: the sizes Word's own paper list names round-trip
/// exactly. The view names the sheet as Word does, the writer writes
/// Word's own twips back, and the view of that names the same sheet again.
#[test]
fn word_s_named_page_sizes_round_trip_exactly() {
    let cases = [
        (12_240, 15_840, "", "Letter"),
        (12_240, 20_160, "", "Legal"),
        (10_440, 15_120, "", "Executive"),
        (15_840, 24_480, "", "Tabloid"),
        (24_480, 15_840, r#" w:orient="landscape""#, "Ledger"),
        (16_838, 23_811, "", "A3"),
        (11_906, 16_838, "", "A4"),
        (8_391, 11_906, "", "A5"),
        (14_570, 20_636, "", "B4 (JIS)"),
        (10_318, 14_570, "", "B5 (JIS)"),
    ];
    for (w, h, orient, name) in cases {
        let source = docx_with_sect_pr(
            r#"<w:p><w:r><w:t>One</w:t></w:r></w:p>"#,
            &[],
            &format!(
                r#"<w:sectPr><w:pgSz w:w="{w}" w:h="{h}"{orient}/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720" w:gutter="0"/></w:sectPr>"#
            ),
        );
        let first = view(&source);
        let expected = format!(
            "page: {name} {}, margins 1in, header/footer 0.5in",
            if orient.is_empty() {
                "portrait"
            } else {
                "landscape"
            }
        );
        assert!(first.contains(&expected), "{name}:\n{first}");
        // The writer writes the same twips back.
        let again = write(&first);
        let document = part_string(&again, "word/document.xml").unwrap();
        assert!(
            document.contains(&format!(r#"<w:pgSz w:w="{w}" w:h="{h}"{orient}/>"#)),
            "{name}:\n{document}"
        );
        assert_eq!(view(&again), first, "{name}");
    }
}

#[test]
fn rejecting_a_section_change_restores_the_earlier_page() {
    // Section 2 is Letter. Its `w:sectPrChange` keeps the earlier A4.
    // The view must carry that page: copying Letter into the change makes
    // Reject leave Letter (#1475).
    let earlier = r#"<w:pgSz w:w="11906" w:h="16838"/><w:pgMar w:top="1701" w:right="1134" w:bottom="1701" w:left="1134" w:header="708" w:footer="708" w:gutter="0"/>"#;
    let letter = r#"<w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1417" w:right="1701" w:bottom="1417" w:left="1701" w:header="720" w:footer="720" w:gutter="0"/>"#;
    let change = format!(
        r#"<w:sectPrChange w:id="1" w:author="Alice" w:date="2026-10-17T09:00:00Z"><w:sectPr>{earlier}</w:sectPr></w:sectPrChange>"#
    );
    let body = format!(
        "<w:p><w:pPr><w:sectPr>{PAGE}</w:sectPr></w:pPr><w:r><w:t>One</w:t></w:r></w:p>\
         <w:p><w:r><w:t>Two</w:t></w:r></w:p>"
    );
    let source = docx_with_sect_pr(
        &body,
        &[],
        &format!("<w:sectPr>{letter}{change}</w:sectPr>"),
    );
    let markdown = view(&source);
    let again = write(&markdown);
    let rejected = reject_changes(&again, &ChangeFilter::default()).expect("reject");
    let sects = live_sections(&rejected);
    let last = sects.last().expect("a section");
    assert!(
        last.contains(r#"w:w="11906""#) && last.contains(r#"w:h="16838""#),
        "reject kept the new page:\n{last}"
    );
    assert!(
        !last.contains(r#"w:w="12240""#),
        "Letter survived reject:\n{last}"
    );
    assert!(
        !part_string(&rejected, "word/document.xml")
            .unwrap()
            .contains("w:sectPrChange"),
        "reject left the section change"
    );
}

#[test]
fn rejecting_ids_sections_restores_the_recorded_page() {
    // The corpus change keeps 11900x16840. The fixture must say that page,
    // and Reject of the docx the fixture writes must put it back (#1475).
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let corpus = std::fs::read(root.join(
        "tests/corpus/neurotic_docx_bench/grok_run/wr0928/rejected_tracking/docx/3c920d5996_rejected_tracking.docx",
    ))
    .expect("corpus");
    let markdown = view(&corpus);
    let section = markdown
        .lines()
        .find(|line| line.contains("fmt: r1 @APS"))
        .expect("section 2");
    let fixture = std::fs::read_to_string(root.join("tests/fixtures/md-roundtrip/ids-sections.md"))
        .expect("fixture");
    let fixture_section = fixture
        .lines()
        .find(|line| line.contains("fmt: r1 @APS"))
        .expect("fixture section");
    assert_eq!(section, fixture_section, "the source view is the fixture");
    let again = write(&fixture);
    let rejected = reject_changes(&again, &ChangeFilter::default()).expect("reject");
    let last = live_sections(&rejected).last().expect("a section").clone();
    assert!(
        last.contains(r#"w:w="11900""#) && last.contains(r#"w:h="16840""#),
        "reject kept the new page:\n{last}"
    );
    assert!(
        last.contains(r#"w:top="1701""#) && last.contains(r#"w:header="708""#),
        "reject dropped the earlier margins:\n{last}"
    );
    assert!(
        !last.contains(r#"w:w="12240""#),
        "Letter survived reject:\n{last}"
    );
    assert!(
        !part_string(&rejected, "word/document.xml")
            .unwrap()
            .contains("w:sectPrChange"),
        "reject left the section change"
    );
}

const LETTER: &str = r#"<w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1417" w:right="1701" w:bottom="1417" w:left="1701" w:header="720" w:footer="720" w:gutter="0"/>"#;
const EARLIER_A4: &str = r#"<w:pgSz w:w="11906" w:h="16838"/><w:pgMar w:top="1701" w:right="1134" w:bottom="1701" w:left="1134" w:header="708" w:footer="708" w:gutter="0"/>"#;

/// Two sections; the last is `live` with a `w:sectPrChange` saving `saved`.
fn changed_last_section(live: &str, saved: &str, extras: &[common::docx::Part<'_>]) -> Vec<u8> {
    let body = format!(
        "<w:p><w:pPr><w:sectPr>{PAGE}</w:sectPr></w:pPr><w:r><w:t>One</w:t></w:r></w:p>\
         <w:p><w:r><w:t>Two</w:t></w:r></w:p>"
    );
    let change = format!(
        r#"<w:sectPrChange w:id="1" w:author="Alice" w:date="2026-10-17T09:00:00Z"><w:sectPr>{saved}</w:sectPr></w:sectPrChange>"#
    );
    docx_with_sect_pr(
        &body,
        extras,
        &format!("<w:sectPr>{live}{change}</w:sectPr>"),
    )
}

#[test]
fn rejecting_a_section_change_keeps_its_title_page() {
    // The change resized the page; the section's first page of its own
    // was on before and after. Reject must not switch it off.
    let source = changed_last_section(
        &format!("{LETTER}<w:titlePg/>"),
        &format!("{EARLIER_A4}<w:titlePg/>"),
        &[],
    );
    let again = write(&view(&source));
    assert!(
        saved_section(&again).contains("<w:titlePg/>"),
        "the saved section lost its title page:\n{}",
        saved_section(&again)
    );
    let rejected = reject_changes(&again, &ChangeFilter::default()).expect("reject");
    let last = live_sections(&rejected).last().expect("a section").clone();
    assert!(
        last.contains(r#"w:w="11906""#),
        "reject kept Letter:\n{last}"
    );
    assert!(
        last.contains("<w:titlePg"),
        "reject dropped the title page:\n{last}"
    );
}

#[test]
fn a_later_section_declaration_without_an_earlier_page_drops_it() {
    // The last declaration of a section wins: one that names no earlier
    // page leaves the change keeping the live page.
    let source = changed_last_section(LETTER, EARLIER_A4, &[]);
    let markdown = view(&source);
    let line = markdown
        .lines()
        .find(|line| line.contains("; was: "))
        .expect("a section names its earlier page")
        .to_string();
    let was = line.find("; was: ").unwrap();
    let after = line[was..].find(", fmt: ").expect("a format tag") + was;
    let plain = format!("{}{}", &line[..was], &line[after..]);
    let markdown = markdown.replacen(&line, &format!("{line}\n{plain}"), 1);
    let again = write(&markdown);
    let saved = saved_section(&again);
    assert!(
        saved.contains(r#"w:w="12240""#),
        "a stale earlier page:\n{saved}"
    );
}

#[test]
fn header_text_naming_an_earlier_page_is_not_one() {
    // The change keeps the live page; the header's own words merely read
    // like a `; was:` clause.
    let xml = format!(
        r#"<w:hdr xmlns:w="{}"><w:p><w:r><w:t>Revision; was: A4 portrait; draft</w:t></w:r></w:p></w:hdr>"#,
        "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
    );
    let header = common::docx::Part {
        name: "word/header1.xml",
        content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
        rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
        xml: &xml,
    };
    let source = changed_last_section(
        &format!(r#"<w:headerReference w:type="default" r:id="rIdX0"/>{LETTER}"#),
        LETTER,
        &[header],
    );
    let markdown = view(&source);
    assert!(
        markdown.contains("Revision; was: A4 portrait"),
        "{markdown}"
    );
    let again = write(&markdown);
    let saved = saved_section(&again);
    assert!(
        saved.contains(r#"w:w="12240""#) && !saved.contains(r#"w:w="11906""#),
        "header text became the earlier page:\n{saved}"
    );
}

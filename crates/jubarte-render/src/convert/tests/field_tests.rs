// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;

#[test]
fn field_instruction_is_not_visible_text() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body><w:p>
  <w:r><w:t xml:space="preserve">Page </w:t></w:r>
  <w:r><w:fldChar w:fldCharType="begin"/></w:r>
  <w:r><w:instrText>PAGE</w:instrText></w:r>
  <w:r><w:fldChar w:fldCharType="separate"/></w:r>
  <w:r><w:t>1</w:t></w:r>
  <w:r><w:fldChar w:fldCharType="end"/></w:r>
  <w:r><w:t xml:space="preserve"> of </w:t></w:r>
  <w:r><w:fldChar w:fldCharType="begin"/></w:r>
  <w:r><w:instrText>NUMPAGES</w:instrText></w:r>
  <w:r><w:fldChar w:fldCharType="separate"/></w:r>
  <w:r><w:t>2</w:t></w:r>
  <w:r><w:fldChar w:fldCharType="end"/></w:r>
</w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
    let joined: String = runs.iter().map(|r| r.text.as_str()).collect();
    assert_eq!(joined, "Page 1 of 2");
    assert!(!joined.contains("PAGE"), "{joined}");
    assert!(!joined.contains("NUMPAGES"), "{joined}");
}

#[test]
fn civil_from_unix_days_epoch_and_y2k() {
    assert_eq!(civil_from_unix_days(0), (1970, 1, 1));
    assert_eq!(civil_from_unix_days(-1), (1969, 12, 31));
    assert_eq!(civil_from_unix_days(10_957), (2000, 1, 1));
    assert_eq!(format_date_picture("yyyy", 1970, 1, 1), "1970");
}

#[test]
fn date_picture_reads_quoted_switch() {
    assert_eq!(date_picture(r#" DATE \@ "yyyy" "#).as_deref(), Some("yyyy"));
    assert_eq!(
        date_picture(r#"DATE \@ "d MMMM yyyy""#).as_deref(),
        Some("d MMMM yyyy")
    );
    assert_eq!(format_date_picture("yyyy", 2020, 3, 15), "2020");
    assert_eq!(
        format_date_picture("d MMMM yyyy", 2020, 3, 15),
        "15 March 2020"
    );
    assert_eq!(format_date_picture("M/d/yyyy", 2020, 3, 15), "3/15/2020");
}

fn civil(y: i32, m: u32, d: u32, h: u32, min: u32, s: u32) -> CivilDateTime {
    CivilDateTime { y, m, d, h, min, s }
}

#[test]
fn w3cdtf_offsets_fold_into_utc() {
    // #140: numeric zones were dropped with everything past char 19.
    assert_eq!(
        parse_w3cdtf("2024-01-01T00:30:00+01:00"),
        Some(civil(2023, 12, 31, 23, 30, 0))
    );
    assert_eq!(
        parse_w3cdtf("2024-02-28T22:15:00-05:30"),
        Some(civil(2024, 2, 29, 3, 45, 0)),
        "negative offsets roll forward across a leap day"
    );
    assert_eq!(
        parse_w3cdtf("2018-07-04T15:30:00.25Z"),
        Some(civil(2018, 7, 4, 15, 30, 0))
    );
    assert_eq!(
        parse_w3cdtf("2018-07-04T15:30Z"),
        Some(civil(2018, 7, 4, 15, 30, 0))
    );
    assert_eq!(parse_w3cdtf("2018-07-04"), Some(civil(2018, 7, 4, 0, 0, 0)));
    for bad in [
        "",
        "2018-07",
        "2018-13-01",
        "2018-07-04T1:30:00Z",
        "2018-07-04T15:30:00+0100",
        "2018-07-04T15:30:00Zjunk",
    ] {
        assert_eq!(parse_w3cdtf(bad), None, "{bad:?}");
    }
}

#[test]
fn core_dates_read_elements_not_text() {
    let core = core_dates_from_xml(
        r#"<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/"><dc:title>Guide:created for review</dc:title><dc:description>&lt;cp:lastPrinted&gt;1999</dc:description><dcterms:created>2018-07-04T15:30:00Z</dcterms:created><cp:lastPrinted>2019-11-22T08:00:00Z</cp:lastPrinted></cp:coreProperties>"#,
    );
    assert_eq!(core.created, Some(civil(2018, 7, 4, 15, 30, 0)));
    assert_eq!(core.printed, Some(civil(2019, 11, 22, 8, 0, 0)));
    assert_eq!(core.modified, None);
}

#[test]
fn unix_seconds_map_to_utc_civil_time() {
    assert_eq!(civil_from_unix_secs(0), civil(1970, 1, 1, 0, 0, 0));
    assert_eq!(
        civil_from_unix_secs(1_700_000_000),
        civil(2023, 11, 14, 22, 13, 20)
    );
    assert_eq!(civil_from_unix_secs(-1), civil(1969, 12, 31, 23, 59, 59));
    for days in [-719_468_i64, -1, 0, 59, 11_016, 19_782, 2_932_896] {
        let (y, m, d) = civil_from_unix_days(days);
        assert_eq!(unix_days_from_civil(y, m, d), days);
    }
}

#[test]
fn time_field_paints_the_given_utc_instant() {
    // #140: deterministic HHmm instead of "any four digits".
    let now = civil(2023, 11, 14, 22, 13, 20);
    assert_eq!(datetime_field_text_at(" TIME \\@ \"HHmm\" ", now), "2213");
    assert_eq!(datetime_field_text_at(" TIME ", now), "10:13 PM");
    assert_eq!(datetime_field_text_at(" DATE ", now), "11/14/2023");
}

#[test]
fn ref_bookmark_skips_pageref() {
    assert_eq!(
        ref_bookmark(" REF _HereRef \\h ").as_deref(),
        Some("_HereRef")
    );
    assert_eq!(ref_bookmark("PAGEREF _HereRef \\h "), None);
    assert_eq!(pageref_bookmark(" REF _HereRef "), None);
    assert!(ref_copies_bookmark_text(" REF _HereRef \\h "));
    assert!(!ref_copies_bookmark_text(" REF _HereRef \\r \\h "));
    assert!(!ref_copies_bookmark_text(" REF _HereRef \\w \\h "));
    assert!(is_date_field(" DATE \\@ \"yyyy\" "));
    assert!(!is_date_field("CREATEDATE"));
    assert!(is_createdate_field(" CREATEDATE \\@ \"yyyy\" "));
    assert!(is_time_field(" TIME \\@ \"HHmm\" "));
    assert!(is_printdate_field(" PRINTDATE \\@ \"yyyy\" "));
    assert!(is_savedate_field(" SAVEDATE \\@ \"yyyy\" "));
    assert!(!is_savedate_field("CREATEDATE"));
    assert!(!is_time_field("DATE"));
    assert!(is_numwords_field(" NUMWORDS "));
    assert!(!is_numwords_field("NUMPAGES"));
    assert_eq!(
        parse_w3cdtf("2018-07-04T15:30:00Z"),
        Some(CivilDateTime {
            y: 2018,
            m: 7,
            d: 4,
            h: 15,
            min: 30,
            s: 0,
        })
    );
    assert_eq!(
        format_datetime_picture(
            "HHmm",
            CivilDateTime {
                y: 2018,
                m: 7,
                d: 4,
                h: 15,
                min: 30,
                s: 0,
            }
        ),
        "1530"
    );
    assert_eq!(
        format_datetime_picture(
            "h:mm am/pm",
            CivilDateTime {
                y: 2018,
                m: 7,
                d: 4,
                h: 15,
                min: 30,
                s: 0,
            }
        ),
        "3:30 pm"
    );
}

#[test]
fn uncached_date_field_paints_picture_not_instr() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body><w:p>
  <w:r><w:t xml:space="preserve">WhenDx </w:t></w:r>
  <w:r><w:fldChar w:fldCharType="begin"/></w:r>
  <w:r><w:instrText xml:space="preserve"> DATE \@ "yyyy" </w:instrText></w:r>
  <w:r><w:fldChar w:fldCharType="end"/></w:r>
</w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
    let joined: String = runs.iter().map(|r| r.text.as_str()).collect();
    assert!(!joined.contains("DATE"), "{joined}");
    assert!(
        joined.chars().any(|c| c.is_ascii_digit()),
        "uncached DATE must paint a year; {joined}"
    );
    assert!(joined.starts_with("WhenDx "), "{joined}");
}

#[test]
fn omml_ssup_marks_sup_run_super() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
<w:body><w:p>
<m:oMath>
  <m:sSup>
    <m:e><m:r><m:t>x</m:t></m:r></m:e>
    <m:sup><m:r><m:t>2</m:t></m:r></m:sup>
  </m:sSup>
</m:oMath>
</w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
    let joined: String = runs.iter().map(|r| r.text.as_str()).collect();
    assert_eq!(
        joined,
        "\u{1D465}2",
        "runs={:?}",
        runs.iter().map(|r| r.text.as_str()).collect::<Vec<_>>()
    );
    let two = runs.iter().find(|r| r.text == "2").expect("2 run");
    assert!(
        matches!(two.style.vert, VertAlign::Super),
        "2 must be Super; n={} texts={:?}",
        runs.len(),
        runs.iter().map(|r| r.text.as_str()).collect::<Vec<_>>()
    );
}

#[test]
fn omml_nary_emits_sum_chr_with_sub_sup() {
    // Strict01 binomial: m:nary chr=∑, sub=k=0, sup=n. convert
    // currently skips naryPr/chr and paints sub/sup at baseline.
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
<w:body><w:p>
<m:oMath>
  <m:nary>
    <m:naryPr><m:chr m:val="∑"/></m:naryPr>
    <m:sub><m:r><m:t>k=0</m:t></m:r></m:sub>
    <m:sup><m:r><m:t>n</m:t></m:r></m:sup>
    <m:e><m:r><m:t>x</m:t></m:r></m:e>
  </m:nary>
</m:oMath>
</w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
    let joined: String = runs.iter().map(|r| r.text.as_str()).collect();
    assert!(
        joined.contains('∑'),
        "nary chr must emit ∑; joined={joined:?}"
    );
    let sub = runs
        .iter()
        .find(|r| r.text.contains("\u{1D458}=0"))
        .expect("sub");
    let sup = runs.iter().find(|r| r.text == "\u{1D45B}").expect("sup");
    assert!(
        matches!(sub.style.vert, VertAlign::Sub),
        "nary sub must be Sub"
    );
    assert!(
        matches!(sup.style.vert, VertAlign::Super),
        "nary sup must be Super"
    );
}

/// The runs of the first paragraph of a `w:document` body fragment.
fn math_runs(body: &str) -> Vec<TextRun> {
    let xml = format!(
        r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
<w:body>{body}</w:body></w:document>"#
    );
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default())
}

#[test]
fn a_tracked_change_inside_a_math_box_keeps_its_marks() {
    // A box paints its glyphs unmarked: a deleted run inside a
    // delimiter drew exactly like plain x. A marked equation keeps the
    // per-run path, which inks and strikes each revised run.
    for (inner, mark) in [
        (
            r#"<m:r><w:del w:id="1" w:author="Ann"><m:t>x</m:t></w:del></m:r>"#,
            "del in run",
        ),
        (
            r#"<w:del w:id="1" w:author="Ann"><m:r><m:t>x</m:t></m:r></w:del>"#,
            "del around run",
        ),
        (
            r#"<w:ins w:id="1" w:author="Ann"><m:r><m:t>x</m:t></m:r></w:ins>"#,
            "ins around run",
        ),
    ] {
        for (open, close) in [
            ("<m:d><m:e>", "</m:e></m:d>"),
            (
                "<m:rad><m:radPr><m:degHide m:val=\"1\"/></m:radPr><m:deg/><m:e>",
                "</m:e></m:rad>",
            ),
            (
                "<m:f><m:num>",
                "</m:num><m:den><m:r><m:t>2</m:t></m:r></m:den></m:f>",
            ),
        ] {
            let runs = math_runs(&format!(
                "<w:p><m:oMath>{open}{inner}{close}</m:oMath></w:p>"
            ));
            assert!(
                runs.iter().all(|r| r.math.is_none()),
                "{mark} in {open}: a marked equation is not one box"
            );
            assert!(
                runs.iter().any(|r| r.rev && r.text.contains('\u{1D465}')),
                "{mark} in {open}: the revised x keeps its mark: {:?}",
                runs.iter().map(|r| (&r.text, r.rev)).collect::<Vec<_>>()
            );
        }
    }
    // Unmarked, the same delimiter is still one laid-out box.
    let runs =
        math_runs(r#"<w:p><m:oMath><m:d><m:e><m:r><m:t>x</m:t></m:r></m:e></m:d></m:oMath></w:p>"#);
    assert!(runs.iter().any(|r| r.math.is_some()), "plain box");
}

#[test]
fn omml_letters_are_cambria_math_italic() {
    // Word's PDF of math_matrix_tests (corpus 6c0ad6c8ef) embeds
    // CambriaMath and draws a matrix's a as U+1D44E: Word sets math
    // text in Cambria Math, letters in the mathematical italic block,
    // digits and operators upright.
    let runs = math_runs(r#"<w:p><m:oMath><m:r><m:t>ah+2=Z</m:t></m:r></m:oMath></w:p>"#);
    let run = runs.iter().find(|r| !r.text.is_empty()).expect("math run");
    assert_eq!(run.style.family, "Cambria Math");
    assert_eq!(run.text, "\u{1D44E}\u{210E}+2=\u{1D44D}");
    assert!(!run.style.italic, "the italic is in the code points");
}

#[test]
fn omml_letters_stay_ascii_italic_without_cambria_math() {
    // A face without the mathematical alphanumeric block drew nothing
    // for U+1D44E (sd_2750_borderbox lost every math letter); without
    // Cambria Math the letters keep their ASCII code points, italic.
    let _none = MathAlphanumerics::set(MathFace::Only(std::collections::HashSet::new()));
    let runs = math_runs(r#"<w:p><m:oMath><m:r><m:t>ab2</m:t></m:r></m:oMath></w:p>"#);
    let run = runs.iter().find(|r| !r.text.is_empty()).expect("math run");
    assert_eq!(run.text, "ab2");
    assert!(run.style.italic);
    assert!(run.math_ascii, "they still read back as math letters");
    drop(_none);
    let runs = math_runs(r#"<w:p><m:oMath><m:r><m:t>a</m:t></m:r></m:oMath></w:p>"#);
    assert_eq!(runs[0].text, "\u{1D44E}", "the flag is restored");
    assert!(!runs[0].math_ascii);
}

#[test]
fn omml_is_found_under_any_prefix() {
    // The namespace, not the producer's prefix, makes an equation.
    let math = "http://schemas.openxmlformats.org/officeDocument/2006/math";
    assert!(writes_omml(&format!(
        r#"<w:document xmlns:m="{math}"><m:oMath/></w:document>"#
    )));
    assert!(writes_omml(&format!(
        r#"<w:document xmlns:mth="{math}"><mth:oMathPara/></w:document>"#
    )));
    assert!(writes_omml(&format!(r#"<oMath xmlns="{math}"/>"#)));
    assert!(!writes_omml(
        r#"<w:document xmlns:m="urn:other"><m:oMath/></w:document>"#
    ));
    assert!(!writes_omml(&format!(r#"<w:document xmlns:m="{math}"/>"#)));
}

#[test]
fn omml_letters_a_subset_face_lacks_stay_ascii_italic() {
    // An embedded Cambria Math subset may draw some math letters and
    // not others: a run with a letter it lacks keeps ASCII italic
    // rather than losing that glyph.
    let _subset = MathAlphanumerics::only(['\u{1D44E}']);
    let runs = math_runs(
        r#"<w:p><m:oMath><m:r><m:t>ab</m:t></m:r><m:r><m:t>a</m:t></m:r></m:oMath></w:p>"#,
    );
    let texts: Vec<(&str, bool)> = runs
        .iter()
        .filter(|r| !r.text.is_empty())
        .map(|r| (r.text.as_str(), r.style.italic))
        .collect();
    assert_eq!(texts, [("ab", true), ("\u{1D44E}", false)]);
}

#[test]
fn omml_text_drops_the_inherited_hansi_face() {
    // A base whose hAnsi face differs (defaults can set one): the hAnsi
    // split would move U+1D44E off Cambria Math onto a face without
    // the block.
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
<w:body><w:p><m:oMath><m:r><m:t>a</m:t></m:r><m:d><m:e><m:r><m:t>b</m:t></m:r></m:e></m:d></m:oMath></w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom.descendants(root, Some(&W::p()))[0];
    let mut base = Defaults::word().run;
    base.family_hansi = Some("Georgia".into());
    let runs = collect_runs(&dom, para, &base, &ThemeFonts::default());
    for run in runs.iter().filter(|r| !r.text.is_empty()) {
        assert_eq!(run.style.family, "Cambria Math", "{:?}", run.text);
        assert_eq!(run.style.family_hansi, None, "{:?}", run.text);
    }
}

#[test]
fn omml_delimiters_take_the_script_placement() {
    // An m:d inside a superscript is raised with its contents. The
    // shift is baked into the box; the run stays on the baseline so
    // paint does not raise it a second time.
    let runs = math_runs(
        r#"<w:p><m:oMath><m:sSup><m:e><m:r><m:t>x</m:t></m:r></m:e><m:sup><m:d><m:e><m:r><m:t>n</m:t></m:r></m:e></m:d></m:sup></m:sSup></m:oMath></w:p>"#,
    );
    let sup = runs
        .iter()
        .find(|r| r.math.is_some())
        .expect("delimiter box");
    assert!(sup.style.vert == VertAlign::Baseline);
    let n = sup
        .math
        .as_ref()
        .unwrap()
        .glyphs
        .iter()
        .find(|g| g.text.contains('\u{1D45B}'))
        .expect("n");
    assert!(
        (n.y - 11.0 * 0.35).abs() < 0.05,
        "delimiter contents sit at +0.35 em, got {}",
        n.y
    );
    assert!(
        (n.size - 11.0 * 0.65).abs() < 0.05,
        "script size {}",
        n.size
    );
}

#[test]
fn omml_upright_and_plain_runs_keep_their_letters() {
    // m:sty p keeps letters upright in Cambria Math; m:nor is
    // ordinary text in the run's own font.
    let runs = math_runs(
        r#"<w:p><m:oMath><m:r><m:rPr><m:sty m:val="p"/></m:rPr><m:t>sin</m:t></m:r><m:r><m:rPr><m:nor/></m:rPr><m:t>if</m:t></m:r></m:oMath></w:p>"#,
    );
    let sin = runs.iter().find(|r| r.text == "sin").expect("sin");
    assert_eq!(sin.style.family, "Cambria Math");
    let plain = runs.iter().find(|r| r.text == "if").expect("if");
    assert_eq!(plain.style.family, Defaults::word().run.family);
}

#[test]
fn text_outside_math_keeps_its_font() {
    let runs = math_runs(r#"<w:p><w:r><w:t>ab</w:t></w:r></w:p>"#);
    assert_eq!(runs[0].text, "ab");
    assert_eq!(runs[0].style.family, Defaults::word().run.family);
}

#[test]
fn display_math_paragraph_takes_the_equation_alignment() {
    // Word centres an m:oMathPara on the page (math_matrix_tests: the
    // 2x2 matrix spans x 293-318 on a 612pt page) unless its m:jc
    // says left or right. A paragraph with text of its own keeps its
    // alignment.
    let align = |body: &str| {
        let xml = format!(
            r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
<w:body>{body}</w:body></w:document>"#
        );
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let root = dom.root(doc).expect("root");
        let para = dom.descendants(root, Some(&W::p()))[0];
        display_math_align(&dom, para)
    };
    let eq = r#"<m:oMath><m:r><m:t>x</m:t></m:r></m:oMath>"#;
    assert_eq!(
        align(&format!("<w:p><m:oMathPara>{eq}</m:oMathPara></w:p>")),
        Some(Align::Center)
    );
    assert_eq!(
        align(&format!(
            r#"<w:p><m:oMathPara><m:oMathParaPr><m:jc m:val="left"/></m:oMathParaPr>{eq}</m:oMathPara></w:p>"#
        )),
        Some(Align::Left)
    );
    assert_eq!(
        align(&format!(
            r#"<w:p><m:oMathPara><m:oMathParaPr><m:jc m:val="right"/></m:oMathParaPr>{eq}</m:oMathPara></w:p>"#
        )),
        Some(Align::Right)
    );
    assert_eq!(
        align(&format!(
            "<w:p><w:r><w:t>Let</w:t></w:r><m:oMathPara>{eq}</m:oMathPara></w:p>"
        )),
        None
    );
    assert_eq!(
        align(&format!("<w:p>{eq}</w:p>")),
        None,
        "inline math is not display"
    );
    // Deleted text is painted in a markup view: the paragraph is mixed.
    assert_eq!(
        align(&format!(
            r#"<w:p><w:del w:id="1" w:author="A"><w:r><w:delText>Let</w:delText></w:r></w:del><m:oMathPara>{eq}</m:oMathPara></w:p>"#
        )),
        None
    );
}

#[test]
fn w14_text_fill_without_color_uses_scheme_accent() {
    // Strict01 "Online Video" run has w14:textFill gradFill accent5
    // and no w:color. Flattening left black. Not w14:shadow (ITT-neg)
    // and not outline extra stroke.
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml">
<w:body><w:p>
  <w:r>
    <w:rPr>
      <w:sz w:val="40"/>
      <w14:textFill>
        <w14:gradFill>
          <w14:gsLst>
            <w14:gs w14:pos="0"><w14:schemeClr w14:val="accent5"/></w14:gs>
          </w14:gsLst>
        </w14:gradFill>
      </w14:textFill>
    </w:rPr>
    <w:t>Filled</w:t>
  </w:r>
</w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
    let run = runs
        .iter()
        .find(|r| r.text.contains("Filled"))
        .expect("run");
    let want = theme_slot_color("accent5").expect("accent5");
    assert!(
        (run.style.color[0] - want[0]).abs() < 0.02
            && (run.style.color[1] - want[1]).abs() < 0.02
            && (run.style.color[2] - want[2]).abs() < 0.02,
        "w14:textFill accent5 must paint, got {:?}",
        run.style.color
    );
}

#[test]
fn w14_text_fill_lummod_stays_unmodulated_after_mini_370() {
    // Strict01 Online Video first stop is accent5 lumMod=50000.
    // Applying RGB×0.5 (mini 370) was Word-shaped but ITT-neg:
    // Strict01 family −0.088 / NR mean −0.012 vs KEEP unmodulated
    // teal. Quartz matches the mid-stop. Not outline/shadow.
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml">
<w:body><w:p>
  <w:r>
    <w:rPr>
      <w:sz w:val="40"/>
      <w14:textFill>
        <w14:solidFill>
          <w14:schemeClr w14:val="accent5">
            <w14:lumMod w14:val="50000"/>
          </w14:schemeClr>
        </w14:solidFill>
      </w14:textFill>
    </w:rPr>
    <w:t>DimFill</w:t>
  </w:r>
</w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
    let run = runs
        .iter()
        .find(|r| r.text.contains("DimFill"))
        .expect("run");
    let want = theme_slot_color("accent5").expect("accent5");
    assert!(
        (run.style.color[0] - want[0]).abs() < 0.02
            && (run.style.color[1] - want[1]).abs() < 0.02
            && (run.style.color[2] - want[2]).abs() < 0.02,
        "mini 370 lumMod ITT-neg; keep unmodulated {want:?}, got {:?}",
        run.style.color
    );
}

#[test]
fn w14_text_outline_stays_fill_only_after_mini_371() {
    // Strict01 keyword: peach fill + accent2 outline. Fill+stroke
    // Tr=2 (mini 371) was Word-shaped but ITT-neg: Strict01 family
    // −0.043 / NR mean −0.006. Extra halo vs Quartz. Keep fill-only.
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml">
<w:body><w:p>
  <w:r>
    <w:rPr>
      <w:color w:val="F7CAAC"/>
      <w14:textOutline w14:w="11112" w14:cap="flat" w14:cmpd="sng" w14:algn="ctr">
        <w14:solidFill><w14:schemeClr w14:val="accent2"/></w14:solidFill>
        <w14:prstDash w14:val="solid"/>
      </w14:textOutline>
    </w:rPr>
    <w:t>Keyword</w:t>
  </w:r>
</w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
    let run = runs
        .iter()
        .find(|r| r.text.contains("Keyword"))
        .expect("run");
    let fill = parse_hex_color("F7CAAC").expect("fill");
    assert!(
        (run.style.color[0] - fill[0]).abs() < 0.02
            && (run.style.color[1] - fill[1]).abs() < 0.02
            && (run.style.color[2] - fill[2]).abs() < 0.02,
        "mini 371 outline ITT-neg; fill stays F7CAAC, got {:?}",
        run.style.color
    );
}

#[test]
fn diag_ln_stroke_keeps_accent1_skips_near_white() {
    // Strict01 connector bar: lt1 fill (skip) + accent1 ln (keep).
    let xml = r#"<?xml version="1.0"?>
<a:sp xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <a:spPr>
    <a:solidFill><a:schemeClr val="lt1"/></a:solidFill>
    <a:ln w="12700"><a:solidFill><a:schemeClr val="accent1"/></a:solidFill></a:ln>
  </a:spPr>
</a:sp>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    assert!(
        diag_solid_fill(&dom, root, &ThemeFonts::default()).is_some_and(is_near_white),
        "lt1 fill must parse (Word paints it when the bar is stroked)"
    );
    let (color, width) =
        diag_ln_stroke(&dom, root, &ThemeFonts::default()).expect("accent1 stroke");
    let want = theme_slot_color("accent1").expect("accent1");
    assert!(
        (color[0] - want[0]).abs() < 0.02
            && (color[1] - want[1]).abs() < 0.02
            && (color[2] - want[2]).abs() < 0.02,
        "connector ln must be accent1, got {color:?}"
    );
    assert!(
        (width - 1.0).abs() < 0.05,
        "ln w=12700 EMU is 1pt, got {width}"
    );
    let xml_white = r#"<?xml version="1.0"?>
<a:sp xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <a:spPr>
    <a:solidFill><a:schemeClr val="accent1"/></a:solidFill>
    <a:ln w="12700"><a:solidFill><a:schemeClr val="lt1"/></a:solidFill></a:ln>
  </a:spPr>
</a:sp>"#;
    let mut dom_w = Dom::new();
    let doc_w = dom_w.parse_xdocument(xml_white);
    let root_w = dom_w.root(doc_w).expect("root");
    assert!(
        diag_ln_stroke(&dom_w, root_w, &ThemeFonts::default()).is_none(),
        "lt1 roundRect stroke is extra halo; skip"
    );
}

#[test]
fn omml_d_draws_its_delimiters() {
    // Word's PDF of math_all_objects (corpus 970e9bbbcc) draws
    // "17. Delimiter: (𝑥 + 𝑦)" and "sin(𝑥)": m:d's begChr/endChr,
    // parentheses by default, sepChr (default |) between its m:e.
    // An empty val draws nothing on that side.
    let joined = |body: &str| -> String {
        math_runs(&format!("<w:p><m:oMath>{body}</m:oMath></w:p>"))
            .iter()
            .map(|r| r.text.as_str())
            .collect()
    };
    let x = r#"<m:e><m:r><m:t>x</m:t></m:r></m:e>"#;
    let y = r#"<m:e><m:r><m:t>y</m:t></m:r></m:e>"#;
    assert_eq!(joined(&format!("<m:d>{x}</m:d>")), "(\u{1D465})");
    assert_eq!(
        joined(&format!(
            r#"<m:d><m:dPr><m:begChr m:val="["/><m:endChr m:val="]"/></m:dPr>{x}</m:d>"#
        )),
        "[\u{1D465}]"
    );
    assert_eq!(
        joined(&format!("<m:d>{x}{y}</m:d>")),
        "(\u{1D465}|\u{1D466})"
    );
    assert_eq!(
        joined(&format!(
            r#"<m:d><m:dPr><m:begChr m:val=""/><m:sepChr m:val=","/></m:dPr>{x}{y}</m:d>"#
        )),
        "\u{1D465},\u{1D466})"
    );
    let runs = math_runs(&format!("<w:p><m:oMath><m:d>{x}</m:d></m:oMath></w:p>"));
    let open = runs.iter().find(|r| r.text.contains('(')).expect("(");
    assert_eq!(open.style.family, "Cambria Math");
    assert!(
        open.math.is_some(),
        "a delimiter is one box, not a flat run of characters"
    );
}

#[test]
fn omml_f_nobar_stacks_num_over_den() {
    // Strict01 binomial is m:f type=noBar. Linear n/k (mini 359)
    // was ITT-neg; Quartz stacks n over k with no bar. Not
    // oMathPara center.
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
<w:body><w:p>
<m:oMath>
  <m:f>
    <m:fPr><m:type m:val="noBar"/></m:fPr>
    <m:num><m:r><m:t>n</m:t></m:r></m:num>
    <m:den><m:r><m:t>k</m:t></m:r></m:den>
  </m:f>
</m:oMath>
</w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
    let joined: String = runs.iter().map(|r| r.text.as_str()).collect();
    assert_eq!(
        joined, "\u{1D45B}\u{1D458}",
        "noBar stays n then k; joined={joined:?}"
    );
    assert!(
        !joined.contains('/'),
        "mini 359 linear slash ITT-neg; joined={joined:?}"
    );
    let num = runs.iter().find(|r| r.text == "\u{1D45B}").expect("num");
    let den = runs.iter().find(|r| r.text == "\u{1D458}").expect("den");
    assert!(
        matches!(num.style.vert, VertAlign::StackNum),
        "noBar num must stack above"
    );
    assert!(
        matches!(den.style.vert, VertAlign::StackDen),
        "noBar den must stack below"
    );
}

#[test]
fn omml_f_nobar_stays_concatenated_after_mini_359() {
    // Strict01 binomial is m:f type=noBar. Linear n/k (mini 359)
    // was ITT-neg: Quartz stacks noBar, extra slash is leftover
    // ink. Keep flatten nk. Not oMathPara center.
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
<w:body><w:p>
<m:oMath>
  <m:f>
    <m:fPr><m:type m:val="noBar"/></m:fPr>
    <m:num><m:r><m:t>n</m:t></m:r></m:num>
    <m:den><m:r><m:t>k</m:t></m:r></m:den>
  </m:f>
</m:oMath>
</w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
    let joined: String = runs.iter().map(|r| r.text.as_str()).collect();
    assert_eq!(
        joined, "\u{1D45B}\u{1D458}",
        "mini 359 n/k ITT-neg; joined={joined:?}"
    );
    assert!(
        !joined.contains('/'),
        "must not emit linear slash; joined={joined:?}"
    );
}

#[test]
fn del_run_is_red_strikethrough() {
    // A Word lock: Word's own revision ink.
    REVISIONS.with(|r| r.set(RevisionStyle::Word));
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body><w:p>
  <w:del w:id="0"><w:r><w:rPr><w:color w:val="0F172A"/></w:rPr><w:delText>gone</w:delText></w:r></w:del>
</w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
    let run = runs.iter().find(|r| r.text.contains("gone")).expect("del");
    assert!(run.style.strike, "Word paints w:del as strike");
    assert!(
        (run.style.color[0] - 209.0 / 255.0).abs() < 0.02
            && (run.style.color[1] - 52.0 / 255.0).abs() < 0.02
            && (run.style.color[2] - 56.0 / 255.0).abs() < 0.02,
        "delText is Word #D13438, got {:?}",
        run.style.color
    );
}

#[test]
fn ins_run_is_green_underline() {
    // A Word lock: Word's own revision ink.
    REVISIONS.with(|r| r.set(RevisionStyle::Word));
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body><w:p>
  <w:ins w:id="1"><w:r><w:rPr><w:color w:val="0F172A"/></w:rPr><w:t>fresh</w:t></w:r></w:ins>
</w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
    let run = runs.iter().find(|r| r.text.contains("fresh")).expect("ins");
    assert!(run.style.underline, "Word paints w:ins as underline");
    assert!(
        (run.style.color[0] - 209.0 / 255.0).abs() < 0.02
            && (run.style.color[1] - 52.0 / 255.0).abs() < 0.02
            && (run.style.color[2] - 56.0 / 255.0).abs() < 0.02,
        "Word first-author ins is #D13438, got {:?}",
        run.style.color
    );
}

#[test]
fn preserved_spaces_are_not_squeezed() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body><w:p>
  <w:r><w:t xml:space="preserve">no backend required           </w:t></w:r>
</w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
    let joined: String = runs.iter().map(|r| r.text.as_str()).collect();
    assert_eq!(
        joined, "no backend required           ",
        "xml:space padding is kept as Word paints it, got {joined:?}"
    );
}

#[test]
fn ins_xml_space_padding_is_kept() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body><w:p>
  <w:ins w:id="1"><w:r><w:t xml:space="preserve">fresh           </w:t></w:r></w:ins>
</w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
    let joined: String = runs.iter().map(|r| r.text.as_str()).collect();
    assert_eq!(
        joined, "fresh           ",
        "soffice keeps ins xml:space padding, got {joined:?}"
    );
}

#[test]
fn body_multi_run_generator_xml_space_is_kept() {
    // Word paints Suggestion-mode pads (`Editing         `); the
    // collapse was a score lock (mini 401). fixtures_500: painting
    // preserved padding lifted 62 of 245 files.
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body><w:p>
  <w:r><w:t xml:space="preserve">Toggle between         </w:t></w:r>
  <w:r><w:t xml:space="preserve">Editing         </w:t></w:r>
  <w:r><w:t xml:space="preserve">and         </w:t></w:r>
  <w:r><w:t xml:space="preserve">Suggesting         </w:t></w:r>
  <w:r><w:t xml:space="preserve">via the toolbar dropdown.         </w:t></w:r>
</w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
    let joined: String = runs.iter().map(|r| r.text.as_str()).collect();
    assert!(
        joined.contains("Editing         and"),
        "every preserved pad space is kept, got {joined:?}"
    );
}

#[test]
fn body_hello_xml_space_padding_is_kept() {
    // Word paints all of a preserved run's padding.
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body><w:p>
  <w:r><w:t xml:space="preserve">Hello         </w:t></w:r>
</w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
    let joined: String = runs.iter().map(|r| r.text.as_str()).collect();
    assert_eq!(
        joined, "Hello         ",
        "Hello pad is kept, got {joined:?}"
    );
}

#[test]
fn pbdr_heading_xml_space_padding_is_kept() {
    // file_146 / iter2 16pt section heads (`1. What this is          `)
    // carry bottom pBdr E2E8F0 plus generator xml:space pads. Mini 401
    // collapsed *all* body pads (sample/eigenpal −6.8). Keep pads only
    // when the paragraph has a bottom pBdr — not Times-240 / footnotes.
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body><w:p>
  <w:pPr><w:pBdr>
    <w:bottom w:val="single" w:sz="3" w:space="4" w:color="E2E8F0"/>
  </w:pBdr></w:pPr>
  <w:r><w:rPr><w:sz w:val="32"/></w:rPr>
    <w:t xml:space="preserve">1. What this is          </w:t></w:r>
</w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
    let joined: String = runs.iter().map(|r| r.text.as_str()).collect();
    assert_eq!(
        joined, "1. What this is          ",
        "pBdr heading xml:space pads must be kept, got {joined:?}"
    );
}

#[test]
fn bom_only_run_stays_in_text_after_mini_521() {
    // Stripping potpourri/file_19 U+FEFF (Word-faithful) was mini 521
    // ITT-neg: NR 59.4772→59.4744, Cicero −0.203, file_19 +0.036.
    let xml = "<?xml version=\"1.0\"?>\
<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
<w:body><w:p>\
  <w:r><w:t>\u{feff}</w:t></w:r>\
  <w:r><w:t>Hello</w:t></w:r>\
</w:p></w:body></w:document>";
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
    let joined: String = runs.iter().map(|r| r.text.as_str()).collect();
    assert_eq!(
        joined, "\u{feff}Hello",
        "mini 521: keep U+FEFF in the run, got {joined:?}"
    );
}

#[test]
fn courier_body_xml_space_is_kept() {
    // Word keeps file_69's Courier pads (and wraps Serialises onto
    // page 2); the collapse was a score lock (mini 520).
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body><w:p>
  <w:r><w:rPr>
    <w:rFonts w:ascii="Courier New" w:hAnsi="Courier New"/>
  </w:rPr>
    <w:t xml:space="preserve">WYSIWYG         .docx         editor</w:t></w:r>
</w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
    let joined: String = runs.iter().map(|r| r.text.as_str()).collect();
    assert_eq!(
        joined, "WYSIWYG         .docx         editor",
        "Courier body xml:space pads are kept, got {joined:?}"
    );
}

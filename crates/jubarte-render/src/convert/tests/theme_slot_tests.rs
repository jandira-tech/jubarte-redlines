// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;

fn family_from_rfonts(attrs: &str, theme: &ThemeFonts) -> String {
    style_from_rfonts(attrs, theme).family
}

fn style_from_rfonts(attrs: &str, theme: &ThemeFonts) -> RunStyle {
    let xml = format!(
        r#"<?xml version="1.0"?>
            <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
            <w:rFonts {attrs}/>
            </w:document>"#
    );
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).expect("root");
    let fonts = first_named(&dom, root, "rFonts").expect("rFonts");
    let mut style = Defaults::word().run;
    apply_rfonts(&dom, fonts, &mut style, theme);
    style
}

fn theme_cambria_minor() -> ThemeFonts {
    ThemeFonts {
        major: Some("Calibri".into()),
        minor: Some("Cambria".into()),
        ..ThemeFonts::default()
    }
}

#[test]
fn a_right_to_left_line_paints_its_number_left_of_the_word() {
    let run = TextRun::new("صفحات(8-12) ", default_run_style());
    let texts: Vec<String> = bidi_visual_line(&[run])
        .into_iter()
        .map(|r| r.text)
        .collect();
    assert_eq!(texts, vec!["(", "8-12", "صفحات)"]);
}

#[test]
fn an_rtl_runs_numbers_read_right_to_left_across_a_hyphen() {
    // Word draws 00205272's rtl "صفحات(12-8)" as "(8-12)صفحات" with
    // "8" leftmost: a w:rtl run's digits are Arabic numbers, which a
    // hyphen does not join.
    let mut style = default_run_style();
    style.rtl = true;
    let texts: Vec<String> = bidi_visual_line(&[TextRun::new("صفحات(12-8)", style)])
        .into_iter()
        .map(|r| r.text)
        .collect();
    assert_eq!(texts, vec!["(", "8", "-", "12", "صفحات)"]);
}

#[test]
fn a_bidi_theme_slot_takes_the_complex_script_face() {
    // fixtures_500 0033735c: asciiTheme="minorBidi" with an empty
    // theme minor cs typeface. Word draws Arial (the Arab script
    // font for the default ar-SA bidi language), not minor latin.
    let mut theme = ThemeFonts {
        minor: Some("Calibri".into()),
        ..ThemeFonts::default()
    };
    theme
        .script_fonts
        .insert(("minor".into(), "Arab".into()), "Arial".into());
    assert_eq!(
        family_from_rfonts(r#"w:asciiTheme="minorBidi""#, &theme),
        "Arial"
    );
    theme.minor_cs = Some("Tahoma".into());
    assert_eq!(
        family_from_rfonts(r#"w:asciiTheme="minorBidi""#, &theme),
        "Tahoma",
        "a named cs face wins"
    );
}

#[test]
fn theme_minor_cambria_without_explicit_ascii_sets_family_cambria() {
    assert_eq!(
        family_from_rfonts(r#"w:asciiTheme="minorHAnsi""#, &theme_cambria_minor()),
        "Cambria"
    );
}

#[test]
fn explicit_ascii_beats_theme_slot() {
    assert_eq!(
        family_from_rfonts(
            r#"w:ascii="Calibri" w:asciiTheme="minorHAnsi""#,
            &theme_cambria_minor()
        ),
        "Calibri"
    );
}

#[test]
fn display_cache_plus_major_slot_uses_theme_major() {
    assert_eq!(
        family_from_rfonts(
            r#"w:ascii="Aptos Display" w:asciiTheme="majorHAnsi""#,
            &theme_cambria_minor()
        ),
        "Calibri"
    );
}

#[test]
fn hansi_next_to_an_ascii_slot_keeps_to_high_ansi() {
    let style = style_from_rfonts(
        r#"w:asciiTheme="minorHAnsi" w:hAnsi="Arial""#,
        &theme_cambria_minor(),
    );
    assert_eq!(style.family, "Cambria");
    assert_eq!(style.family_hansi.as_deref(), Some("Arial"));
}

fn theme_with_east_asia() -> ThemeFonts {
    ThemeFonts {
        major: Some("Calibri".into()),
        minor: Some("Cambria".into()),
        minor_ea: Some("Verdana".into()),
        major_ea: Some("Georgia".into()),
        minor_cs: Some("Times New Roman".into()),
        ..ThemeFonts::default()
    }
}

#[test]
fn east_asia_theme_slot_sets_family_ea() {
    let style = style_from_rfonts(
        r#"w:asciiTheme="minorHAnsi" w:eastAsiaTheme="minorEastAsia""#,
        &theme_with_east_asia(),
    );
    assert_eq!(style.family, "Cambria");
    assert_eq!(
        style.family_ea.as_deref(),
        Some("Verdana"),
        "eastAsiaTheme=minorEastAsia must not stay on the latin slot"
    );
}

#[test]
fn explicit_east_asia_beats_theme_slot() {
    let style = style_from_rfonts(
        r#"w:eastAsia="MS Mincho" w:eastAsiaTheme="minorEastAsia""#,
        &theme_with_east_asia(),
    );
    assert_eq!(style.family_ea.as_deref(), Some("MS Mincho"));
}

#[test]
fn hint_east_asia_leaves_latin_letters_on_the_latin_face() {
    // fixtures_500 00d2ca27: "Suppl 1." carries w:hint="eastAsia"; Word
    // sets it in Times New Roman. The hint only decides characters
    // either script may own (curly quotes, dashes, symbols).
    let mut style = Defaults::word().run;
    style.family = "Times New Roman".into();
    style.family_ea = Some("DengXian".into());
    style.hint = FontHint::EastAsia;
    assert_eq!(super::paint_family(&style, "Suppl 1."), "Times New Roman");
    assert_eq!(super::paint_family(&style, "\u{201C}"), "DengXian");
    assert_eq!(super::paint_family(&style, "你好"), "DengXian");
}

#[test]
fn hint_east_asia_is_recorded() {
    let style = style_from_rfonts(r#"w:hint="eastAsia""#, &theme_with_east_asia());
    assert_eq!(style.hint, FontHint::EastAsia);
}

fn wrap_texts(runs: &[TextRun], width: f32) -> Vec<String> {
    wrap_runs(fonts(), runs, width, width, false)
        .iter()
        .map(|line| line.iter().map(|r| r.text.as_str()).collect())
        .collect()
}

fn body_width(text: &str) -> f32 {
    let style = Defaults::word().run;
    let fid = fonts().resolve(&style.family, false, false);
    fonts().get(fid).width_pt(text, style.layout_size())
}

#[test]
fn a_line_may_break_after_a_hyphen() {
    // fixtures_500 014babb2: Word ends line 1 on "sham-" and starts
    // line 2 with "vaccinated"; one token pushed the whole word down.
    let style = Defaults::word().run;
    let runs = [TextRun::new("aaaa sham-vaccinated", style)];
    let lines = wrap_texts(&runs, body_width("aaaa sham-") + 1.0);
    assert_eq!(lines, ["aaaa sham-", "vaccinated"]);
}

#[test]
fn a_run_ending_on_a_hyphen_may_end_the_line() {
    // b6dd0b6ae8: "-Karaman-" closes a run and "Kepenekci," opens the
    // next; Word ends the line on the hyphen, as inside one run.
    let style = Defaults::word().run;
    let runs = [
        TextRun::new("aaaa sham-", style.clone()),
        TextRun::new("vaccinated", style),
    ];
    let lines = wrap_texts(&runs, body_width("aaaa sham-") + 1.0);
    assert_eq!(lines, ["aaaa sham-", "vaccinated"]);
}

#[test]
fn a_hyphen_holds_its_word_under_compressed_punctuation() {
    // 496e2984f7 (compressPunctuation, no compatibility mode): Word
    // moves "gastro-oesophageal" whole, in one run or cut after the
    // hyphen.
    let mut style = Defaults::word().run;
    style.punct_compress = true;
    let one = [TextRun::new("aaaa sham-vaccinated", style.clone())];
    let cut = [
        TextRun::new("aaaa sham-", style.clone()),
        TextRun::new("vaccinated", style),
    ];
    for runs in [&one[..], &cut[..]] {
        let lines = wrap_texts(runs, body_width("aaaa sham-") + 1.0);
        assert_eq!(lines.last().map(String::as_str), Some("sham-vaccinated"));
    }
}

#[test]
fn closing_cjk_punctuation_hangs_past_the_right_edge() {
    // fixtures_500 0022e7ab: Word keeps "…届く予定です。" on one line with
    // "。" past the margin (565.3 vs 559.3). Word's oracle hangs 、。，．」）
    // but wraps ！, ー and kana to the next line with the character before.
    let style = Defaults::word().run;
    let ink = |t: &str| {
        fonts()
            .get(ink_face(fonts(), &style, t))
            .width_pt(t, style.layout_size())
    };
    let width = ink("あああ予定です") + 1.0;
    let hung = [TextRun::new("あああ予定です。", style.clone())];
    assert_eq!(wrap_texts(&hung, width), ["あああ予定です。"]);
    let bang = [TextRun::new("あああ予定です！", style.clone())];
    assert_eq!(wrap_texts(&bang, width), ["あああ予定で", "す！"]);
}

#[test]
fn a_run_boundary_inside_a_word_is_not_a_break() {
    // `birds` + bold `.`: Word never starts a line with the period.
    let plain = Defaults::word().run;
    let mut bold = plain.clone();
    bold.bold = true;
    let runs = [TextRun::new("aaaa bbbb", plain), TextRun::new(".", bold)];
    let lines = wrap_texts(&runs, body_width("aaaa bbbb") + 0.5);
    assert_eq!(lines, ["aaaa ", "bbbb."]);
}

#[test]
fn runs_with_different_script_fonts_do_not_merge() {
    // #111: "AB " (eastAsia SimSun) and "你好" (eastAsia MS Gothic)
    // share the Latin family; merging kept SimSun for 你好.
    let mut a = Defaults::word().run;
    a.family_ea = Some("SimSun".into());
    let mut b = a.clone();
    b.family_ea = Some("MS Gothic".into());
    let runs = vec![TextRun::new("AB ", a.clone()), TextRun::new("你好", b)];
    let lines = wrap_runs(fonts(), &runs, 400.0, 400.0, false);
    assert_eq!(lines.len(), 1);
    assert_eq!(
        lines[0].len(),
        2,
        "different eastAsia faces stay separate runs"
    );
    let mut c = a.clone();
    c.hint = FontHint::EastAsia;
    let lines = wrap_runs(
        fonts(),
        &[TextRun::new("A ", a.clone()), TextRun::new("B", c)],
        400.0,
        400.0,
        false,
    );
    assert_eq!(lines[0].len(), 2, "a w:hint boundary stays a run boundary");
    let mut d = a;
    d.family_cs = Some("Arial".into());
    let lines = wrap_runs(
        fonts(),
        &[
            TextRun::new("A ", Defaults::word().run),
            TextRun::new("B", d),
        ],
        400.0,
        400.0,
        false,
    );
    assert_eq!(lines[0].len(), 2, "different cs faces stay separate runs");
}

#[test]
fn a_no_break_space_holds_its_room_and_its_word() {
    // Word 16 probe d3a (d328fa3674): "tunách\u{a0}" ending a cell
    // wraps where "tunách " fits, the plain space hanging past the
    // edge; and the no-break space never opens a line.
    let style = Defaults::word().run;
    let width = body_width("aaaa bbbb") + 0.5;
    let hung = [TextRun::new("aaaa bbbb ", style.clone())];
    assert_eq!(wrap_texts(&hung, width), ["aaaa bbbb "]);
    let held = [TextRun::new("aaaa bbbb\u{a0}", style.clone())];
    assert_eq!(wrap_texts(&held, width), ["aaaa ", "bbbb\u{a0}"]);
    let joined = [TextRun::new("aaaa bbbb\u{a0}cc", style)];
    assert_eq!(
        wrap_texts(&joined, body_width("aaaa bbbb ") + 0.5),
        ["aaaa ", "bbbb\u{a0}cc"]
    );
}

#[test]
fn empty_theme_typefaces_are_absent_not_empty_families() {
    // Word writes `<a:ea typeface=""/>` / `<a:cs typeface=""/>` in its
    // stock themes; an empty minor latin is legal too. None of them may
    // become a "" family (which resolves as an unknown face).
    let xml = r#"<?xml version="1.0"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <a:themeElements><a:fontScheme name="Office">
    <a:majorFont><a:latin typeface=" "/><a:ea typeface=""/><a:cs typeface=""/></a:majorFont>
    <a:minorFont><a:latin typeface=""/><a:ea typeface=""/><a:cs typeface=""/></a:minorFont>
  </a:fontScheme></a:themeElements>
</a:theme>"#;
    let theme = parse_theme_xml(xml);
    assert_eq!(theme.major, None);
    assert_eq!(theme.minor, None);
    assert_eq!(theme.minor_ea, None);
    assert_eq!(theme.minor_cs, None);
    let style = style_from_rfonts(
        r#"w:asciiTheme="minorHAnsi" w:hAnsiTheme="minorHAnsi" w:eastAsiaTheme="minorEastAsia""#,
        &theme,
    );
    assert_eq!(
        style.family,
        Defaults::word().run.family,
        "an empty minor slot keeps the inherited family"
    );
    assert_eq!(
        style.family_ea, None,
        "an empty ea slot sets no East Asian face"
    );
}

#[test]
fn load_theme_reads_minor_east_asia_typeface() {
    let xml = r#"<?xml version="1.0"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <a:themeElements><a:fontScheme name="Office">
    <a:majorFont><a:latin typeface="Calibri"/><a:ea typeface="Georgia"/></a:majorFont>
    <a:minorFont><a:latin typeface="Cambria"/><a:ea typeface="Verdana"/><a:cs typeface="Times New Roman"/></a:minorFont>
  </a:fontScheme></a:themeElements>
</a:theme>"#;
    let theme = parse_theme_xml(xml);
    assert_eq!(theme.minor.as_deref(), Some("Cambria"));
    assert_eq!(
        theme.minor_ea.as_deref(),
        Some("Verdana"),
        "a:ea on minorFont is xml 3.2 ckpt 2"
    );
    assert_eq!(theme.major_ea.as_deref(), Some("Georgia"));
    assert_eq!(theme.minor_cs.as_deref(), Some("Times New Roman"));
}

#[test]
fn load_theme_reads_script_font_jpan() {
    let xml = r#"<?xml version="1.0"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <a:themeElements><a:fontScheme name="Office">
    <a:minorFont>
      <a:latin typeface="Cambria"/>
      <a:ea typeface="Verdana"/>
      <a:font script="Jpan" typeface="Yu Mincho"/>
    </a:minorFont>
  </a:fontScheme></a:themeElements>
</a:theme>"#;
    let theme = parse_theme_xml(xml);
    assert_eq!(
        theme
            .script_fonts
            .get(&("minor".into(), "Jpan".into()))
            .map(String::as_str),
        Some("Yu Mincho"),
        "a:font script=Jpan is xml 3.2 ckpt 3; map={:?}",
        theme.script_fonts
    );
}

#[test]
fn load_theme_reads_fmt_scheme_fill_and_ln() {
    // xml leftover: fmtScheme fillStyleLst / lnStyleLst. phClr stays
    // None so fillRef/lnRef schemeClr still substitutes.
    let xml = r#"<?xml version="1.0"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <a:themeElements><a:fmtScheme name="Office">
    <a:fillStyleLst>
      <a:solidFill><a:srgbClr val="CC0000"/></a:solidFill>
      <a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
      <a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
    </a:fillStyleLst>
    <a:lnStyleLst>
      <a:ln w="25400"><a:solidFill><a:srgbClr val="00AA00"/></a:solidFill></a:ln>
      <a:ln w="12700"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln>
    </a:lnStyleLst>
  </a:fmtScheme></a:themeElements>
</a:theme>"#;
    let theme = parse_theme_xml(xml);
    assert_eq!(
        theme.fill_styles[0],
        parse_hex_color("CC0000"),
        "fillStyleLst[0] solid CC0000"
    );
    assert_eq!(
        theme.fill_styles[1], None,
        "phClr must not freeze fillRef substitution; {:?}",
        theme.fill_styles
    );
    assert_eq!(theme.ln_widths[0], Some(2.0), "25400 EMU = 2pt");
    assert_eq!(
        theme.ln_colors[0],
        parse_hex_color("00AA00"),
        "lnStyleLst[0] solid 00AA00"
    );
    assert_eq!(theme.ln_widths[1], Some(1.0));
    assert_eq!(theme.ln_colors[1], None);
}

fn style_from_rpr(inner: &str, theme: &ThemeFonts) -> RunStyle {
    let xml = format!(
        r#"<?xml version="1.0"?>
            <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
            <w:rPr>{inner}</w:rPr>
            </w:document>"#
    );
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).expect("root");
    let rpr = first_named(&dom, root, "rPr").expect("rPr");
    let mut style = Defaults::word().run;
    apply_rpr(&dom, rpr, &mut style, theme);
    style
}

/// Apply `layers` (rPr inner XML) in cascade order, like docDefaults →
/// style chain → run.
fn style_from_layers(layers: &[&str], theme: &ThemeFonts) -> RunStyle {
    let mut style = Defaults::word().run;
    for inner in layers {
        let xml = format!(
            r#"<w:rPr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">{inner}</w:rPr>"#
        );
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let rpr = dom.root(doc).expect("rPr");
        apply_rpr(&dom, rpr, &mut style, theme);
    }
    style
}

fn jpan_theme() -> ThemeFonts {
    let mut theme = theme_with_east_asia();
    for (bucket, face) in [("minor", "Yu Mincho"), ("major", "Yu Gothic")] {
        theme
            .script_fonts
            .insert((bucket.into(), "Jpan".into()), face.into());
    }
    theme
}

#[test]
fn inherited_explicit_east_asia_survives_a_run_language() {
    // #112: the style names MS Mincho; the run sets only Latin fonts
    // and ja-JP. No East Asian slot at the run, so the face stays.
    let style = style_from_layers(
        &[
            r#"<w:rFonts w:eastAsia="MS Mincho"/>"#,
            r#"<w:rFonts w:ascii="Calibri"/><w:lang w:eastAsia="ja-JP"/>"#,
        ],
        &jpan_theme(),
    );
    assert_eq!(style.family_ea.as_deref(), Some("MS Mincho"));
}

#[test]
fn script_font_follows_the_inherited_theme_slot_and_lang_only_runs() {
    // #112: majorEastAsia inherited from the style picks the major Jpan
    // face, and a run with only w:lang still resolves it.
    let style = style_from_layers(
        &[
            r#"<w:rFonts w:eastAsiaTheme="majorEastAsia"/>"#,
            r#"<w:lang w:eastAsia="ja-JP"/>"#,
        ],
        &jpan_theme(),
    );
    assert_eq!(style.family_ea.as_deref(), Some("Yu Gothic"));
}

#[test]
fn no_east_asia_theme_slot_selects_no_script_font() {
    let style = style_from_layers(&[r#"<w:lang w:eastAsia="ja-JP"/>"#], &jpan_theme());
    assert_eq!(style.family_ea, None);
}

#[test]
fn lang_without_east_asia_keeps_the_inherited_east_asian_language() {
    let style = style_from_layers(
        &[
            r#"<w:rFonts w:eastAsiaTheme="minorEastAsia"/><w:lang w:eastAsia="ja-JP"/>"#,
            r#"<w:lang w:val="en-US"/>"#,
        ],
        &jpan_theme(),
    );
    assert_eq!(style.lang_ea.as_deref(), Some("ja-JP"));
    assert_eq!(style.family_ea.as_deref(), Some("Yu Mincho"));
}

#[test]
fn ja_lang_uses_jpan_script_font_over_generic_ea() {
    let mut theme = theme_with_east_asia();
    theme
        .script_fonts
        .insert(("minor".into(), "Jpan".into()), "Yu Mincho".into());
    let style = style_from_rpr(
        r#"<w:rFonts w:eastAsiaTheme="minorEastAsia" w:hint="eastAsia"/>
               <w:lang w:eastAsia="ja-JP"/>"#,
        &theme,
    );
    assert_eq!(
        style.family_ea.as_deref(),
        Some("Yu Mincho"),
        "ja-JP must pick a:font script=Jpan, not generic a:ea; family_ea={:?}",
        style.family_ea
    );
    assert_eq!(style.lang_ea.as_deref(), Some("ja-JP"));
}

#[test]
fn explicit_east_asia_beats_script_font() {
    let mut theme = theme_with_east_asia();
    theme
        .script_fonts
        .insert(("minor".into(), "Jpan".into()), "Yu Mincho".into());
    let style = style_from_rpr(
        r#"<w:rFonts w:eastAsia="MS Mincho" w:eastAsiaTheme="minorEastAsia"/>
               <w:lang w:eastAsia="ja-JP"/>"#,
        &theme,
    );
    assert_eq!(style.family_ea.as_deref(), Some("MS Mincho"));
}

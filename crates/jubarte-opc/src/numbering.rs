// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! List numbering shared by every layer above the package floor.
//!
//! `append_numbering` grows a numbering part (`w:abstractNum` / `w:num`);
//! the label family renders a `w:numFmt` counter (`lowerRoman`,
//! `japaneseCounting`, ...) exactly as Word prints it, which the layout,
//! the agent view and the doc-level fields all need without reaching into
//! the renderer.

use crate::opc::{PartFs, relative_rel_target};

const RELS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const W_NS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";

/// The part name beside `main` (`word/document.xml` -> `word/<name>`).
#[doc(hidden)]
pub fn sibling(main: &str, name: &str) -> String {
    match main.rsplit_once('/') {
        Some((dir, _)) => format!("{dir}/{name}"),
        None => name.to_string(),
    }
}

/// The part `main` relates to with the relationship type ending in `kind`.
#[doc(hidden)]
pub fn related(package: &PartFs, main: &str, kind: &str) -> Option<String> {
    let rels = package.read_rels_for(main)?;
    let rel = rels
        .items
        .iter()
        .find(|r| r.rel_type.ends_with(kind) && r.target_mode.as_deref() != Some("External"))?;
    let part = package.resolve_rel_target(main, &rel.target);
    package.part_bytes(&part).is_some().then_some(part)
}

/// The largest value of attribute `name` on elements `element` in `xml`.
#[doc(hidden)]
pub fn max_attribute(xml: &str, element: &str, name: &str) -> Option<i64> {
    attribute_values(xml, element, name).into_iter().max()
}

/// Every numeric `name` attribute of `element` in `xml`.
#[doc(hidden)]
pub fn attribute_values(xml: &str, element: &str, name: &str) -> Vec<i64> {
    let mut values = Vec::new();
    let mut rest = xml;
    let open = format!("<{element} ");
    let key = format!("{name}=\"");
    while let Some(at) = rest.find(&open) {
        let tag_end = rest[at..].find('>').map_or(rest.len(), |i| at + i);
        let tag = &rest[at..tag_end];
        if let Some(value) = tag
            .find(&key)
            .map(|i| &tag[i + key.len()..])
            .and_then(|v| v[..v.find('"')?].parse::<i64>().ok())
        {
            values.push(value);
        }
        rest = &rest[tag_end..];
    }
    values
}

/// The numbering part `main` relates to, made empty when there is none.
#[doc(hidden)]
pub fn numbering_part(package: &mut PartFs, main: &str) -> String {
    related(package, main, "/numbering").unwrap_or_else(|| {
        let part = sibling(main, "numbering.xml");
        package.add_document_relationship(
            main,
            &format!("{RELS}/numbering"),
            &relative_rel_target(main, &part),
        );
        package.add_content_type_override(
            &format!("/{part}"),
            "application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml",
        );
        package.set_part(
            &part,
            format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
                 <w:numbering xmlns:w=\"{W_NS}\"></w:numbering>"
            )
            .into_bytes(),
        );
        part
    })
}

/// Appends `w:abstractNum`s and `w:num`s to `main`'s numbering part, making
/// one if needed. `build` gets the first free abstract and num ids and
/// returns the `(abstracts, nums)` markup; returns those first ids.
#[doc(hidden)]
pub fn append_numbering(
    package: &mut PartFs,
    main: &str,
    build: impl FnOnce(u32, u32) -> (String, String),
) -> Option<(u32, u32)> {
    let part = numbering_part(package, main);
    let existing = package.part_string(&part)?;
    let next = |value: Option<i64>| u32::try_from(value.unwrap_or(0).max(0) + 1).unwrap_or(1);
    let first_abstract = next(max_attribute(&existing, "w:abstractNum", "w:abstractNumId"));
    let first_num = next(max_attribute(&existing, "w:num", "w:numId"));
    let (abstracts, nums) = build(first_abstract, first_num);
    // Every w:abstractNum precedes every w:num.
    let abstract_at = [
        "<w:num ",
        "<w:num>",
        "<w:numIdMacAtCleanup",
        "</w:numbering>",
    ]
    .iter()
    .filter_map(|tag| existing.find(tag))
    .min();
    let num_at = ["<w:numIdMacAtCleanup", "</w:numbering>"]
        .iter()
        .filter_map(|tag| existing.find(tag))
        .min();
    if let (Some(a), Some(n)) = (abstract_at, num_at) {
        let updated = format!(
            "{}{abstracts}{}{nums}{}",
            &existing[..a],
            &existing[a..n],
            &existing[n..]
        );
        package.set_part(&part, updated.into_bytes());
    }
    Some((first_abstract, first_num))
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[doc(hidden)]
pub enum NumFmt {
    Decimal,
    DecimalZero,
    CardinalText,
    Ordinal,
    OrdinalText,
    Hex,
    Chicago,
    IdeographDigital,
    IdeographTraditional,
    IdeographZodiac,
    IdeographZodiacTraditional,
    IdeographLegalTraditional,
    IdeographEnclosedCircle,
    ChineseCounting,
    ChineseCountingThousand,
    TaiwaneseCountingThousand,
    JapaneseCounting,
    Aiueo,
    Iroha,
    DecimalFullWidth,
    Ganada,
    Chosung,
    DecimalEnclosedCircle,
    DecimalEnclosedParen,
    DecimalEnclosedFullstop,
    Hebrew1,
    Hebrew2,
    ArabicAlpha,
    ArabicAbjad,
    RussianLower,
    RussianUpper,
    ThaiNumbers,
    ThaiLetters,
    ThaiCounting,
    HindiNumbers,
    HindiVowels,
    HindiConsonants,
    HindiCounting,
    KoreanCounting,
    KoreanDigital,
    KoreanDigital2,
    KoreanLegal,
    LowerLetter,
    UpperLetter,
    LowerRoman,
    UpperRoman,
    Bullet,
}

/// Whether a level text names a level deeper than `ilvl` (`%3` on level 1,
/// 0-based). Word then prints no label and no list indent, though the
/// paragraph still counts (probe lvl_undef_1010).
#[doc(hidden)]
pub fn names_deeper_level(text: &str, ilvl: u32) -> bool {
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '%'
            && let Some(k) = chars.clone().next().and_then(|d| d.to_digit(10))
            && (1..=9).contains(&k)
            && k > ilvl + 1
        {
            return true;
        }
    }
    false
}

#[doc(hidden)]
pub fn parse_num_fmt(val: &str) -> NumFmt {
    match val {
        "lowerLetter" => NumFmt::LowerLetter,
        "upperLetter" => NumFmt::UpperLetter,
        "lowerRoman" => NumFmt::LowerRoman,
        "upperRoman" => NumFmt::UpperRoman,
        "bullet" => NumFmt::Bullet,
        "decimalZero" => NumFmt::DecimalZero,
        "cardinalText" => NumFmt::CardinalText,
        "ordinal" => NumFmt::Ordinal,
        "ordinalText" => NumFmt::OrdinalText,
        "hex" => NumFmt::Hex,
        "chicago" => NumFmt::Chicago,
        "ideographDigital" => NumFmt::IdeographDigital,
        "ideographTraditional" => NumFmt::IdeographTraditional,
        "ideographZodiac" => NumFmt::IdeographZodiac,
        "ideographZodiacTraditional" => NumFmt::IdeographZodiacTraditional,
        "ideographLegalTraditional" => NumFmt::IdeographLegalTraditional,
        "ideographEnclosedCircle" => NumFmt::IdeographEnclosedCircle,
        "japaneseCounting" => NumFmt::JapaneseCounting,
        "chineseCounting" | "taiwaneseCounting" => NumFmt::ChineseCounting,
        "chineseCountingThousand" => NumFmt::ChineseCountingThousand,
        "taiwaneseCountingThousand" => NumFmt::TaiwaneseCountingThousand,
        "aiueo" => NumFmt::Aiueo,
        "iroha" => NumFmt::Iroha,
        "decimalFullWidth" => NumFmt::DecimalFullWidth,
        "ganada" => NumFmt::Ganada,
        "chosung" => NumFmt::Chosung,
        "decimalEnclosedCircle" => NumFmt::DecimalEnclosedCircle,
        "decimalEnclosedParen" => NumFmt::DecimalEnclosedParen,
        "decimalEnclosedFullstop" => NumFmt::DecimalEnclosedFullstop,
        "hebrew1" => NumFmt::Hebrew1,
        "hebrew2" => NumFmt::Hebrew2,
        "arabicAlpha" => NumFmt::ArabicAlpha,
        "arabicAbjad" => NumFmt::ArabicAbjad,
        "russianLower" => NumFmt::RussianLower,
        "russianUpper" => NumFmt::RussianUpper,
        "thaiNumbers" => NumFmt::ThaiNumbers,
        "thaiLetters" => NumFmt::ThaiLetters,
        "thaiCounting" => NumFmt::ThaiCounting,
        "hindiNumbers" => NumFmt::HindiNumbers,
        "hindiVowels" => NumFmt::HindiVowels,
        "hindiConsonants" => NumFmt::HindiConsonants,
        "hindiCounting" => NumFmt::HindiCounting,
        "koreanCounting" => NumFmt::KoreanCounting,
        "koreanDigital" => NumFmt::KoreanDigital,
        "koreanDigital2" => NumFmt::KoreanDigital2,
        "koreanLegal" => NumFmt::KoreanLegal,
        _ => NumFmt::Decimal,
    }
}

/// A list counter in a `w:numFmt` (`lowerLetter`, `upperRoman`, …), as the
/// layout prints it; for the agent view's id lines.
#[doc(hidden)]
pub fn list_number(num_fmt: &str, n: u32) -> String {
    format_num(parse_num_fmt(num_fmt), n)
}

#[doc(hidden)]
pub fn format_num(fmt: NumFmt, n: u32) -> String {
    match fmt {
        NumFmt::Decimal => n.to_string(),
        NumFmt::DecimalZero => format!("{n:02}"),
        NumFmt::CardinalText => cardinal_label(n),
        NumFmt::Ordinal => ordinal_label(n),
        NumFmt::OrdinalText => ordinal_text_label(n),
        NumFmt::Hex => format!("{n:X}"),
        NumFmt::Chicago => chicago_label(n),
        NumFmt::IdeographDigital => ideograph_digital_label(n),
        NumFmt::IdeographTraditional => cycle_cjk(&IDEOGRAPH_STEMS, n),
        NumFmt::IdeographZodiac => cycle_cjk(&IDEOGRAPH_BRANCHES, n),
        NumFmt::IdeographZodiacTraditional => ideograph_zodiac_traditional_label(n),
        NumFmt::IdeographLegalTraditional => ideograph_legal_traditional_label(n),
        NumFmt::IdeographEnclosedCircle => ideograph_enclosed_circle_label(n),
        NumFmt::JapaneseCounting => japanese_counting_label(n),
        NumFmt::ChineseCounting => chinese_counting_label(n),
        NumFmt::ChineseCountingThousand => chinese_counting_thousand(n, '〇', '万'),
        NumFmt::TaiwaneseCountingThousand => chinese_counting_thousand(n, '零', '萬'),
        NumFmt::Aiueo => cycle_cjk(&AIUEO, n),
        NumFmt::Iroha => cycle_cjk(&IROHA, n),
        NumFmt::DecimalFullWidth => decimal_fullwidth_label(n),
        NumFmt::Ganada => cycle_cjk(&GANADA, n),
        NumFmt::Chosung => cycle_cjk(&CHOSUNG, n),
        NumFmt::DecimalEnclosedCircle => enclosed_decimal_label(n, &DECIMAL_ENCLOSED_CIRCLE),
        NumFmt::DecimalEnclosedParen => enclosed_decimal_label(n, &DECIMAL_ENCLOSED_PAREN),
        NumFmt::DecimalEnclosedFullstop => enclosed_decimal_label(n, &DECIMAL_ENCLOSED_FULLSTOP),
        NumFmt::Hebrew1 => hebrew1_label(n),
        NumFmt::Hebrew2 => cycle_cjk(&HEBREW2, n),
        NumFmt::ArabicAlpha => cycle_cjk(&ARABIC_ALPHA, n),
        NumFmt::ArabicAbjad => cycle_cjk(&ARABIC_ABJAD, n),
        NumFmt::RussianLower => cycle_cjk(&RUSSIAN_LOWER, n),
        NumFmt::RussianUpper => cycle_cjk(&RUSSIAN_UPPER, n),
        NumFmt::ThaiNumbers => thai_numbers_label(n),
        NumFmt::ThaiLetters => cycle_cjk(&THAI_LETTERS, n),
        NumFmt::ThaiCounting => thai_counting_label(n),
        NumFmt::HindiNumbers => hindi_numbers_label(n),
        NumFmt::HindiVowels => cycle_cjk(&HINDI_VOWELS, n),
        NumFmt::HindiConsonants => cycle_cjk(&HINDI_CONSONANTS, n),
        NumFmt::HindiCounting => hindi_counting_label(n),
        NumFmt::KoreanCounting => korean_counting_label(n),
        NumFmt::KoreanDigital => korean_digital_label(n),
        NumFmt::KoreanDigital2 => korean_digital2_label(n),
        NumFmt::KoreanLegal => korean_legal_label(n),
        NumFmt::LowerLetter => alpha_label(n, false),
        NumFmt::UpperLetter => alpha_label(n, true),
        NumFmt::LowerRoman => roman_label(n, false),
        NumFmt::UpperRoman => roman_label(n, true),
        NumFmt::Bullet => "•".into(),
    }
}

#[doc(hidden)]
pub fn cardinal_label(n: u32) -> String {
    const ONES: [&str; 20] = [
        "Zero",
        "One",
        "Two",
        "Three",
        "Four",
        "Five",
        "Six",
        "Seven",
        "Eight",
        "Nine",
        "Ten",
        "Eleven",
        "Twelve",
        "Thirteen",
        "Fourteen",
        "Fifteen",
        "Sixteen",
        "Seventeen",
        "Eighteen",
        "Nineteen",
    ];
    const TENS: [&str; 10] = [
        "", "", "Twenty", "Thirty", "Forty", "Fifty", "Sixty", "Seventy", "Eighty", "Ninety",
    ];
    if (n as usize) < ONES.len() {
        return ONES[n as usize].into();
    }
    if n < 100 {
        let ten = TENS[(n / 10) as usize];
        let one = n % 10;
        if one == 0 {
            return ten.into();
        }
        let ones = ONES[one as usize];
        return format!("{ten}-{ones}");
    }
    n.to_string()
}

#[doc(hidden)]
pub fn ordinal_label(n: u32) -> String {
    format!("{n}{}", ordinal_suffix(n))
}

#[doc(hidden)]
pub fn ordinal_suffix(n: u32) -> &'static str {
    if matches!(n % 100, 11..=13) {
        "th"
    } else {
        match n % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        }
    }
}

#[doc(hidden)]
pub fn ordinal_text_label(n: u32) -> String {
    match n {
        1 => "First".into(),
        2 => "Second".into(),
        3 => "Third".into(),
        4 => "Fourth".into(),
        5 => "Fifth".into(),
        6 => "Sixth".into(),
        7 => "Seventh".into(),
        8 => "Eighth".into(),
        9 => "Ninth".into(),
        10 => "Tenth".into(),
        11 => "Eleventh".into(),
        12 => "Twelfth".into(),
        13 => "Thirteenth".into(),
        14 => "Fourteenth".into(),
        15 => "Fifteenth".into(),
        16 => "Sixteenth".into(),
        17 => "Seventeenth".into(),
        18 => "Eighteenth".into(),
        19 => "Nineteenth".into(),
        20 => "Twentieth".into(),
        30 => "Thirtieth".into(),
        40 => "Fortieth".into(),
        50 => "Fiftieth".into(),
        60 => "Sixtieth".into(),
        70 => "Seventieth".into(),
        80 => "Eightieth".into(),
        90 => "Ninetieth".into(),
        n if n < 100 => {
            let card = cardinal_label(n);
            match card.split_once('-') {
                Some((ten, one)) => format!("{ten}-{}", ordinal_text_ones(one)),
                None => format!("{card}th"),
            }
        }
        n => format!("{n}th"),
    }
}

#[doc(hidden)]
pub fn ordinal_text_ones(one: &str) -> String {
    match one {
        "One" => "First".into(),
        "Two" => "Second".into(),
        "Three" => "Third".into(),
        "Five" => "Fifth".into(),
        "Eight" => "Eighth".into(),
        "Nine" => "Ninth".into(),
        other => format!("{other}th"),
    }
}

/// No Word document carries a Chicago sequence past a handful of cycles.
/// A larger start renders at this cap instead of allocating one mark per
/// cycle (`u32::MAX` would ask for about a gigabyte of label bytes).
const MAX_CHICAGO_MARKS: usize = 256;

#[doc(hidden)]
pub fn chicago_label(n: u32) -> String {
    const MARKS: [&str; 4] = ["*", "†", "‡", "§"];
    let i = n.saturating_sub(1) as usize;
    MARKS[i % 4].repeat((i / 4 + 1).min(MAX_CHICAGO_MARKS))
}

#[doc(hidden)]
pub fn ideograph_digital_label(n: u32) -> String {
    const DIGITS: [char; 10] = ['〇', '一', '二', '三', '四', '五', '六', '七', '八', '九'];
    if n == 0 {
        return "〇".into();
    }
    let mut buf = Vec::new();
    let mut x = n;
    while x > 0 {
        buf.push(DIGITS[(x % 10) as usize]);
        x /= 10;
    }
    buf.iter().rev().collect()
}

/// Heavenly stems. [MS-DOCX] ideographTraditional: U+7532, U+4E59, U+4E19, …
const IDEOGRAPH_STEMS: [char; 10] = ['甲', '乙', '丙', '丁', '戊', '己', '庚', '辛', '壬', '癸'];
/// Earthly branches. [MS-DOCX] ideographZodiac: U+5B50, U+4E11, U+5BC5, …
const IDEOGRAPH_BRANCHES: [char; 12] = [
    '子', '丑', '寅', '卯', '辰', '巳', '午', '未', '申', '酉', '戌', '亥',
];

#[doc(hidden)]
pub fn cycle_cjk(chars: &[char], n: u32) -> String {
    let i = n.saturating_sub(1) as usize % chars.len();
    chars[i].to_string()
}

#[doc(hidden)]
pub fn ideograph_zodiac_traditional_label(n: u32) -> String {
    let i = n.saturating_sub(1) as usize;
    format!("{}{}", IDEOGRAPH_STEMS[i % 10], IDEOGRAPH_BRANCHES[i % 12])
}

#[doc(hidden)]
pub fn ideograph_legal_traditional_label(n: u32) -> String {
    cjk_counting(n, true)
}

/// Positional CJK counting (十/百/千 and a myriad unit per 10⁴).
/// `legal`: traditional financial forms — 壹 is written before 佰/仟/萬
/// (only a leading 拾 below 20 stays bare) and one 零 marks a gap.
/// Japanese counting drops 一 before 十/百/千 and writes no zero.
#[doc(hidden)]
pub fn cjk_counting(n: u32, legal: bool) -> String {
    const JAPANESE: [char; 10] = ['零', '一', '二', '三', '四', '五', '六', '七', '八', '九'];
    // Traditional legal digits: [MS-DOCX] U+58F9, U+8CB3, U+53C3, …
    const LEGAL: [char; 10] = ['零', '壹', '貳', '參', '肆', '伍', '陸', '柒', '捌', '玖'];
    let (digits, units, myriad) = if legal {
        (&LEGAL, ['拾', '佰', '仟'], '萬')
    } else {
        (&JAPANESE, ['十', '百', '千'], '万')
    };
    if n == 0 {
        return digits[0].to_string();
    }
    let mut out = String::new();
    if n >= 10_000 {
        out.push_str(&cjk_counting(n / 10_000, legal));
        out.push(myriad);
    }
    let low = n % 10_000;
    let mut gap = false;
    for (place, div) in [(3usize, 1000u32), (2, 100), (1, 10), (0, 1)] {
        let d = ((low / div) % 10) as usize;
        if d == 0 {
            gap |= !out.is_empty();
            continue;
        }
        if gap && legal {
            out.push(digits[0]);
        }
        gap = false;
        let bare_one = d == 1 && place > 0 && (!legal || (out.is_empty() && n < 20));
        if !bare_one {
            out.push(digits[d]);
        }
        if place > 0 {
            out.push(units[place - 1]);
        }
    }
    out
}

#[doc(hidden)]
pub fn ideograph_enclosed_circle_label(n: u32) -> String {
    // U+3220..=U+3229 are ㈠–㈩. Past them Word writes plain decimal
    // (Word 16, a list and a footer PAGE at 11: "11", not 一一).
    const ENCLOSED: [char; 10] = ['㈠', '㈡', '㈢', '㈣', '㈤', '㈥', '㈦', '㈧', '㈨', '㈩'];
    if (1..=10).contains(&n) {
        ENCLOSED[(n - 1) as usize].to_string()
    } else {
        n.to_string()
    }
}

const CHINESE_DIGITS: [char; 10] = ['〇', '一', '二', '三', '四', '五', '六', '七', '八', '九'];

/// chineseCounting / taiwaneseCounting as Word writes them: counting words
/// to 九十九, then digit by digit with ○ (U+25CB) for zero (一○一).
#[doc(hidden)]
pub fn chinese_counting_label(n: u32) -> String {
    let digit = |d: u32| CHINESE_DIGITS[d as usize];
    match n {
        0 => '○'.to_string(),
        1..=9 => digit(n).to_string(),
        10..=99 => {
            let mut out = String::new();
            if n >= 20 {
                out.push(digit(n / 10));
            }
            out.push('十');
            if !n.is_multiple_of(10) {
                out.push(digit(n % 10));
            }
            out
        }
        _ => n
            .to_string()
            .chars()
            .map(|c| match c {
                '0' => '○',
                c => digit(c.to_digit(10).unwrap_or(0)),
            })
            .collect(),
    }
}

/// chineseCountingThousand / taiwaneseCountingThousand as Word writes them:
/// 一百〇一, 一百一十, 一千〇一十, 一万〇一 (one `zero` per gap, 一十 after
/// a higher unit, a bare 十 only at the front).
#[doc(hidden)]
pub fn chinese_counting_thousand(n: u32, zero: char, myriad: char) -> String {
    // `front`: nothing precedes, so ten is a bare 十 (十九, but 一百一十).
    fn below_myriad(n: u32, zero: char, front: bool) -> String {
        let mut out = String::new();
        let mut pending_zero = false;
        for (div, unit) in [
            (1000, Some('千')),
            (100, Some('百')),
            (10, Some('十')),
            (1, None),
        ] {
            let d = (n / div) % 10;
            if d == 0 {
                pending_zero |= !out.is_empty();
                continue;
            }
            if pending_zero {
                out.push(zero);
                pending_zero = false;
            }
            if !(unit == Some('十') && d == 1 && out.is_empty() && front) {
                out.push(CHINESE_DIGITS[d as usize]);
            }
            if let Some(u) = unit {
                out.push(u);
            }
        }
        out
    }
    if n == 0 {
        return zero.to_string();
    }
    if n < 10_000 {
        return below_myriad(n, zero, true);
    }
    let mut out = chinese_counting_thousand(n / 10_000, zero, myriad);
    out.push(myriad);
    let low = n % 10_000;
    if low > 0 {
        if low < 1000 {
            out.push(zero);
        }
        out.push_str(&below_myriad(low, zero, false));
    }
    out
}

/// MS-DOCX japaneseCounting: 一, 二, …, 十, 十一, 百, 千, 一万 (not
/// digit-wise 一〇).
#[doc(hidden)]
pub fn japanese_counting_label(n: u32) -> String {
    cjk_counting(n, false)
}

/// MS-DOCX aiueo: half-width katakana ｱｲｳ… (U+FF71). 46-mora gojūon.
const AIUEO: [char; 46] = [
    'ｱ', 'ｲ', 'ｳ', 'ｴ', 'ｵ', 'ｶ', 'ｷ', 'ｸ', 'ｹ', 'ｺ', 'ｻ', 'ｼ', 'ｽ', 'ｾ', 'ｿ', 'ﾀ', 'ﾁ', 'ﾂ', 'ﾃ',
    'ﾄ', 'ﾅ', 'ﾆ', 'ﾇ', 'ﾈ', 'ﾉ', 'ﾊ', 'ﾋ', 'ﾌ', 'ﾍ', 'ﾎ', 'ﾏ', 'ﾐ', 'ﾑ', 'ﾒ', 'ﾓ', 'ﾔ', 'ﾕ', 'ﾖ',
    'ﾗ', 'ﾘ', 'ﾙ', 'ﾚ', 'ﾛ', 'ﾜ', 'ｦ', 'ﾝ',
];

/// MS-DOCX iroha: half-width ｲﾛﾊ… (U+FF72). ゐ→ｲ, ゑ→ｴ.
const IROHA: [char; 47] = [
    'ｲ', 'ﾛ', 'ﾊ', 'ﾆ', 'ﾎ', 'ﾍ', 'ﾄ', 'ﾁ', 'ﾘ', 'ﾇ', 'ﾙ', 'ｦ', 'ﾜ', 'ｶ', 'ﾖ', 'ﾀ', 'ﾚ', 'ｿ', 'ﾂ',
    'ﾈ', 'ﾅ', 'ﾗ', 'ﾑ', 'ｳ', 'ｲ', 'ﾉ', 'ｵ', 'ｸ', 'ﾔ', 'ﾏ', 'ｹ', 'ﾌ', 'ｺ', 'ｴ', 'ﾃ', 'ｱ', 'ｻ', 'ｷ',
    'ﾕ', 'ﾒ', 'ﾐ', 'ｼ', 'ｴ', 'ﾋ', 'ﾓ', 'ｾ', 'ｽ',
];

/// MS-DOCX decimalFullWidth: U+FF10…U+FF19 (０１２…).
#[doc(hidden)]
pub fn decimal_fullwidth_label(n: u32) -> String {
    const DIGITS: [char; 10] = ['０', '１', '２', '３', '４', '５', '６', '７', '８', '９'];
    n.to_string()
        .bytes()
        .map(|b| DIGITS[usize::from(b - b'0')])
        .collect()
}

/// MS-DOCX ganada: U+AC00, U+B098, U+B2E4… 가나다라마바사아자차카타파하.
const GANADA: [char; 14] = [
    '가', '나', '다', '라', '마', '바', '사', '아', '자', '차', '카', '타', '파', '하',
];

/// MS-DOCX chosung: U+3131, U+3134, U+3137… ㄱㄴㄷㄹㅁㅂㅅㅇㅈㅊㅋㅌㅍㅎ.
const CHOSUNG: [char; 14] = [
    'ㄱ', 'ㄴ', 'ㄷ', 'ㄹ', 'ㅁ', 'ㅂ', 'ㅅ', 'ㅇ', 'ㅈ', 'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ',
];

/// MS-DOCX decimalEnclosedCircle: U+2460…U+2473 (①–⑳). After 20, decimal.
const DECIMAL_ENCLOSED_CIRCLE: [char; 20] = [
    '①', '②', '③', '④', '⑤', '⑥', '⑦', '⑧', '⑨', '⑩', '⑪', '⑫', '⑬', '⑭', '⑮', '⑯', '⑰', '⑱', '⑲',
    '⑳',
];
/// MS-DOCX decimalEnclosedParen: U+2474…U+2487 (⑴–⒇).
const DECIMAL_ENCLOSED_PAREN: [char; 20] = [
    '⑴', '⑵', '⑶', '⑷', '⑸', '⑹', '⑺', '⑻', '⑼', '⑽', '⑾', '⑿', '⒀', '⒁', '⒂', '⒃', '⒄', '⒅', '⒆',
    '⒇',
];
/// MS-DOCX decimalEnclosedFullstop: U+2488…U+249B (⒈–⒛).
const DECIMAL_ENCLOSED_FULLSTOP: [char; 20] = [
    '⒈', '⒉', '⒊', '⒋', '⒌', '⒍', '⒎', '⒏', '⒐', '⒑', '⒒', '⒓', '⒔', '⒕', '⒖', '⒗', '⒘', '⒙', '⒚',
    '⒛',
];

#[doc(hidden)]
pub fn enclosed_decimal_label(n: u32, glyphs: &[char; 20]) -> String {
    if (1..=20).contains(&n) {
        glyphs[(n - 1) as usize].to_string()
    } else {
        n.to_string()
    }
}

/// MS-DOCX hebrew1: Hebrew numerals א, י, ק… (U+05D0, U+05D9, U+05E7).
/// Additive gematria without geresh; 15=טו, 16=טז. After 999, decimal.
#[doc(hidden)]
pub fn hebrew1_label(n: u32) -> String {
    if n == 0 || n > 999 {
        return n.to_string();
    }
    let mut out = String::new();
    let hundreds = n / 100;
    let mut rest = n % 100;
    const HUNDREDS: [&str; 9] = ["ק", "ר", "ש", "ת", "תק", "תר", "תש", "תת", "תתק"];
    if hundreds >= 1 {
        out.push_str(HUNDREDS[(hundreds - 1) as usize]);
    }
    if rest == 15 {
        out.push('ט');
        out.push('ו');
        return out;
    }
    if rest == 16 {
        out.push('ט');
        out.push('ז');
        return out;
    }
    const TENS: [char; 9] = ['י', 'כ', 'ל', 'מ', 'נ', 'ס', 'ע', 'פ', 'צ'];
    const ONES: [char; 9] = ['א', 'ב', 'ג', 'ד', 'ה', 'ו', 'ז', 'ח', 'ט'];
    let tens = rest / 10;
    rest %= 10;
    if tens > 0 {
        out.push(TENS[(tens - 1) as usize]);
    }
    if rest > 0 {
        out.push(ONES[(rest - 1) as usize]);
    }
    out
}

/// MS-DOCX hebrew2: Hebrew alphabet אבג… (U+05D0, U+05D1, U+05D2). 22 letters.
const HEBREW2: [char; 22] = [
    'א', 'ב', 'ג', 'ד', 'ה', 'ו', 'ז', 'ח', 'ט', 'י', 'כ', 'ל', 'מ', 'נ', 'ס', 'ע', 'פ', 'צ', 'ק',
    'ר', 'ש', 'ת',
];

/// MS-DOCX arabicAlpha: أ ب ت… (U+0623, U+0628, U+062A). Hijāʾī, hamza-alef first.
const ARABIC_ALPHA: [char; 28] = [
    'أ', 'ب', 'ت', 'ث', 'ج', 'ح', 'خ', 'د', 'ذ', 'ر', 'ز', 'س', 'ش', 'ص', 'ض', 'ط', 'ظ', 'ع', 'غ',
    'ف', 'ق', 'ك', 'ل', 'م', 'ن', 'ه', 'و', 'ي',
];

/// MS-DOCX arabicAbjad: أ ب ج… (U+0623, U+0628, U+062C). Abjadī, hamza-alef first.
const ARABIC_ABJAD: [char; 28] = [
    'أ', 'ب', 'ج', 'د', 'ه', 'و', 'ز', 'ح', 'ط', 'ي', 'ك', 'ل', 'م', 'ن', 'س', 'ع', 'ف', 'ص', 'ق',
    'ر', 'ش', 'ت', 'ث', 'خ', 'ذ', 'ض', 'ظ', 'غ',
];

/// MS-DOCX russianLower: абв… (U+0430). 33-letter Russian including ё as 7.
const RUSSIAN_LOWER: [char; 33] = [
    'а', 'б', 'в', 'г', 'д', 'е', 'ё', 'ж', 'з', 'и', 'й', 'к', 'л', 'м', 'н', 'о', 'п', 'р', 'с',
    'т', 'у', 'ф', 'х', 'ц', 'ч', 'ш', 'щ', 'ъ', 'ы', 'ь', 'э', 'ю', 'я',
];
/// MS-DOCX russianUpper: АБВ… (U+0410). Ё is U+0401, not sequential after Е.
const RUSSIAN_UPPER: [char; 33] = [
    'А', 'Б', 'В', 'Г', 'Д', 'Е', 'Ё', 'Ж', 'З', 'И', 'Й', 'К', 'Л', 'М', 'Н', 'О', 'П', 'Р', 'С',
    'Т', 'У', 'Ф', 'Х', 'Ц', 'Ч', 'Ш', 'Щ', 'Ъ', 'Ы', 'Ь', 'Э', 'Ю', 'Я',
];

/// MS-DOCX thaiNumbers: U+0E51… (๑๒๓). Digit-wise, like decimalFullWidth.
#[doc(hidden)]
pub fn thai_numbers_label(n: u32) -> String {
    const DIGITS: [char; 10] = ['๐', '๑', '๒', '๓', '๔', '๕', '๖', '๗', '๘', '๙'];
    n.to_string()
        .bytes()
        .map(|b| DIGITS[usize::from(b - b'0')])
        .collect()
}

/// MS-DOCX/ECMA thaiLetters: ก ข ค… (U+0E01, U+0E02, U+0E04). Skip obsolete ฃ/ฅ and vowels ฤ/ฦ.
const THAI_LETTERS: [char; 42] = [
    'ก', 'ข', 'ค', 'ฆ', 'ง', 'จ', 'ฉ', 'ช', 'ซ', 'ฌ', 'ญ', 'ฎ', 'ฏ', 'ฐ', 'ฑ', 'ฒ', 'ณ', 'ด', 'ต',
    'ถ', 'ท', 'ธ', 'น', 'บ', 'ป', 'ผ', 'ฝ', 'พ', 'ฟ', 'ภ', 'ม', 'ย', 'ร', 'ล', 'ว', 'ศ', 'ษ', 'ส',
    'ห', 'ฬ', 'อ', 'ฮ',
];

/// MS-DOCX thaiCounting: หนึ่ง, สอง, สาม… (U+0E2B U+0E19 U+0E36 U+0E48 U+0E07).
#[doc(hidden)]
pub fn thai_counting_label(n: u32) -> String {
    const ONES: [&str; 10] = [
        "",
        "หนึ่ง",
        "สอง",
        "สาม",
        "สี่",
        "ห้า",
        "หก",
        "เจ็ด",
        "แปด",
        "เก้า",
    ];
    match n {
        0 => "ศูนย์".into(),
        1..=9 => ONES[n as usize].into(),
        10 => "สิบ".into(),
        11 => "สิบเอ็ด".into(),
        12..=19 => format!("สิบ{}", ONES[(n - 10) as usize]),
        20..=99 => {
            let tens = n / 10;
            let ones = n % 10;
            let head = if tens == 2 {
                "ยี่สิบ".into()
            } else {
                format!("{}สิบ", ONES[tens as usize])
            };
            match ones {
                0 => head,
                1 => format!("{head}เอ็ด"),
                _ => format!("{head}{}", ONES[ones as usize]),
            }
        }
        n => n.to_string(),
    }
}

/// MS-DOCX hindiNumbers: U+0967… (१२३). Digit-wise Devanagari.
#[doc(hidden)]
pub fn hindi_numbers_label(n: u32) -> String {
    const DIGITS: [char; 10] = ['०', '१', '२', '३', '४', '५', '६', '७', '८', '९'];
    n.to_string()
        .bytes()
        .map(|b| DIGITS[usize::from(b - b'0')])
        .collect()
}

/// MS-DOCX hindiVowels: क ख ग… (U+0915). 33 Devanagari consonants despite the name.
const HINDI_VOWELS: [char; 33] = [
    'क', 'ख', 'ग', 'घ', 'ङ', 'च', 'छ', 'ज', 'झ', 'ञ', 'ट', 'ठ', 'ड', 'ढ', 'ण', 'त', 'थ', 'द', 'ध',
    'न', 'प', 'फ', 'ब', 'भ', 'म', 'य', 'र', 'ल', 'व', 'श', 'ष', 'स', 'ह',
];

/// MS-DOCX hindiConsonants: अ आ इ… (U+0905). Independent vowels despite the name.
const HINDI_CONSONANTS: [char; 11] = ['अ', 'आ', 'इ', 'ई', 'उ', 'ऊ', 'ऋ', 'ए', 'ऐ', 'ओ', 'औ'];

/// MS-DOCX hindiCounting: एक, दो, तीन… (U+090F U+0915). After 20, decimal.
#[doc(hidden)]
pub fn hindi_counting_label(n: u32) -> String {
    const WORDS: [&str; 21] = [
        "शून्य",
        "एक",
        "दो",
        "तीन",
        "चार",
        "पाँच",
        "छह",
        "सात",
        "आठ",
        "नौ",
        "दस",
        "ग्यारह",
        "बारह",
        "तेरह",
        "चौदह",
        "पंद्रह",
        "सोलह",
        "सत्रह",
        "अठारह",
        "उन्नीस",
        "बीस",
    ];
    if (n as usize) < WORDS.len() {
        WORDS[n as usize].into()
    } else {
        n.to_string()
    }
}

/// MS-DOCX koreanCounting: 일, 이, 삼, 십, 십일, 백 (U+C77C). Not digit-wise 일영.
/// A unit multiplier of one is dropped (십, 백, 천, never 일십).
#[doc(hidden)]
pub fn korean_counting_label(n: u32) -> String {
    const DIGITS: [&str; 10] = ["영", "일", "이", "삼", "사", "오", "육", "칠", "팔", "구"];
    const UNITS: [(u32, &str); 3] = [(1000, "천"), (100, "백"), (10, "십")];
    if n == 0 || n > 9999 {
        return if n == 0 { "영".into() } else { n.to_string() };
    }
    let mut s = String::new();
    let mut rest = n;
    for (unit, name) in UNITS {
        let d = rest / unit;
        rest %= unit;
        if d > 1 {
            s.push_str(DIGITS[d as usize]);
        }
        if d > 0 {
            s.push_str(name);
        }
    }
    if rest > 0 {
        s.push_str(DIGITS[rest as usize]);
    }
    s
}

/// MS-DOCX koreanDigital: 일, 일영, 일영영… Hangul digits with 영 for 0.
#[doc(hidden)]
pub fn korean_digital_label(n: u32) -> String {
    const DIGITS: [&str; 10] = ["영", "일", "이", "삼", "사", "오", "육", "칠", "팔", "구"];
    n.to_string()
        .bytes()
        .map(|b| DIGITS[usize::from(b - b'0')])
        .collect()
}

/// MS-DOCX koreanDigital2: 一, 一零… (U+96F6), not ideographDigital 一〇 (U+3007).
#[doc(hidden)]
pub fn korean_digital2_label(n: u32) -> String {
    const DIGITS: [char; 10] = ['零', '一', '二', '三', '四', '五', '六', '七', '八', '九'];
    n.to_string()
        .bytes()
        .map(|b| DIGITS[usize::from(b - b'0')])
        .collect()
}

/// MS-DOCX koreanLegal: 하나, 둘, 셋… (U+D558 U+B098). Native Korean.
#[doc(hidden)]
pub fn korean_legal_label(n: u32) -> String {
    const ONES: [&str; 10] = [
        "", "하나", "둘", "셋", "넷", "다섯", "여섯", "일곱", "여덟", "아홉",
    ];
    const TENS: [&str; 10] = [
        "", "열", "스물", "서른", "마흔", "쉰", "예순", "일흔", "여든", "아흔",
    ];
    match n {
        0 => "영".into(),
        1..=9 => ONES[n as usize].into(),
        10..=99 => {
            let tens = n / 10;
            let ones = n % 10;
            if ones == 0 {
                TENS[tens as usize].into()
            } else {
                format!("{}{}", TENS[tens as usize], ONES[ones as usize])
            }
        }
        n => n.to_string(),
    }
}

/// The last letter label Word writes, thirty z's.
#[doc(hidden)]
pub const ALPHA_LABEL_MAX: u32 = 26 * 30;
/// The last Roman numeral Word writes.
#[doc(hidden)]
pub const ROMAN_LABEL_MAX: u32 = 32767;

/// `lowerLetter` / `upperLetter` and `\* alphabetic`: past z Word repeats
/// the letter, 27 "aa", 28 "bb", 53 "aaa" (Word 16 probes la1006 and
/// nf1006, 2026-10-06). Past thirty z's (780) a list label starts again
/// at "a" (probe bn1006b). Zero counts as 1.
#[doc(hidden)]
pub fn alpha_label(n: u32, upper: bool) -> String {
    let n = (n.max(1) - 1) % ALPHA_LABEL_MAX;
    let ch = char::from(b'a' + (n % 26) as u8);
    let ch = if upper { ch.to_ascii_uppercase() } else { ch };
    std::iter::repeat_n(ch, (n / 26) as usize + 1).collect()
}

/// Past 3999 Word adds M's (4000 "MMMM"); past 32767 a list label starts
/// again at "I", 100000 "MDCXCIX" (Word 16 probe bn1006b, 2026-10-06).
#[doc(hidden)]
pub fn roman_label(n: u32, upper: bool) -> String {
    if n == 0 {
        return "0".into();
    }
    let mut n = (n - 1) % ROMAN_LABEL_MAX + 1;
    const MAP: &[(u32, &str)] = &[
        (1000, "m"),
        (900, "cm"),
        (500, "d"),
        (400, "cd"),
        (100, "c"),
        (90, "xc"),
        (50, "l"),
        (40, "xl"),
        (10, "x"),
        (9, "ix"),
        (5, "v"),
        (4, "iv"),
        (1, "i"),
    ];
    let mut out = String::new();
    for (val, sym) in MAP {
        while n >= *val {
            out.push_str(sym);
            n -= *val;
        }
    }
    if upper {
        out.make_ascii_uppercase();
    }
    out
}

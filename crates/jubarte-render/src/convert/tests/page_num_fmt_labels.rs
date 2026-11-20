// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::{
    NumFmt, chicago_label, decimal_fullwidth_label, format_num, ideograph_digital_label,
    ideograph_enclosed_circle_label, ideograph_legal_traditional_label,
    ideograph_zodiac_traditional_label, japanese_counting_label, parse_num_fmt,
};

#[test]
fn hex_is_uppercase() {
    assert_eq!(format_num(NumFmt::Hex, 10), "A");
    assert_eq!(format_num(NumFmt::Hex, 16), "10");
}

#[test]
fn chicago_cycles_star_dagger_double_section() {
    assert_eq!(chicago_label(1), "*");
    assert_eq!(chicago_label(2), "†");
    assert_eq!(chicago_label(5), "**");
}

#[test]
fn chicago_label_does_not_grow_with_a_huge_start() {
    // 1025 is the first value whose uncapped mark is 257 stars.
    let label = chicago_label(1025);
    assert_eq!(label, "*".repeat(256), "len {}", label.chars().count());
}

#[test]
fn ideograph_digital_uses_cjk_digits() {
    assert_eq!(ideograph_digital_label(1), "一");
    assert_eq!(ideograph_digital_label(10), "一〇");
    assert_eq!(ideograph_digital_label(12), "一二");
}

#[test]
fn ideograph_traditional_is_heavenly_stems() {
    assert_eq!(format_num(NumFmt::IdeographTraditional, 1), "甲");
    assert_eq!(format_num(NumFmt::IdeographTraditional, 2), "乙");
    assert_eq!(format_num(NumFmt::IdeographTraditional, 11), "甲");
}

#[test]
fn ideograph_zodiac_is_earthly_branches() {
    assert_eq!(format_num(NumFmt::IdeographZodiac, 1), "子");
    assert_eq!(format_num(NumFmt::IdeographZodiac, 3), "寅");
    assert_eq!(format_num(NumFmt::IdeographZodiac, 13), "子");
}

#[test]
fn ideograph_zodiac_traditional_is_sexagenary() {
    assert_eq!(ideograph_zodiac_traditional_label(1), "甲子");
    assert_eq!(ideograph_zodiac_traditional_label(2), "乙丑");
}

#[test]
fn ideograph_legal_traditional_is_financial_forms() {
    assert_eq!(ideograph_legal_traditional_label(1), "壹");
    assert_eq!(ideograph_legal_traditional_label(2), "貳");
    assert_eq!(ideograph_legal_traditional_label(10), "拾");
    assert_eq!(ideograph_legal_traditional_label(11), "拾壹");
    assert_eq!(ideograph_legal_traditional_label(20), "貳拾");
    assert_eq!(ideograph_legal_traditional_label(99), "玖拾玖");
}

#[test]
fn ideograph_legal_traditional_counts_past_ninety_nine() {
    // #157: 100+ fell through to Arabic "100". Financial numerals write
    // the leading 壹 before 佰/仟/萬 and a single 零 across gaps.
    for (n, want) in [
        (100, "壹佰"),
        (101, "壹佰零壹"),
        (110, "壹佰壹拾"),
        (111, "壹佰壹拾壹"),
        (999, "玖佰玖拾玖"),
        (1000, "壹仟"),
        (1001, "壹仟零壹"),
        (1010, "壹仟零壹拾"),
        (10000, "壹萬"),
        (10001, "壹萬零壹"),
        (12345, "壹萬貳仟參佰肆拾伍"),
    ] {
        assert_eq!(ideograph_legal_traditional_label(n), want, "{n}");
        assert_eq!(
            format_num(NumFmt::IdeographLegalTraditional, n),
            want,
            "{n}"
        );
    }
}

#[test]
fn japanese_counting_counts_past_ninety_nine() {
    // #158: 100+ fell through to Arabic. Japanese drops the leading 一
    // before 十/百/千 (not before 万) and writes no zero.
    for (n, want) in [
        (99, "九十九"),
        (100, "百"),
        (101, "百一"),
        (110, "百十"),
        (111, "百十一"),
        (200, "二百"),
        (1000, "千"),
        (1111, "千百十一"),
        (2024, "二千二十四"),
        (10000, "一万"),
        (10010, "一万十"),
        (100_000, "十万"),
    ] {
        assert_eq!(japanese_counting_label(n), want, "{n}");
        assert_eq!(format_num(NumFmt::JapaneseCounting, n), want, "{n}");
    }
}

#[test]
fn chinese_and_taiwanese_counting_match_word() {
    // Word's oracle (fixtures_500 00243d36 "一、", 0017ac0f): counting
    // formats count to 99 then go digit-wise with ○ (U+25CB); the
    // Thousand formats count through, zero 〇 (U+3007) / 零, myriad 万 / 萬.
    let cases: [(&str, &[(u32, &str)]); 4] = [
        (
            "chineseCounting",
            &[
                (1, "一"),
                (10, "十"),
                (11, "十一"),
                (20, "二十"),
                (21, "二十一"),
                (99, "九十九"),
                (100, "一○○"),
                (101, "一○一"),
                (110, "一一○"),
                (1001, "一○○一"),
                (10001, "一○○○一"),
            ],
        ),
        ("taiwaneseCounting", &[(15, "十五"), (120, "一二○")]),
        (
            "chineseCountingThousand",
            &[
                (10, "十"),
                (19, "十九"),
                (100, "一百"),
                (101, "一百〇一"),
                (110, "一百一十"),
                (1010, "一千〇一十"),
                (1100, "一千一百"),
                (10000, "一万"),
                (10001, "一万〇一"),
            ],
        ),
        (
            "taiwaneseCountingThousand",
            &[
                (21, "二十一"),
                (101, "一百零一"),
                (111, "一百一十一"),
                (1001, "一千零一"),
                (10000, "一萬"),
                (10001, "一萬零一"),
            ],
        ),
    ];
    for (fmt, rows) in cases {
        for &(n, want) in rows {
            assert_eq!(format_num(parse_num_fmt(fmt), n), want, "{fmt} {n}");
        }
    }
}

#[test]
fn ideograph_enclosed_circle_is_parenthesized() {
    assert_eq!(ideograph_enclosed_circle_label(1), "㈠");
    assert_eq!(ideograph_enclosed_circle_label(2), "㈡");
    // Word 16 (probe lst 1001, and a footer PAGE): past ㈩ the label is
    // plain decimal, "11.", not 一一.
    assert_eq!(ideograph_enclosed_circle_label(11), "11");
}

#[test]
fn japanese_counting_uses_ten_not_digital_zero() {
    assert_eq!(japanese_counting_label(1), "一");
    assert_eq!(japanese_counting_label(10), "十");
    assert_eq!(japanese_counting_label(11), "十一");
    assert_eq!(japanese_counting_label(21), "二十一");
    assert_eq!(ideograph_digital_label(10), "一〇");
}

#[test]
fn aiueo_is_halfwidth_katakana_gojuon() {
    assert_eq!(format_num(NumFmt::Aiueo, 1), "ｱ");
    assert_eq!(format_num(NumFmt::Aiueo, 2), "ｲ");
    assert_eq!(format_num(NumFmt::Aiueo, 3), "ｳ");
}

#[test]
fn iroha_is_halfwidth_iroha_order() {
    assert_eq!(format_num(NumFmt::Iroha, 1), "ｲ");
    assert_eq!(format_num(NumFmt::Iroha, 2), "ﾛ");
    assert_eq!(format_num(NumFmt::Iroha, 3), "ﾊ");
}

#[test]
fn decimal_fullwidth_uses_ff10_digits() {
    assert_eq!(decimal_fullwidth_label(1), "１");
    assert_eq!(decimal_fullwidth_label(10), "１０");
    assert_eq!(format_num(NumFmt::DecimalFullWidth, 10), "１０");
}

#[test]
fn ganada_is_hangul_syllable_order() {
    assert_eq!(format_num(NumFmt::Ganada, 1), "가");
    assert_eq!(format_num(NumFmt::Ganada, 2), "나");
    assert_eq!(format_num(NumFmt::Ganada, 3), "다");
}

#[test]
fn chosung_is_hangul_jamo_order() {
    assert_eq!(format_num(NumFmt::Chosung, 1), "ㄱ");
    assert_eq!(format_num(NumFmt::Chosung, 2), "ㄴ");
    assert_eq!(format_num(NumFmt::Chosung, 3), "ㄷ");
}

#[test]
fn decimal_enclosed_circle_is_circled_digits() {
    assert_eq!(format_num(NumFmt::DecimalEnclosedCircle, 1), "①");
    assert_eq!(format_num(NumFmt::DecimalEnclosedCircle, 2), "②");
    assert_eq!(format_num(NumFmt::DecimalEnclosedCircle, 20), "⑳");
    assert_eq!(format_num(NumFmt::DecimalEnclosedCircle, 21), "21");
}

#[test]
fn decimal_enclosed_paren_and_fullstop() {
    assert_eq!(format_num(NumFmt::DecimalEnclosedParen, 1), "⑴");
    assert_eq!(format_num(NumFmt::DecimalEnclosedParen, 2), "⑵");
    assert_eq!(format_num(NumFmt::DecimalEnclosedParen, 20), "⒇");
    assert_eq!(format_num(NumFmt::DecimalEnclosedFullstop, 1), "⒈");
    assert_eq!(format_num(NumFmt::DecimalEnclosedFullstop, 2), "⒉");
    assert_eq!(format_num(NumFmt::DecimalEnclosedFullstop, 20), "⒛");
    assert_eq!(format_num(NumFmt::DecimalEnclosedParen, 21), "21");
    assert_eq!(format_num(NumFmt::DecimalEnclosedFullstop, 21), "21");
}

#[test]
fn hebrew1_is_gematria_without_geresh() {
    assert_eq!(format_num(NumFmt::Hebrew1, 1), "א");
    assert_eq!(format_num(NumFmt::Hebrew1, 10), "י");
    assert_eq!(format_num(NumFmt::Hebrew1, 11), "יא");
    assert_eq!(format_num(NumFmt::Hebrew1, 15), "טו");
    assert_eq!(format_num(NumFmt::Hebrew1, 100), "ק");
    // #161: 16 avoids the divine-name spelling like 15; 0 and >999
    // fall back to decimal.
    assert_eq!(format_num(NumFmt::Hebrew1, 16), "טז");
    assert_eq!(format_num(NumFmt::Hebrew1, 0), "0");
    assert_eq!(format_num(NumFmt::Hebrew1, 999), "תתקצט");
    assert_eq!(format_num(NumFmt::Hebrew1, 1000), "1000");
}

#[test]
fn hebrew2_is_hebrew_alphabet() {
    assert_eq!(format_num(NumFmt::Hebrew2, 1), "א");
    assert_eq!(format_num(NumFmt::Hebrew2, 2), "ב");
    assert_eq!(format_num(NumFmt::Hebrew2, 3), "ג");
}

#[test]
fn arabic_alpha_is_hijai_not_abjad() {
    assert_eq!(format_num(NumFmt::ArabicAlpha, 1), "أ");
    assert_eq!(format_num(NumFmt::ArabicAlpha, 2), "ب");
    assert_eq!(format_num(NumFmt::ArabicAlpha, 3), "ت");
    assert_eq!(format_num(NumFmt::ArabicAbjad, 3), "ج");
}

#[test]
fn russian_letters_are_cyrillic_alphabet() {
    assert_eq!(format_num(NumFmt::RussianLower, 1), "а");
    assert_eq!(format_num(NumFmt::RussianLower, 2), "б");
    assert_eq!(format_num(NumFmt::RussianLower, 3), "в");
    assert_eq!(format_num(NumFmt::RussianLower, 7), "ё");
    assert_eq!(format_num(NumFmt::RussianLower, 33), "я");
    assert_eq!(format_num(NumFmt::RussianUpper, 1), "А");
    assert_eq!(format_num(NumFmt::RussianUpper, 7), "Ё");
    assert_eq!(format_num(NumFmt::RussianUpper, 33), "Я");
}

#[test]
fn thai_numbers_use_thai_digits() {
    assert_eq!(format_num(NumFmt::ThaiNumbers, 1), "๑");
    assert_eq!(format_num(NumFmt::ThaiNumbers, 10), "๑๐");
}

#[test]
fn thai_letters_skip_obsolete_kho() {
    assert_eq!(format_num(NumFmt::ThaiLetters, 1), "ก");
    assert_eq!(format_num(NumFmt::ThaiLetters, 2), "ข");
    assert_eq!(format_num(NumFmt::ThaiLetters, 3), "ค");
}

#[test]
fn thai_counting_uses_thai_words() {
    assert_eq!(format_num(NumFmt::ThaiCounting, 1), "หนึ่ง");
    assert_eq!(format_num(NumFmt::ThaiCounting, 2), "สอง");
    assert_eq!(format_num(NumFmt::ThaiCounting, 10), "สิบ");
    assert_eq!(format_num(NumFmt::ThaiCounting, 11), "สิบเอ็ด");
}

#[test]
fn hindi_numbers_use_devanagari_digits() {
    assert_eq!(format_num(NumFmt::HindiNumbers, 1), "१");
    assert_eq!(format_num(NumFmt::HindiNumbers, 10), "१०");
}

#[test]
fn hindi_vowels_are_ka_kha_ga() {
    assert_eq!(format_num(NumFmt::HindiVowels, 1), "क");
    assert_eq!(format_num(NumFmt::HindiVowels, 2), "ख");
    assert_eq!(format_num(NumFmt::HindiVowels, 3), "ग");
    assert_eq!(format_num(NumFmt::HindiConsonants, 1), "अ");
    assert_eq!(format_num(NumFmt::HindiConsonants, 3), "इ");
}

#[test]
fn hindi_counting_uses_hindi_words() {
    assert_eq!(format_num(NumFmt::HindiCounting, 1), "एक");
    assert_eq!(format_num(NumFmt::HindiCounting, 2), "दो");
    assert_eq!(format_num(NumFmt::HindiCounting, 3), "तीन");
    assert_eq!(format_num(NumFmt::HindiCounting, 10), "दस");
}

#[test]
fn korean_counting_uses_sip_not_digital_zero() {
    assert_eq!(format_num(NumFmt::KoreanCounting, 1), "일");
    assert_eq!(format_num(NumFmt::KoreanCounting, 10), "십");
    assert_eq!(format_num(NumFmt::KoreanCounting, 11), "십일");
    assert_eq!(format_num(NumFmt::KoreanCounting, 20), "이십");
    assert_eq!(format_num(NumFmt::KoreanCounting, 100), "백");
    assert_eq!(format_num(NumFmt::KoreanCounting, 305), "삼백오");
    assert_eq!(format_num(NumFmt::KoreanCounting, 1010), "천십");
    assert_eq!(format_num(NumFmt::KoreanDigital, 10), "일영");
    assert_eq!(ideograph_digital_label(10), "一〇");
    assert_eq!(format_num(NumFmt::KoreanDigital2, 10), "一零");
}

#[test]
fn korean_legal_is_native_hangul() {
    assert_eq!(format_num(NumFmt::KoreanLegal, 1), "하나");
    assert_eq!(format_num(NumFmt::KoreanLegal, 2), "둘");
    assert_eq!(format_num(NumFmt::KoreanLegal, 3), "셋");
    assert_eq!(format_num(NumFmt::KoreanLegal, 10), "열");
}
